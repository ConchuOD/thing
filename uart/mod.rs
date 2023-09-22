use crate::{bus, lebytes::LeBytes};
use std::{
	fmt::Display,
	io::Read,
	io::Write,
	io::{stdin, stdout},
};

const BUFFER_OVERRUN_MASK: u8 = 0b00000010;
const DATA_READY_MASK: u8 = 0b00000001;

#[derive(Debug, PartialEq)]
struct Uart<T: std::io::Read, U: std::io::Write>
{
	registers: UartRegisters,
	input: T,
	output: U,
}
impl<T: std::io::Read, U: std::io::Write> Uart<T, U>
{
	fn new(input: T, output: U) -> Self
	{
		return Self {
			registers: UartRegisters::default(),
			input,
			output,
		};
	}
	fn read_at(&self, address: RegisterAddress) -> Result<u8, Error>
	{
		use RegisterAddress as A;
		return match address {
			A::ReceiverBuffer => Ok(self.registers.receiver_buffer.read()),
			A::TransmitterHolding => Err(Error::DisallowedRead),
			A::InterruptEnable => Ok(self.registers.interrupt_enable.read()),
			A::InterruptIdent => Err(Error::DisallowedRead),
			A::LineControl => Ok(self.registers.line_control.read()),
			A::ModemControl => Ok(self.registers.modem_control.read()),
			A::LineStatus => Ok(self.registers.line_status.read()),
			A::ModemStatus => Ok(self.registers.modem_status.read()),
			A::Scratch => Ok(self.registers.scratch.read()),
		};
	}
	fn write_at(
		&mut self, address: RegisterAddress, value: u8,
	) -> Result<(), Error>
	{
		use RegisterAddress as A;
		return match address {
			A::ReceiverBuffer => Err(Error::DisallowedWrite),
			A::TransmitterHolding => {
				self.registers.transmitter_holding.write(value);
				Ok(())
			},
			A::InterruptEnable => {
				self.registers.interrupt_enable.write(value);
				Ok(())
			},
			A::InterruptIdent => Err(Error::DisallowedWrite),
			A::LineControl => {
				self.registers.line_control.write(value);
				Ok(())
			},
			A::ModemControl => {
				self.registers.modem_control.write(value);
				Ok(())
			},
			A::LineStatus => Err(Error::DisallowedWrite),
			A::ModemStatus => Err(Error::DisallowedWrite),
			A::Scratch => {
				self.registers.scratch.write(value);
				Ok(())
			},
		};
	}

	fn poll(&mut self)
	{
		if self.registers.line_status.bits & 1 == 1 {
			self.registers.line_status.bits |= 2;
		}
		self.registers.line_status.bits |= 1;
	}
}

impl<V: std::io::Read, W: std::io::Write> bus::Bus for Uart<V, W>
{
	fn read<T>(&mut self, address: usize) -> Result<T, bus::Error>
	where
		T: crate::lebytes::LeBytes,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
		if <T as LeBytes>::SIZE > 1 {
			return Err(bus::Error::new(
				bus::ErrorKind::UnsupportedRead,
				"multi-byte reads are not supported by the uart",
			));
		}

		if self.registers.line_status.bits & DATA_READY_MASK != DATA_READY_MASK
		{
			return Err(bus::Error::new(
				bus::ErrorKind::NoData,
				"can not read from uart before data is ready",
			));
		}

		let mut address = RegisterAddress::try_from(address)?;
		if address == RegisterAddress::TransmitterHolding {
			address = RegisterAddress::ReceiverBuffer;
		}
		let mut return_bytes = [0; <T as LeBytes>::SIZE];
		return_bytes[0] = self.read_at(address)?;
		self.registers.line_status.bits = 0;
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
		self.registers.line_status.bits = 1;
		return Ok(());
	}
}

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

#[derive(Debug, PartialEq)]
struct Register
{
	bits: u8,
}

impl Register
{
	fn read(&self) -> u8
	{
		return self.bits;
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
impl Display for RegisterAddress
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		let v: usize = (*self).into();
		return write!(f, "{}", v);
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

#[cfg(test)]
mod test
{
	use std::io::Read;
	use std::io::Write;

	use crate::bus;
	use crate::bus::{Bus, Error, ErrorKind};
	use crate::uart::BUFFER_OVERRUN_MASK;
	use crate::uart::DATA_READY_MASK;

	use super::{RegisterAddress, Uart};

	#[test]
	fn reading_from_address_0_returns_receiver_buffer_register_value()
	{
		let expected = 27u8;
		let mut stdout = MockStdout::default();
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, &mut stdout);
		uart.write(RegisterAddress::ReceiverBuffer, expected).unwrap();

		let actual = uart.read(0).unwrap();

		assert_eq!(expected, actual);
	}

	#[test]
	fn writing_to_address_0_sets_transmitter_holding_register()
	{
		let mut stdout = MockStdout::default();
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, &mut stdout);
		let expected = b'f';

		uart.write(0usize, expected).unwrap();

		let actual =
			uart.read(RegisterAddress::TransmitterHolding.into()).unwrap();
		assert_eq!(expected, actual);
	}

	#[test]
	fn writing_receiver_buffer_register_also_sets_transmitter_holding_register()
	{
		let stdout = MockStdout::default();
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, stdout);
		let expected = b'a';

		uart.write(RegisterAddress::ReceiverBuffer, expected).unwrap();
		let actual = uart
			.read::<u8>(RegisterAddress::TransmitterHolding.into())
			.unwrap();

		assert_eq!(expected, actual);
	}

	#[test]
	fn writing_multiple_bytes_causes_bus_error()
	{
		let stdout = MockStdout::default();
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, stdout);
		let expected = Err(Error::new(
			ErrorKind::Unimplemented,
			"multi-byte writes are not implemented yet",
		));

		let res = uart
			.write(RegisterAddress::TransmitterHolding, 0b00000001_00000001u16);

		assert_eq!(res, expected);
	}

	#[test]
	fn writing_with_file_output_writes_to_file()
	{
		const TEST_FILE_PATH: &str = "test_output";
		let mut file = std::fs::File::create(TEST_FILE_PATH).unwrap();
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, &mut file);
		let bytes: Vec<u8> = "Hello, World!".bytes().collect();

		for byte in &bytes {
			uart.write(RegisterAddress::TransmitterHolding, *byte).unwrap();
		}

		let expected = String::from_utf8(bytes).unwrap();
		let actual = std::fs::read_to_string(TEST_FILE_PATH).unwrap();
		assert_eq!(expected, actual);

		std::fs::remove_file(TEST_FILE_PATH).unwrap();
	}

	#[test]
	fn receiving_data_into_uart_sets_lsr_dr_bit()
	{
		let stdout_buf: Vec<u8> = Vec::new();
		let stdout = MockStdout {
			buf: stdout_buf,
		};
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, stdout);

		// NOTE(js): uart.poll is a helper method that's meant to only be
		// used during development. The poll should really happen when reading
		// from the uart, but reading also resets the bits that are tested
		// against here. As such, to be able to test-drive the behaviour,
		// poll_stdin is directly called in this test.
		uart.poll();

		assert_eq!(uart.registers.line_status.bits & 1u8, 1);
	}

	#[test]
	fn reading_from_uart_resets_lsr_dr_bit()
	{
		let stdout_buf = Vec::new();
		let stdout = MockStdout {
			buf: stdout_buf,
		};
		let stdin = MockStdin::default();
		let mut uart = Uart::new(stdin, stdout);
		uart.registers.line_status.bits = 1;
		uart.read::<u8>(RegisterAddress::ReceiverBuffer.into()).unwrap();
		assert_eq!(uart.registers.line_status.bits & 1u8, 0);
	}

	#[test]
	fn receiving_data_while_data_is_ready_sets_lsr_overrun_bit()
	{
		let mut uart = setup();
		uart.registers.line_status.bits |= DATA_READY_MASK;
		uart.poll();
		assert_eq!(
			uart.registers.line_status.bits & BUFFER_OVERRUN_MASK,
			BUFFER_OVERRUN_MASK
		);
	}

	#[test]
	fn reading_line_status_register_clears_buffer_overrun_bit()
	{
		let mut uart = setup();
		uart.registers.line_status.bits |= DATA_READY_MASK;
		uart.registers.line_status.bits |= BUFFER_OVERRUN_MASK;
		let status_bits =
			uart.read::<u8>(RegisterAddress::LineStatus.into()).unwrap();
		assert_eq!(status_bits & BUFFER_OVERRUN_MASK, BUFFER_OVERRUN_MASK);
		assert_eq!(status_bits & DATA_READY_MASK, DATA_READY_MASK);
		assert!(
			uart.registers.line_status.bits & BUFFER_OVERRUN_MASK
				!= BUFFER_OVERRUN_MASK
		);
	}

	#[test]
	fn reading_more_than_one_byte_causes_unsupported_read_error()
	{
		let input = "some input".as_bytes();
		let output = MockStdout::default();
		let mut uart = Uart::new(input, output);
		let expected_result = Err(bus::Error::new(
			bus::ErrorKind::UnsupportedRead,
			"multi-byte reads are not supported by the uart",
		));

		let acutal_result =
			uart.read::<u16>(RegisterAddress::ReceiverBuffer.into());

		assert_eq!(acutal_result, expected_result);
	}

	#[test]
	fn reading_from_uart_without_data_ready_causes_error()
	{
		let input = "Hello, World".as_bytes();
		let output = MockStdout::default();
		let mut uart = Uart::new(input, output);

		let result = uart.read::<u8>(RegisterAddress::ReceiverBuffer.into());
		let expected = Err(bus::Error::new(
			bus::ErrorKind::NoData,
			"can not read from uart before data is ready",
		));
		assert_eq!(result, expected);
	}

	fn setup() -> Uart<MockStdin, MockStdout>
	{
		let stdout_buf = Vec::new();
		let stdout = MockStdout {
			buf: stdout_buf,
		};
		let stdin = MockStdin::default();
		let uart = Uart::new(stdin, stdout);
		return uart;
	}

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

	struct MockStdin
	{
		out_buf: Vec<u8>,
	}

	impl MockStdin
	{
		fn new(output: &[u8]) -> Self
		{
			return Self {
				out_buf: output.to_vec(),
			};
		}

		fn default() -> Self
		{
			return Self::new(&[1]);
		}
	}

	impl std::io::Read for MockStdin
	{
		fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize>
		{
			buf.write(&[1]);
			return Ok(1);
		}
	}
}
