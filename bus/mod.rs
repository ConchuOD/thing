#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::lebytes::LeBytes;

pub trait Bus
{
	fn read<T>(&mut self, address: usize) -> T
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:;

	fn write<T>(&mut self, address: usize, value: T)
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:;
}
