// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

#[macro_export]
macro_rules! gen_mask {
	($h:expr, $l:expr, $typ:ty) => {
		(((!0) - (1_u64.wrapping_shl($l)) + 1)
			& (!0 & (!0_u64 >> (64 - 1 - ($h)) as u64))) as $typ
	};
}
