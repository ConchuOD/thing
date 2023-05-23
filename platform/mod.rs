// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::bus::{self, Bus};
use crate::hart::Hart;
use crate::insn::Insn;
use crate::lebytes::LeBytes;
use std::error::Error;

fn u8s_to_insn(input: &[u8; 4]) -> u32
{
	return (input[0] as u32)
		| ((input[1] as u32) << 8)
		| ((input[2] as u32) << 16)
		| ((input[3] as u32) << 24);
}

#[derive(Default)]
pub struct Platform
{
	pub hart: Hart,
	memory: Memory,
}

impl Platform
{
	pub fn load_file(
		&mut self, blob: Vec<u8>, load_address: usize, entry_point: usize,
	) -> Result<(), Box<dyn Error>>
	{
		self.hart.pc = entry_point as u64;

		let memory = &mut self.memory;

		if !(memory.start..memory.end).contains(&load_address) {
			return Err(Box::<dyn Error>::from(
				"load address out of bounds".to_string(),
			));
		}

		let elf_end = load_address + blob.len();
		if elf_end > memory.end {
			return Err(Box::<dyn Error>::from(
				"insufficient memory for blob".to_string(),
			));
		}

		memory.memory[load_address..elf_end].copy_from_slice(&blob[..]);

		return Ok(());
	}

	pub fn emulate(&mut self) -> Result<(), Box<dyn Error>>
	{
		let mut pc = self.hart.pc as usize;
		loop {
			let insn_bits: &[u8] = &self.memory.memory[pc..(pc + 4)];
			let insn: u32 = u8s_to_insn(insn_bits.try_into()?);
			let mut insn: Insn = Insn::from(insn);

			insn.handle(self);
			pc += 4;
		}
	}
}

impl Bus for Platform
{
	fn read<T>(&mut self, address: usize) -> Result<T, bus::Error>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		let memory = &self.memory;
		if (memory.start..memory.end).contains(&address) {
			return self.memory.read(address - MEMORY_BASE);
		}

		return Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			&format!("addr: {:}", address),
		));
	}

	fn write<T>(&mut self, address: usize, value: T) -> Result<(), bus::Error>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		let memory = &self.memory;
		if (memory.start..memory.end).contains(&address) {
			return self.memory.write(address - MEMORY_BASE, value);
		}

		return Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			&format!("addr: {:}", address),
		));
	}
}

const MEMORY_BASE: usize = 0x0;
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
	fn read<T>(&mut self, address: usize) -> Result<T, bus::Error>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		return Ok(T::from_le_bytes(
			self.memory[address..address + <T as LeBytes>::SIZE]
				.try_into()
				.unwrap(),
		));
	}

	fn write<T>(&mut self, address: usize, value: T) -> Result<(), bus::Error>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
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
