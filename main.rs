#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use std::fs;

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
	((input[0] as u32) << 0)
		| ((input[1] as u32) << 8)
		| ((input[2] as u32) << 16)
		| ((input[3] as u32) << 24)
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
	let args = Args::parse();
	let elf: Vec<u8> = fs::read(args.elf)?;
	let mut registers: [u64; 32] = [0; 32];
	let mut pc: u64;

	let entry_point: usize = 0x142;
	let insn_start: usize = 0xe8;

	let program: &[u8] = &elf[insn_start..];

	pc = entry_point as u64;

	while (pc as usize) < program.len() {
		let insn_bits: &[u8] = &program[pc as usize..(pc + 4) as usize];
		let insn: u32 = u8s_to_insn(insn_bits.try_into()?);
		let something: insn::Insn = insn::Insn::from(insn);
		something.handle(&mut registers, &mut pc);
	}

	return Ok(());
}
