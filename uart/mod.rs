use crate::{
	bus::{self, Bus},
	lebytes::LeBytes,
};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
struct Uart<T: std::io::Write>
{
	registers: UartRegisters,
	output: T,
}
impl<T: std::io::Write> Uart<T>
{
	fn new(output: T) -> Self
	{
		return Self {
			registers: UartRegisters::default(),
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

	fn word_length(&self) -> WordLength
	{
		let character_length_control_bits = self
			.read::<u8>(RegisterAddress::LineControl.into())
			.unwrap() & 0b11;
		return WordLength::try_from(character_length_control_bits).expect("This error can never happen because of the masking in the previous line. The mask ensures 2 bit values. Thus, due to the implementation of TryFrom for WordLength, this can never fail.");
	}

	fn stop_bit_count(&self) -> StopBitCount
	{
		let control_bits =
			self.read::<u8>(RegisterAddress::LineControl.into()).unwrap();

		let stop_bit_control_bit = control_bits & 0b100;
		let x: StopBitMask = stop_bit_control_bit.try_into().unwrap();

		return match x {
			StopBitMask::Unset => StopBitCount::One,
			StopBitMask::Set => {
				let word_length_bits = control_bits & 0b11;
				let word_length = WordLength::try_from(word_length_bits).expect("This error can never happen because of the masking in the previous line. The mask ensures 2 bit values. Thus, due to the implementation of TryFrom for WordLength, this can never fail.");
				StopBitCount::from(word_length)
			},
		};
	}
}

#[derive(Debug, PartialEq)]
enum WordLength
{
	FiveBits,
	SixBits,
	SevenBits,
	EightBits,
}

impl TryFrom<u8> for WordLength
{
	type Error = Error;
	fn try_from(value: u8) -> Result<Self, Self::Error>
	{
		return match value {
			0 => Ok(Self::FiveBits),
			1 => Ok(Self::SixBits),
			2 => Ok(Self::SevenBits),
			3 => Ok(Self::EightBits),
			_ => Err(Error::InvalidCharacterLength),
		};
	}
}

#[derive(Debug, PartialEq)]
enum StopBitCount
{
	One,
	OneAndAHalf,
	Two,
}

impl From<WordLength> for StopBitCount
{
	fn from(word_length: WordLength) -> Self
	{
		return match word_length {
			WordLength::FiveBits => Self::OneAndAHalf,
			WordLength::SixBits => Self::Two,
			WordLength::SevenBits => Self::Two,
			WordLength::EightBits => Self::Two,
		};
	}
}

impl<V: std::io::Write> bus::Bus for Uart<V>
{
	fn read<T>(&self, address: usize) -> Result<T, bus::Error>
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

#[derive(Debug, PartialEq)]
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

impl Default for UartRegisters
{
	fn default() -> Self
	{
		return Self {
			receiver_buffer: ReadOnlyRegister::default(),
			transmitter_holding: WriteOnlyRegister::default(),
			interrupt_enable: Register::default(),
			interrupt_ident: ReadOnlyRegister::default(),
			line_control: Register {
				bits: 0b0000_0011,
			},
			modem_control: Register::default(),
			line_status: Register::default(),
			modem_status: Register::default(),
			scratch: Register::default(),
			divisor_latch_ls: Register::default(),
			divisor_latch_ms: Register::default(),
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

	fn write(&mut self, v: u8)
	{
		self.bits = v;
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
	InvalidCharacterLength,
InvalidStopBitMask,
}
impl From<Error> for bus::Error
{
	fn from(value: Error) -> Self
	{
		match value {
			Error::DisallowedRead => todo!("bus error disallowed read"),
			Error::DisallowedWrite => todo!("bus disallowed write"),
			Error::InvalidCharacterLength => {
				todo!("no corresponding bus error")
			},
			Error::InvalidStopBitMask => {
				todo!("no corresponding bus error")
			},
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

enum WordLengthMask
{
	FiveBit = 0b0000_0000,
	SixBit = 0b0000_0001,
	SevenBit = 0b0000_0010,
	EightBit = 0b0000_0011,
}

enum StopBitMask
{
	Unset = 0b0000_0000,
	Set = 0b0000_0100,
}

impl TryFrom<u8> for StopBitMask {
	type Error = Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		return match value {
			0b0000_0000 => Ok(StopBitMask::Unset),
			0b0000_0100 => Ok(StopBitMask::Set),
			_ => Err(Error::InvalidStopBitMask),
		};
	}
}

#[cfg(test)]
mod test
{
	use crate::{
		bus::{Bus, Error, ErrorKind},
		uart::{StopBitCount, WordLength, WordLengthMask, StopBitMask},
	};

	use super::{RegisterAddress, Uart};

	#[test]
	fn reading_from_address_0_returns_receiver_buffer_register_value()
	{
		let expected = 27u8;
		let mut stdout = MockStdout::default();
		let mut uart = Uart::new(&mut stdout);
		uart.write(RegisterAddress::ReceiverBuffer, expected).unwrap();

		let actual = uart.read(0).unwrap();

		assert_eq!(expected, actual);
	}

	#[test]
	fn writing_to_address_0_sets_transmitter_holding_register()
	{
		let mut mock_stdout = MockStdout::default();
		let mut uart = Uart::new(&mut mock_stdout);
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
		let mut uart = Uart::<MockStdout>::new(stdout);
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
		let mut uart = Uart::<MockStdout>::new(stdout);
		let expected = Err(Error::new(
			ErrorKind::Unimplemented,
			"multi-byte writes are not implemented yet",
		));

		let res = uart
			.write(RegisterAddress::TransmitterHolding, 0b00000001_00000001u16);

		assert_eq!(res, expected);
	}

	#[test]
	fn reading_multiple_bytes_causes_bus_error()
	{
		let stdout = MockStdout {
			buf: Vec::new(),
		};
		let uart = Uart::<MockStdout>::new(stdout);
		let expected = Err(Error::new(
			ErrorKind::Unimplemented,
			"multi-byte reads are not implemented yet",
		));

		let res = uart.read::<u16>(RegisterAddress::TransmitterHolding.into());

		assert_eq!(res, expected);
	}

	#[test]
	fn writing_with_file_output_writes_to_file()
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

	#[test]
	fn default_word_length_is_8_bits()
	{
		let stdout = MockStdout::default();
		let uart = Uart::<MockStdout>::new(stdout);
		let character_length = uart.word_length();
		assert_eq!(character_length, WordLength::EightBits);
	}

	#[test]
	fn writing_to_lcr_lower_2_bits_sets_word_length_correctly() {
		let stdout = MockStdout::default();
		let mut uart = Uart::<MockStdout>::new(stdout);

		uart.write(RegisterAddress::LineControl, WordLengthMask::FiveBit as u8).unwrap();
		assert_eq!(uart.word_length(), WordLength::FiveBits);

		uart.write(RegisterAddress::LineControl, WordLengthMask::SixBit as u8).unwrap();
		assert_eq!(uart.word_length(), WordLength::SixBits);

		uart.write(RegisterAddress::LineControl, WordLengthMask::SevenBit as u8).unwrap();
		assert_eq!(uart.word_length(), WordLength::SevenBits);

		uart.write(RegisterAddress::LineControl, WordLength::EightBits as u8).unwrap();
		assert_eq!(uart.word_length(), WordLength::EightBits);
	}

	#[test]
	fn default_stop_bit_count_is_1()
	{
		let stdout = MockStdout::default();
		let uart = Uart::<MockStdout>::new(stdout);

		assert_eq!(uart.stop_bit_count(), StopBitCount::One);
	}

	#[test]
	fn stop_bit_count_is_one_when_stop_bit_count_bit_is_unset() {
		let stdout = MockStdout::default();
		let mut uart = Uart::<MockStdout>::new(stdout);
		let mask = StopBitMask::Unset as u8;
		uart.write(RegisterAddress::LineControl, mask).unwrap();

		let stop_bit_count = uart.stop_bit_count();

		assert_eq!(stop_bit_count, StopBitCount::One);
	}

	#[test]
	fn stop_bit_count_is_one_and_a_half_when_stop_bit_count_bit_is_set_and_word_length_set_to_five(
	)
	{

		let stdout = MockStdout::default();
		let mut uart = Uart::<MockStdout>::new(stdout);
		let mask = StopBitMask::Set as u8 | WordLengthMask::FiveBit as u8;	
		uart.write(RegisterAddress::LineControl, mask).unwrap();

		let stop_bit_count = uart.stop_bit_count();

		assert_eq!(stop_bit_count, StopBitCount::OneAndAHalf);
	}

	#[test]
	fn stop_bit_count_is_two_when_stop_bit_count_is_set_and_word_length_is_six() {
		let stdout = MockStdout::default();
		let mut uart = Uart::<MockStdout>::new(stdout);
		let mask = StopBitMask::Set as u8 | WordLengthMask::SixBit as u8;
		uart.write(RegisterAddress::LineControl, mask).unwrap();

		let stop_bit_count = uart.stop_bit_count();

		assert_eq!(stop_bit_count, StopBitCount::Two);
	}

	#[test]
	fn stop_bit_count_is_two_when_stop_bit_count_is_set_and_word_length_is_seven() {
		let stdout = MockStdout::default();
		let mut uart = Uart::<MockStdout>::new(stdout);
		let mask = StopBitMask::Set as u8 | WordLengthMask::SevenBit as u8;
		uart.write(RegisterAddress::LineControl, mask).unwrap();

		let stop_bit_count = uart.stop_bit_count();

		assert_eq!(stop_bit_count, StopBitCount::Two);
	}

	#[test]
	fn stop_bit_count_is_two_when_stop_bit_count_is_set_and_word_length_is_eight() {
		let stdout = MockStdout::default();
		let mut uart = Uart::<MockStdout>::new(stdout);
		let mask = StopBitMask::Set as u8 | WordLengthMask::EightBit as u8;
		uart.write(RegisterAddress::LineControl, mask).unwrap();

		let stop_bit_count = uart.stop_bit_count();

		assert_eq!(stop_bit_count, StopBitCount::Two);
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
}
