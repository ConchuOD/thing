// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

pub trait LeBytes
{
	const SIZE: usize;
	fn to_le_bytes(self) -> [u8; <Self as LeBytes>::SIZE];
	fn from_le_bytes(input: [u8; <Self as LeBytes>::SIZE]) -> Self;
}

impl LeBytes for u8
{
	const SIZE: usize = 1;

	fn to_le_bytes(self) -> [u8; <Self as LeBytes>::SIZE]
	{
		return u8::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; <Self as LeBytes>::SIZE]) -> Self
	{
		return u8::from_le_bytes(input);
	}
}

impl LeBytes for u16
{
	const SIZE: usize = 2;

	fn to_le_bytes(self) -> [u8; <Self as LeBytes>::SIZE]
	{
		return u16::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; <Self as LeBytes>::SIZE]) -> Self
	{
		return u16::from_le_bytes(input);
	}
}

impl LeBytes for u32
{
	const SIZE: usize = 4;

	fn to_le_bytes(self) -> [u8; <Self as LeBytes>::SIZE]
	{
		return u32::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; <Self as LeBytes>::SIZE]) -> Self
	{
		return u32::from_le_bytes(input);
	}
}

impl LeBytes for u64
{
	const SIZE: usize = 8;

	fn to_le_bytes(self) -> [u8; <Self as LeBytes>::SIZE]
	{
		return u64::to_le_bytes(self);
	}

	fn from_le_bytes(input: [u8; <Self as LeBytes>::SIZE]) -> Self
	{
		return u64::from_le_bytes(input);
	}
}
