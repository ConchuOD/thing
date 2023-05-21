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
	blob: String,

	/// load address
	#[clap(short, long)]
	load_address: Option<usize>,

	/// entry point
	#[clap(short, long)]
	entry_point: Option<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
	let args = Args::parse();
	let blob: Vec<u8> = fs::read(args.blob)?;
	let mut load_address: usize = 0x0;
	let mut entry_point: usize = 0x164;

	if args.load_address.is_some() {
		load_address = args.load_address.unwrap();
	}

	if args.entry_point.is_some() {
		entry_point = args.entry_point.unwrap();
	}

	let mut platform: Platform = Platform::default();

	platform.load_file(blob, load_address, entry_point)?;
	return platform.emulate();
}
