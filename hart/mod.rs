// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(non_camel_case_types)]

use crate::bus::Bus;
use crate::lebytes::LeBytes;

pub enum RegisterNames
{
	zero,
	ra,
	sp,
	gp,
	tp,
	t0,
	t1,
	t2,
	s0,
	s1,
	a0,
	a1,
	a2,
	a3,
	a4,
	a5,
	a6,
	a7,
	s2,
	s3,
	s4,
	s5,
	s6,
	s7,
	s8,
	s9,
	s10,
	s11,
	t3,
	t4,
	t5,
	t6,
}

const MEMORY_BASE: usize = 0x100;
const MEMORY_SIZE: usize = 0x100; // TODO: clearly 0x100 is insufficient
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

pub struct Hart
{
	pub registers: [u64; 32],
	pub pc: u64,
	pub memory: Memory,
}

impl Default for Hart
{
	fn default() -> Hart
	{
		return Hart {
			registers: [0; 32],
			pc: 0,
			memory: Memory::default(),
		};
	}
}

impl Bus for Memory
{
	fn read<T>(&mut self, address: usize) -> T
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		return T::from_le_bytes(
			self.memory[address..address + <T as LeBytes>::SIZE - 1]
				.try_into()
				.unwrap(),
		);
	}

	fn write<T>(&mut self, address: usize, value: T)
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		let tmp: [u8; <T as LeBytes>::SIZE] = value.to_le_bytes();
		self.memory[address..address + <T as LeBytes>::SIZE - 1]
			.copy_from_slice(&tmp[..<T as LeBytes>::SIZE - 1]);
	}
}

impl Hart
{
	pub fn write_register(&mut self, offset: usize, value: u64)
	{
		self.registers[offset] = value;
	}

	pub fn read_register(&mut self, offset: usize) -> u64
	{
		return self.registers[offset];
	}
}

impl Bus for Hart
{
	fn write<T>(&mut self, address: usize, value: T)
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		if (MEMORY_BASE..MEMORY_END).contains(&address) {
			self.memory.write(address, value);
		}
	}

	fn read<T>(&mut self, address: usize) -> T
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		if (MEMORY_BASE..MEMORY_END).contains(&address) {
			return self.memory.read(address);
		} else {
			return T::from_le_bytes([0; <T as LeBytes>::SIZE]);
		}
	}
}
