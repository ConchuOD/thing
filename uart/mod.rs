use std::fmt::Display;

use crate::{bus, lebytes::LeBytes};

#[derive(Debug, PartialEq)]
struct Uart
{
	receiver_buffer: ReadOnlyRegister,
	transmitter_holding: WriteOnlyRegister,
	interrupt_enable: Register,
	interrupt_ident: ReadOnlyRegister,
	line_control: Register,
	modem_control: Register,
	line_status: Register,
	modem_status: Register,
	scratch: Register,
	divisor_latch_ls: Register,
	divisor_latch_ms: Register,
}

impl Uart
{
	fn read_at(&self, address: RegisterAddress) -> Result<u8, Error>
	{
		use RegisterAddress::*;
		return match address {
			ReceiverBuffer => Ok(self.receiver_buffer.read()),
			TransmitterHolding => Err(Error::DisallowedRead),
			InterruptEnable => Ok(self.interrupt_enable.read()),
			InterruptIdent => Err(Error::DisallowedRead),
			LineControl => Ok(self.line_control.read()),
			ModemControl => Ok(self.modem_control.read()),
			LineStatus => Ok(self.line_status.read()),
			ModemStatus => Ok(self.modem_status.read()),
			Scratch => Ok(self.scratch.read()),
		};
	}

	fn write_at(
		&mut self, address: RegisterAddress, value: u8,
	) -> Result<(), Error>
	{
		use RegisterAddress::*;
		return match address {
			ReceiverBuffer => Err(Error::DisallowedWrite),
			TransmitterHolding => {
				self.transmitter_holding.write(value);
				Ok(())
			},
			InterruptEnable => {
				self.interrupt_enable.write(value);
				Ok(())
			},
			InterruptIdent => Err(Error::DisallowedWrite),
			LineControl => {
				self.line_control.write(value);
				Ok(())
			},
			ModemControl => {
				self.modem_control.write(value);
				Ok(())
			},
			LineStatus => Err(Error::DisallowedWrite),
			ModemStatus => Err(Error::DisallowedWrite),
			Scratch => {
				self.scratch.write(value);
				Ok(())
			},
		};
	}
}

impl Default for Uart
{
	fn default() -> Self
	{
		return Uart {
			receiver_buffer: ReadOnlyRegister::default(),
			transmitter_holding: WriteOnlyRegister::default(),
			interrupt_enable: Register::default(),
			interrupt_ident: ReadOnlyRegister::default(),
			line_control: Register::default(),
			modem_control: Register::default(),
			line_status: Register::default(),
			modem_status: Register::default(),
			scratch: Register::default(),
			divisor_latch_ls: Register::default(),
			divisor_latch_ms: Register::default(),
		};
	}
}

impl bus::Bus for Uart
{
	fn read<T>(&mut self, address: usize) -> Result<T, bus::Error>
	where
		T: crate::lebytes::LeBytes,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
		if <T as LeBytes>::SIZE > 1 {
			return Err(bus::Error::new(
				bus::ErrorKind::Unimplemented,
				"multi-byte reads are not implemented yet",
			));
		}

		let mut address = RegisterAddress::try_from(address)?;
		if address == RegisterAddress::TransmitterHolding {
			address = RegisterAddress::ReceiverBuffer;
		}
		let mut return_bytes = [0; <T as LeBytes>::SIZE];
		return_bytes[0] = self.read_at(address)?;
		return Ok(T::from_le_bytes(return_bytes));
	}

	fn write<T, U>(&mut self, address: U, value: T) -> Result<(), bus::Error>
	where
		T: crate::lebytes::LeBytes,
		U: Into<usize>,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
		let bytes: [u8; <T as LeBytes>::SIZE] = value.to_le_bytes();
		if bytes.len() > 1 {
			return Err(bus::Error::new(
				bus::ErrorKind::Unimplemented,
				"multi-byte writes are not implemented yet",
			));
		}

		let mut address: RegisterAddress = address.into().try_into()?;
		if address == RegisterAddress::ReceiverBuffer {
			address = RegisterAddress::TransmitterHolding;
		}
		self.write_at(address, bytes[0])?;
		self.receiver_buffer.bits = self.transmitter_holding.bits;
		return Ok(());
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RegisterAddress
{
	ReceiverBuffer = 0,
	TransmitterHolding = 1,
	InterruptEnable = 2,
	InterruptIdent = 3,
	LineControl = 4,
	ModemControl = 5,
	LineStatus = 6,
	ModemStatus = 7,
	Scratch = 8,
}

impl TryFrom<usize> for RegisterAddress
{
	type Error = AddressConvertError;
	fn try_from(value: usize) -> Result<Self, Self::Error>
	{
		use RegisterAddress::*;
		return match value {
			0 => Ok(ReceiverBuffer),
			1 => Ok(TransmitterHolding),
			2 => Ok(InterruptEnable),
			3 => Ok(InterruptIdent),
			4 => Ok(LineControl),
			5 => Ok(ModemControl),
			6 => Ok(LineStatus),
			7 => Ok(ModemStatus),
			8 => Ok(Scratch),
			_ => Err(AddressConvertError),
		};
	}
}

impl From<RegisterAddress> for u8
{
	fn from(val: RegisterAddress) -> Self
	{
		use RegisterAddress::*;
		return match val {
			ReceiverBuffer => 0,
			TransmitterHolding => 1,
			InterruptEnable => 2,
			InterruptIdent => 3,
			LineControl => 4,
			ModemControl => 5,
			LineStatus => 6,
			ModemStatus => 7,
			Scratch => 8,
		};
	}
}

impl From<RegisterAddress> for usize
{
	fn from(value: RegisterAddress) -> usize
	{
		return u8::from(value) as usize;
	}
}

impl Display for RegisterAddress
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		let v: usize = (*self).into();
		return write!(f, "{}", v);
	}
}

#[derive(Debug)]
struct AddressConvertError;

impl From<AddressConvertError> for bus::Error
{
	fn from(value: AddressConvertError) -> Self
	{
		_ = value; // NOTE: nothing interesting in here for now.
		return bus::Error::new(bus::ErrorKind::OutOfBounds, "todo, put a better error message here. needs more context. But uart::AddressConvertError implies that one tried to convert a numerical address into a uart address that does not exist");
	}
}

#[derive(Debug, PartialEq)]
struct Register
{
	bits: u8,
}

impl Register
{
	fn read(&self) -> u8
	{
		todo!("Register::read is not implemented yet!");
	}

	fn write(&self, _v: u8)
	{
		todo!("Register::write is not implemented yet!");
	}
}

impl Default for Register
{
	fn default() -> Self
	{
		return Self {
			bits: 0,
		};
	}
}

#[derive(Debug, PartialEq)]
struct ReadOnlyRegister
{
	bits: u8,
}

impl ReadOnlyRegister
{
	fn read(&self) -> u8
	{
		return self.bits;
	}
}

impl Default for ReadOnlyRegister
{
	fn default() -> Self
	{
		return Self {
			bits: 0,
		};
	}
}

#[derive(Debug, PartialEq)]
struct WriteOnlyRegister
{
	bits: u8,
}

impl WriteOnlyRegister
{
	fn write(&mut self, v: u8)
	{
		self.bits = v;
	}
}

impl Default for WriteOnlyRegister
{
	fn default() -> Self
	{
		return Self {
			bits: 0,
		};
	}
}

#[derive(Debug)]
enum Error
{
	DisallowedRead,
	DisallowedWrite,
}

impl From<Error> for bus::Error
{
	fn from(value: Error) -> Self
	{
		match value {
			Error::DisallowedRead => todo!("bus error disallowed read"),
			Error::DisallowedWrite => todo!("bus disallowed write"),
		}
	}
}

#[cfg(test)]
mod test
{
	use crate::bus::{self, Bus};
	use crate::uart::ReadOnlyRegister;

	use super::{RegisterAddress, Uart, WriteOnlyRegister};

	#[test]
	fn reading_from_address_0_returns_rbr_value()
	{
		let v = 27u8;
		let mut uart = Uart {
			receiver_buffer: ReadOnlyRegister {
				bits: v,
			},
			..Uart::default()
		};
		let actual = uart.read(0).unwrap();

		assert_eq!(v, actual);
	}

	#[test]
	fn writing_to_address_0_writes_to_thr()
	{
		let mut uart = Uart::default();
		let expected = WriteOnlyRegister {
			bits: 27,
		};
		uart.write(0usize, 27u8).unwrap();
		assert_eq!(uart.transmitter_holding, expected);
	}

	#[test]
	fn rbr_and_thr_are_the_same_register()
	{
		let mut uart = Uart::default();
		let value = 23u8;
		uart.write(RegisterAddress::ReceiverBuffer, value).unwrap();
		let res = uart
			.read::<u8>(RegisterAddress::TransmitterHolding.into())
			.unwrap();

		assert_eq!(res, value);
	}
	#[test]
	fn multi_byte_write_causes_error()
	{
		let mut uart = Uart::default();
		let expected = Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			"multi-byte writes are not implemented yet",
		));

		let res = uart
			.write(RegisterAddress::TransmitterHolding, 0b00000001_00000001u16);

		assert_eq!(res, expected);
	}
	#[test]
	fn multi_byte_read_causes_error()
	{
		let mut uart = Uart::default();
		let expected = Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			"multi-byte reads are not implemented yet",
		));

		let res = uart.read::<u16>(RegisterAddress::TransmitterHolding.into());

		assert_eq!(res, expected);
	}
}
