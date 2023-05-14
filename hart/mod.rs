#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(non_camel_case_types)]

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

pub struct Hart
{
	pub registers: [u64; 32],
	pub pc: u64,
	pub memory: [u64; 100], // TODO: clearly 100 is insufficient
}

impl Default for Hart
{
	fn default() -> Hart
	{
		return Hart {
			registers: [0; 32],
			pc: 0,
			memory: [0; 100],
		};
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

	pub fn write_memory(&mut self, address: usize, value: u64)
	{
		self.registers[address] = value;
	}

	pub fn read_memory(&mut self, address: usize) -> u64
	{
		return self.registers[address];
	}
}
