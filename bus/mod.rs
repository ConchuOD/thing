// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::lebytes::LeBytes;
use std::fmt;

#[derive(Debug)]
pub enum BusErrorKind
{
	Unimplemented,
}

#[derive(Debug)]
pub struct BusError
{
	kind: BusErrorKind,
	details: String,
}

impl BusError
{
	pub fn new(kind: BusErrorKind, msg: &str) -> BusError
	{
		return BusError {
			kind,
			details: msg.to_string(),
		};
	}
}

impl fmt::Display for BusError
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		return write!(f, "BusError {:?}: {}", self.kind, self.details);
	}
}

pub trait Bus
{
	fn read<T>(&mut self, address: usize) -> Result<T, BusError>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:;

	fn write<T>(&mut self, address: usize, value: T) -> Result<(), BusError>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:;
}
