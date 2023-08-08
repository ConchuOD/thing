use std::fmt::Display;
use std::io::Write;

use crate::{bus, lebytes::LeBytes};

#[derive(Debug, PartialEq, Default)]
struct UartRegisters
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

#[derive(Debug, PartialEq)]
struct Uart<'a, T>
where
	&'a mut T: std::io::Write,
{
	registers: UartRegisters,
	output: &'a mut T,
}

impl<'a, T> Uart<'a, T>
where
	&'a mut T: std::io::Write,
{
	fn new(output: &'a mut T) -> Self
	{
		return Self {
			registers: UartRegisters::default(),
			output,
		};
	}
	fn read_at(&self, address: RegisterAddress) -> Result<u8, Error>
	{
		use RegisterAddress::*;
		return match address {
			ReceiverBuffer => Ok(self.registers.receiver_buffer.read()),
			TransmitterHolding => Err(Error::DisallowedRead),
			InterruptEnable => Ok(self.registers.interrupt_enable.read()),
			InterruptIdent => Err(Error::DisallowedRead),
			LineControl => Ok(self.registers.line_control.read()),
			ModemControl => Ok(self.registers.modem_control.read()),
			LineStatus => Ok(self.registers.line_status.read()),
			ModemStatus => Ok(self.registers.modem_status.read()),
			Scratch => Ok(self.registers.scratch.read()),
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
				self.registers.transmitter_holding.write(value);
				Ok(())
			},
			InterruptEnable => {
				self.registers.interrupt_enable.write(value);
				Ok(())
			},
			InterruptIdent => Err(Error::DisallowedWrite),
			LineControl => {
				self.registers.line_control.write(value);
				Ok(())
			},
			ModemControl => {
				self.registers.modem_control.write(value);
				Ok(())
			},
			LineStatus => Err(Error::DisallowedWrite),
			ModemStatus => Err(Error::DisallowedWrite),
			Scratch => {
				self.registers.scratch.write(value);
				Ok(())
			},
		};
	}
}

impl<'a, V> bus::Bus for Uart<'a, V>
where
	&'a mut V: std::io::Write,
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
		self.registers.receiver_buffer.bits =
			self.registers.transmitter_holding.bits;
		let bits = self.registers.transmitter_holding.bits;
		self.output.write_all(&[bits]).unwrap();
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
	use crate::uart::{ReadOnlyRegister, UartRegisters};

	use super::{RegisterAddress, Uart, WriteOnlyRegister};

	#[derive(Default)]
	struct MockStdout
	{
		buf: Vec<u8>,
	}
	impl std::io::Write for MockStdout
	{
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
		{
			return self.buf.write(buf);
		}

		fn flush(&mut self) -> std::io::Result<()>
		{
			return self.buf.flush();
		}
	}
	#[test]
	fn reading_from_address_0_returns_rbr_value()
	{
		let v = 27u8;
		let mut uart = Uart::<MockStdout> {
			output: &mut MockStdout::default(),
			registers: UartRegisters {
				receiver_buffer: ReadOnlyRegister {
					bits: v,
				},
				..Default::default()
			},
		};

		let actual = uart.read(0).unwrap();

		assert_eq!(v, actual);
	}
	#[test]
	fn writing_to_address_0_writes_to_thr()
	{
		let mock_stdout = &mut MockStdout {
			buf: Vec::new(),
		};
		let mut uart = Uart::new(mock_stdout);
		let expected = WriteOnlyRegister {
			bits: b'f',
		};
		uart.write(0usize, b'f').unwrap();
		assert_eq!(uart.registers.transmitter_holding, expected);
	}
	#[test]
	fn rbr_and_thr_are_the_same_register()
	{
		let mut stdout = MockStdout {
			buf: Vec::new(),
		};
		let mut uart = Uart::<MockStdout>::new(&mut stdout);
		let value = b'a';
		uart.write(RegisterAddress::ReceiverBuffer, value).unwrap();
		let res = uart
			.read::<u8>(RegisterAddress::TransmitterHolding.into())
			.unwrap();

		assert_eq!(res, value);
	}
	#[test]
	fn multi_byte_write_causes_error()
	{
		let mut stdout = MockStdout {
			buf: Vec::new(),
		};
		let mut uart = Uart::<MockStdout>::new(&mut stdout);
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
		let mut stdout = MockStdout {
			buf: Vec::new(),
		};
		let mut uart = Uart::<MockStdout>::new(&mut stdout);
		let expected = Err(bus::Error::new(
			bus::ErrorKind::Unimplemented,
			"multi-byte reads are not implemented yet",
		));

		let res = uart.read::<u16>(RegisterAddress::TransmitterHolding.into());

		assert_eq!(res, expected);
	}

	#[test]
	fn can_plug_output()
	{
		let mut stdout = MockStdout::default();
		let s = &mut stdout;
		let mut uart = Uart::new(s);
		let bytes: Vec<u8> = "Hello, World!".bytes().collect();

		for byte in &bytes {
			println!("byte: {}, char: {}", byte, *byte as char);
			uart.write(RegisterAddress::TransmitterHolding, *byte).unwrap();
		}

		assert_eq!(bytes, stdout.buf);
	}

	#[test]
	fn can_output_to_file()
	{
		const TEST_FILE_PATH: &str = "test_output";
		let mut file = std::fs::File::create(TEST_FILE_PATH).unwrap();
		let mut uart = Uart::new(&mut file);
		let bytes: Vec<u8> = "Hello, World!".bytes().collect();

		for byte in &bytes {
			println!("byte: {}, char: {}", byte, *byte as char);
			uart.write(RegisterAddress::TransmitterHolding, *byte).unwrap();
		}

		let expected = String::from_utf8(bytes).unwrap();
		let actual = std::fs::read_to_string(TEST_FILE_PATH).unwrap();
		assert_eq!(expected, actual);

		std::fs::remove_file(TEST_FILE_PATH).unwrap();
	}
}
