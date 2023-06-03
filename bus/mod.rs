// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::lebytes::LeBytes;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorKind
{
	Unimplemented,
	OutOfBounds,
}

#[derive(Debug, PartialEq)]
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
	fn read<T>(&mut self, address: usize) -> Result<T, Error>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:;

	fn write<T, U>(&mut self, address: U, value: T) -> Result<(), Error>
	where
		U: Into<usize>,
		T: LeBytes,
		U: Into<usize>,
		[(); <T as LeBytes>::SIZE]:;
}
