// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::bus::{Bus, BusError, BusErrorKind};
use crate::hart::Hart;
use crate::insn::Insn;
use crate::lebytes::LeBytes;

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
	elf: Vec<u8>,
	memory: Memory,
}

impl Platform
{
	pub fn load_file(&mut self, elf: Vec<u8>, entry_point: usize)
	{
		self.hart.pc = entry_point as u64;
		self.elf = elf;
	}

	pub fn emulate(&mut self) -> Result<(), Box<dyn std::error::Error>>
	{
		while (self.hart.pc as usize) < self.elf.len() {
			let insn_bits: &[u8] =
				&self.elf[self.hart.pc as usize..(self.hart.pc + 4) as usize];
			let insn: u32 = u8s_to_insn(insn_bits.try_into()?);
			let mut something: Insn = Insn::from(insn);

			something.handle(self);

			self.hart.pc += 4;
		}

		return Ok(());
	}
}

impl Bus for Platform
{
	fn read<T>(&mut self, address: usize) -> Result<T, BusError>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		if (MEMORY_BASE..MEMORY_END).contains(&address) {
			return self.memory.read(address - MEMORY_BASE);
		}

		return Err(BusError::new(
			BusErrorKind::Unimplemented,
			&format!("addr: {:}", address),
		));
	}

	fn write<T>(&mut self, address: usize, value: T) -> Result<(), BusError>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		if (MEMORY_BASE..MEMORY_END).contains(&address) {
			return self.memory.write(address - MEMORY_BASE, value);
		}

		return Err(BusError::new(
			BusErrorKind::Unimplemented,
			&format!("addr: {:}", address),
		));
	}
}

const MEMORY_BASE: usize = 0x0;
const MEMORY_SIZE: usize = 0x1000; // TODO: clearly 0x1000 is insufficient
const MEMORY_END: usize = MEMORY_BASE + MEMORY_SIZE;

pub struct Memory
{
	memory: [u8; MEMORY_SIZE],
}

impl Default for Memory
{
	fn default() -> Memory
	{
		return Memory {
			memory: [0; MEMORY_SIZE],
		};
	}
}

impl Bus for Memory
{
	fn read<T>(&mut self, address: usize) -> Result<T, BusError>
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

	fn write<T>(&mut self, address: usize, value: T) -> Result<(), BusError>
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
