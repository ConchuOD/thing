// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::hart::Hart;
use crate::insn::Insn;

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
	hart: Hart,
	elf: Vec<u8>,
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

			something.handle(&mut self.hart);
		}

		return Ok(());
	}
}
