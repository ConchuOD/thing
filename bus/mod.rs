// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::lebytes::LeBytes;
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind
{
	Unimplemented,
	OutOfBounds,
}

#[derive(Debug)]
pub struct Error
{
	kind: ErrorKind,
	details: String,
}

impl Error
{
	pub fn new(kind: ErrorKind, msg: &str) -> Error
	{
		return Error {
			kind,
			details: msg.to_string(),
		};
	}
}

impl fmt::Display for Error
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		return write!(f, "BusError {:?}: {}", self.kind, self.details);
	}
}

pub trait Bus
{
	fn read<T, const T_SIZE: usize>(
		&mut self, address: usize,
	) -> Result<T, Error>
	where
		T: LeBytes<T_SIZE>,
		[(); T_SIZE]:;

	fn write<T, const T_SIZE: usize, U>(
		&mut self, address: U, value: T,
	) -> Result<(), Error>
	where
		T: LeBytes<T_SIZE>,
		U: Into<usize>,
		[(); T_SIZE]:;
}
