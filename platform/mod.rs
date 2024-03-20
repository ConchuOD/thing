// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::bus::{self, Bus};
use crate::hart::{Hart, RegisterNames};
use crate::insn::Insn;
use crate::lebytes::LeBytes;
use std::error::Error;
use debug_print::debug_println;

fn u8s_to_insn(input: &[u8; 4]) -> u32
{
	return (input[0] as u32)
		| ((input[1] as u32) << 8)
		| ((input[2] as u32) << 16)
		| ((input[3] as u32) << 24);
}

#[derive(Debug, Default)]
struct ReservationSet
{
	pub address: usize,
	pub size: usize,
	pub valid: bool,
	pub hart_id: usize,
}

#[derive(Default)]
pub struct Platform
{
	pub hart: Hart,
	memory: Memory,
	reservation_sets: Vec<ReservationSet>,
}

impl Platform
{
	pub fn load_dtb(
		&mut self, dtb: Vec<u8>, load_address: usize,
	) -> Result<(), Box<dyn Error>>
	{
		self.load_file(dtb, load_address)?;

		self.hart.registers[RegisterNames::a1 as usize] = load_address as u64;

		return Ok(());
	}

	pub fn load_kernel(
		&mut self, kernel: Vec<u8>, load_address: usize, entry_point: usize,
	) -> Result<(), Box<dyn Error>>
	{
		self.hart.pc = entry_point as u64;
		return self.load_file(kernel, load_address);
	}

	fn load_file(
		&mut self, blob: Vec<u8>, load_address: usize,
	) -> Result<(), Box<dyn Error>>
	{
		let memory = &mut self.memory;

		if !(memory.start..memory.end).contains(&load_address) {
			return Err(Box::<dyn Error>::from(
				"kernel load address out of bounds".to_string(),
			));
		}

		let blob_end = load_address + blob.len();
		if blob_end > memory.end {
			return Err(Box::<dyn Error>::from(
				"insufficient memory for kernel".to_string(),
			));
		}

		let memory_load_offset = load_address - memory.start;
		let memory_load_end = memory_load_offset + blob.len();
		memory.memory[memory_load_offset..memory_load_end]
			.copy_from_slice(&blob[..]);

		return Ok(());
	}

	pub fn emulate(&mut self) -> Result<(), Box<dyn Error>>
	{
		self.reservation_sets.push(ReservationSet::default());

		loop {
			let pc = self.hart.pc as usize - self.memory.start;
			let insn_bits: &[u8] = &self.memory.memory[pc..(pc + 4)];
			let insn: u32 = u8s_to_insn(insn_bits.try_into()?);
			let mut insn: Insn = Insn::from(insn);

			insn.handle(self);
		}
	}

	/// Claim a reservation set for this hart, replacing any existing one.
	/// Must be called with the bus write lock taken.
	pub fn claim_reservation_set<T>(
		&mut self, hart_id: usize, address: T, size: usize,
	) where
		T: Into<usize>,
	{
		// TODO; what if reservation exists?
		let address = usize::try_from(address).unwrap();
		let reservation_set = &mut self.reservation_sets[hart_id];

		reservation_set.hart_id = hart_id;
		reservation_set.address = address;
		reservation_set.size = size;
		reservation_set.valid = true;
	}

	/// Invalidates reservations taken by other harts that overlap with a
	/// store from this hart.
	/// Must be called with the bus write lock taken.
	pub fn invalidate_reservation_sets<T>(
		&mut self, hart_id: usize, address: T, size: usize,
	) where
		T: Into<usize>,
	{
		let address = usize::try_from(address).unwrap();

		for reservation_set in self.reservation_sets.iter_mut() {
			if reservation_set.hart_id == hart_id {
				continue;
			}

			if !reservation_set.valid {
				continue;
			}

			// We have to check that no bytes of the store intersect
			// with the reserved region. So for a 2 byte write, we
			// need to check that the second byte is not the first
			// of the reservation & so on
			let start = reservation_set.address - (size - 1);
			let end = reservation_set.address + reservation_set.size;
			if (start..end).contains(&address) {
				reservation_set.valid = false;
			}
		}
	}

	/// Check if a reservation for this hart is still valid, and if it is,
	/// invalidate it.
	/// Must be called with the bus write lock taken.
	pub fn check_invalidate_reservation_set<T>(
		&mut self, hart_id: usize, address: T, size: usize,
	) -> bool
	where
		T: Into<usize>,
	{
		let address = usize::try_from(address).unwrap();
		let reservation_set = &mut self.reservation_sets[hart_id];
		if !reservation_set.valid {
			return false;
		}

		// We have to check that no bytes of the store intersect
		// with the reserved region. So for a 2 byte write, we
		// need to check that the second byte is not the first
		// of the reservation & so on
		let start = reservation_set.address - (size - 1);
		let end = reservation_set.address + reservation_set.size;
		if !(start..end).contains(&address) {
			return false;
		}

		reservation_set.valid = false;
		return true;
	}

	pub fn write_from_hart<T>(
		&mut self, hart_id: usize, address: usize, value: T,
	) -> Result<(), bus::Error>
	where
		T: LeBytes,
		T: std::fmt::LowerHex,
		[(); <T as LeBytes>::SIZE]:,
	{
		self.invalidate_reservation_sets(
			hart_id,
			address,
			<T as LeBytes>::SIZE,
		);

		return self.write(address, value);
	}
}

impl Bus for Platform
{
	fn read<T>(&self, address: usize) -> Result<T, bus::Error>
	where
		T: LeBytes,
		T: std::fmt::LowerHex,
		[(); <T as LeBytes>::SIZE]:,
	{
		let memory = &self.memory;
		if (memory.start..memory.end).contains(&address) {
			let value = self.memory.read(address - MEMORY_BASE);
			debug_println!("reading {:x} from address {:x}", value.as_ref().unwrap(), address);
			return value;
		}

		return Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			&format!("Unimplemented read at addr: {:x}", address),
		));
	}

	fn write<T, U>(&mut self, address: U, value: T) -> Result<(), bus::Error>
	where
		T: LeBytes,
		T: std::fmt::LowerHex,
		U: Into<usize>,
		[(); <T as LeBytes>::SIZE]:,
	{
		let address = address.into();
		let memory = &self.memory;
		if (memory.start..memory.end).contains(&address) {
			debug_println!("writing {:x} into address {:x}", value, address);
			return self.memory.write(address - MEMORY_BASE, value);
		}

		return Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			&format!("Unimplemented write at addr: {:x}", address),
		));
	}
}

const MEMORY_BASE: usize = 0x8000_0000;
const MEMORY_SIZE: usize = 0x1000_0000;
const MEMORY_END: usize = MEMORY_BASE + MEMORY_SIZE;

fn heap_allocate_memory() -> Box<[u8]>
{
	let memory: Box<[u8]> = vec![0u8; MEMORY_SIZE].into_boxed_slice();
	return memory;
}

pub struct Memory
{
	start: usize,
	end: usize,
	memory: Box<[u8]>,
}

impl Memory
{
	pub fn size(self) -> usize
	{
		return self.end - self.start;
	}
}

impl Default for Memory
{
	fn default() -> Memory
	{
		return Memory {
			start: MEMORY_BASE,
			end: MEMORY_END,
			memory: heap_allocate_memory(),
		};
	}
}

impl Bus for Memory
{
	fn read<T>(&self, address: usize) -> Result<T, bus::Error>
	where
		T: LeBytes,
		T: std::fmt::LowerHex,
		[(); <T as LeBytes>::SIZE]:,
	{
		for n in 0..<T as LeBytes>::SIZE {
			println!("	{:x} {:x}", address + n, self.memory[address + n]);
		}
		return Ok(T::from_le_bytes(
			self.memory[address..address + <T as LeBytes>::SIZE]
				.try_into()
				.unwrap(),
		));
	}

	fn write<T, U>(&mut self, address: U, value: T) -> Result<(), bus::Error>
	where
		T: LeBytes,
		T: std::fmt::LowerHex,
		U: Into<usize>,
		[(); <T as LeBytes>::SIZE]:,
	{
		let address = address.into();
		let tmp: [u8; <T as LeBytes>::SIZE] = value.to_le_bytes();
		self.memory[address..address + <T as LeBytes>::SIZE]
			.copy_from_slice(&tmp[..<T as LeBytes>::SIZE]);

		return Ok(());
	}
}

#[cfg(test)]
mod test
{
	use crate::platform::MEMORY_SIZE;

	use super::heap_allocate_memory;

	#[test]
	fn can_heap_alloc()
	{
		let memory = heap_allocate_memory();
		assert_eq!(memory.len(), MEMORY_SIZE);
	}
}
