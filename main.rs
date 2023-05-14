#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use std::fs;

mod hart;
mod insn;

/// thing
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args
{
	/// input binary
	#[clap(short, long, default_value = "hi.elf")]
	elf: String,
}

fn u8s_to_insn(input: &[u8; 4]) -> u32
{
	return (input[0] as u32)
		| ((input[1] as u32) << 8)
		| ((input[2] as u32) << 16)
		| ((input[3] as u32) << 24);
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
	let args = Args::parse();
	let elf: Vec<u8> = fs::read(args.elf)?;

	// @Johan: "hart" is the RISC-V term for what Intel calls a "thread"
	let mut hart: hart::Hart = hart::Hart::default();

	let entry_point: usize = 0x164;
	let _insn_start: usize = 0xe8;

	// fe010113
	hart.pc = entry_point as u64;

	let mut hack: usize = 0;

	while (hart.pc as usize) < elf.len() {
		let insn_bits: &[u8] = &elf[hart.pc as usize..(hart.pc + 4) as usize];
		let insn: u32 = u8s_to_insn(insn_bits.try_into()?);
		let mut something: insn::Insn = insn::Insn::from(insn);

		something.handle(&mut hart);

		hack += 1;
		if hack > 20 {
			return Ok(());
		}
	}

	return Ok(());
}
