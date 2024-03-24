// SPDX-License-Identifier: GPL-2.0-only
#![feature(generic_const_exprs)]
#![feature(concat_idents)]
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use platform::Platform;
use crate::uart::Uart;
use std::fs;

mod bitfield;
mod bus;
mod hart;
mod insn;
mod lebytes;
mod platform;
mod uart;

/// thing
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args
{
	/// kernel
	#[clap(short, long, default_value = "vmlinux")]
	kernel: String,

	/// kernel load address
	#[clap(long)]
	kernel_load_address: Option<usize>,

	/// entry point
	#[clap(long)]
	entry_point: Option<usize>,

	/// dtb
	#[clap(short, long, default_value = "emu.dtb")]
	dtb: String,

	/// dtb load address
	#[clap(long)]
	dtb_load_address: Option<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
	let args = Args::parse();
	let mut kernel: Vec<u8> = fs::read(args.kernel)?;
	let dtb: Vec<u8> = fs::read(args.dtb)?;
	let mut kernel_load_address: usize = 0x8000_0000;
	let mut entry_point: usize = kernel_load_address;

	if args.kernel_load_address.is_some() {
		kernel_load_address = args.kernel_load_address.unwrap();
	}

	let mut dtb_load_address = kernel_load_address + dtb.len();

	if args.entry_point.is_some() {
		entry_point = args.entry_point.unwrap();
	}

	if args.dtb_load_address.is_some() {
		dtb_load_address = args.dtb_load_address.unwrap();
	}

	let mut platform: Platform = Platform::default();

	let stripped_blob: Vec<u8> = kernel.split_off(0x1000);
	platform.load_dtb(dtb, dtb_load_address)?;
	platform.load_kernel(stripped_blob, kernel_load_address, entry_point)?;
	return platform.emulate();
}
