// SPDX-License-Identifier: GPL-2.0-only
#![feature(generic_const_exprs)]
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use platform::Platform;
use std::fs;

mod bus;
mod hart;
mod insn;
mod lebytes;
mod platform;

/// thing
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args
{
	/// input binary
	#[clap(short, long, default_value = "hi.elf")]
	elf: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
	let args = Args::parse();
	let elf: Vec<u8> = fs::read(args.elf)?;

	let mut platform: Platform = Platform::default();

	let entry_point: usize = 0x164;

	platform.load_file(elf, entry_point);
	return platform.emulate();
}
