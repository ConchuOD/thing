#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;

/// thing
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// input yaml config file
	#[clap(short, long, default_value = "config.yaml")]
	config: String,
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
	return Ok(());
}
