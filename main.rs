#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use std::fs;

mod insn;

/// thing
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// input binary
	#[clap(short, long, default_value = "hi.elf")]
	elf: String,
}

fn u8s_to_insn(input: &[u8; 4]) -> u32 {
	((input[0] as u32) <<  0) |
	((input[1] as u32) <<  8) |
	((input[2] as u32) << 16) |
	((input[3] as u32) << 24)
}

fn main() -> Result<(),Box<dyn std::error::Error>> {

	let args = Args::parse();
	let elf: Vec<u8> = fs::read(args.elf)?;

	let entry_point: usize = 0x142;
	let insn_start: usize = 0xe8;

	let program: &[u8] = &elf[insn_start..];

	for n in 0..program.len() {
		if n % 4 == 0 {
			let insn_bits: &[u8] = &program[n..n+4];
			let insn: u32 = u8s_to_insn(insn_bits.try_into()?);
			let something: insn::Insn = insn::Insn::from(insn);
		}
	}

	return Ok(());
}
