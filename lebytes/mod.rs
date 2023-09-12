// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

pub trait LeBytes<const SIZE: usize>
{
	fn to_le_bytes(self) -> [u8; SIZE];
	fn from_le_bytes(input: [u8; SIZE]) -> Self;
}

impl LeBytes<1> for u8
{
	fn to_le_bytes(self) -> [u8; 1]
	{
		return u8::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; 1]) -> Self
	{
		return u8::from_le_bytes(input);
	}
}

impl LeBytes<2> for u16
{
	fn to_le_bytes(self) -> [u8; 2]
	{
		return u16::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; 2]) -> Self
	{
		return u16::from_le_bytes(input);
	}
}

impl LeBytes<4> for u32
{
	fn to_le_bytes(self) -> [u8; 4]
	{
		return u32::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; 4]) -> Self
	{
		return u32::from_le_bytes(input);
	}
}

impl LeBytes<8> for u64
{
	fn to_le_bytes(self) -> [u8; 8]
	{
		return u64::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; 8]) -> Self
	{
		return u64::from_le_bytes(input);
	}
}
