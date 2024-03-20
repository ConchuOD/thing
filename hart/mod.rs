// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(non_camel_case_types)]

use debug_print::debug_println;

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
#[derive(Debug)]
pub struct Hart
{
	pub registers: [u64; 32],
	pub csrs: [u64; 4096],
	pub pc: u64,
	pub id: usize,
}

impl Default for Hart
{
	fn default() -> Hart
	{
		return Hart {
			registers: [0; 32],
			csrs: [0; 4096],
			pc: 0,
			id: 0,
		};
	}
}

impl Hart
{
	pub fn write_register<T>(&mut self, offset: T, value: u64)
	where
		T: Into<usize>,
	{
		let offset = usize::try_from(offset).unwrap();
		if offset == 0 {
			return;
		}

		debug_println!("writing {:x} into register {:x}", value, offset);

		self.registers[offset] = value;
	}

	pub fn read_register<T>(&self, offset: T) -> u64
	where
		T: Into<usize>,
	{
		let offset = usize::try_from(offset).unwrap();
		if offset == 0 {
			return 0_u64;
		}

		let value = self.registers[offset];

		debug_println!("reading {:x} from register {:x}", value, offset);

		return value;
	}

	pub fn write_csr<T>(&mut self, offset: T, value: u64)
	where
		T: Into<usize>,
	{
		let offset = usize::try_from(offset).unwrap();
		self.csrs[offset] = value;
	}

	pub fn read_csr<T>(&self, offset: T) -> u64
	where
		T: Into<usize>,
	{
		let offset = usize::try_from(offset).unwrap();
		return self.csrs[offset];
	}
}
