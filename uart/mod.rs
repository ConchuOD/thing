use crate::bus;

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
	fn r(&self, address: RegisterAddress) -> Result<u8, Error>
	{
		return match address {
			RegisterAddress::ReceiverBuffer => Ok(self.receiver_buffer.read()),
			RegisterAddress::TransmitterHolding => Err(Error),
			RegisterAddress::InterruptEnable => {
				Ok(self.interrupt_enable.read())
			},
			RegisterAddress::InterruptIdent => Err(Error),
			RegisterAddress::LineControl => Ok(self.line_control.read()),
			RegisterAddress::ModemControl => Ok(self.modem_control.read()),
			RegisterAddress::LineStatus => Ok(self.line_status.read()),
			RegisterAddress::ModemStatus => Ok(self.modem_status.read()),
			RegisterAddress::Scratch => Ok(self.scratch.read()),
		};
	}

	fn w(&self, address: RegisterAddress, value: u8) -> Result<(), Error>
	{
		return match address {
			RegisterAddress::ReceiverBuffer => Err(Error),
			RegisterAddress::TransmitterHolding => {
				self.transmitter_holding.write(value);
				Ok(())
			},
			RegisterAddress::InterruptEnable => {
				self.interrupt_enable.write(value);
				Ok(())
			},
			RegisterAddress::InterruptIdent => Err(Error),
			RegisterAddress::LineControl => {
				self.line_control.write(value);
				Ok(())
			},
			RegisterAddress::ModemControl => {
				self.modem_control.write(value);
				Ok(())
			},
			RegisterAddress::LineStatus => Err(Error),
			RegisterAddress::ModemStatus => Err(Error),
			RegisterAddress::Scratch => {
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
		let addr = match address {
			0 => RegisterAddress::ReceiverBuffer,
			1 => RegisterAddress::InterruptEnable,
			2 => RegisterAddress::InterruptIdent,
			3 => RegisterAddress::ModemControl,
			4 => RegisterAddress::LineControl,
			5 => RegisterAddress::ModemStatus,
			6 => RegisterAddress::LineStatus,
			7 => RegisterAddress::Scratch,
			_ => {
				return Err(bus::Error::new(
					bus::ErrorKind::OutOfBounds,
					&format!("invalid read address {}", address),
				))
			},
		};

		return match self.r(addr) {
			Ok(v) => Ok(u8::to_le_bytes(v)),
			Err(e) => {
				todo!("convert the uart error to a bus error and return it")
			},
		};
	}

	fn write<T, U>(&mut self, address: U, value: T) -> Result<(), bus::Error>
	where
		T: crate::lebytes::LeBytes,
		U: Into<usize>,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
		todo!()
	}
}

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

impl From<RegisterAddress> for u8
{
	fn from(val: RegisterAddress) -> Self
	{
		use RegisterAddress::*;
		return match val {
			ReceiverBuffer => 0,
			TransmitterHolding => 0,
			InterruptEnable => 1,
			InterruptIdent => 2,
			LineControl => 3,
			ModemControl => 4,
			LineStatus => 5,
			ModemStatus => 6,
			Scratch => 7,
		};
	}
}

impl From<RegisterAddress> for usize
{
	fn from(value: RegisterAddress) -> usize
	{
		use RegisterAddress::*;
		return match value {
			ReceiverBuffer => 0,
			TransmitterHolding => 0,
			InterruptEnable => 1,
			InterruptIdent => 2,
			LineControl => 3,
			ModemControl => 4,
			LineStatus => 5,
			ModemStatus => 6,
			Scratch => 7,
		};
	}
}

struct Register
{
	bits: u8,
}

impl Register
{
	fn read(&self) -> u8
	{
		todo!();
	}

	fn write(&self, v: u8)
	{
		todo!();
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

struct ReadOnlyRegister
{
	bits: u8,
}

impl ReadOnlyRegister
{
	fn read(&self) -> u8
	{
		todo!();
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

struct WriteOnlyRegister
{
	bits: u8,
}

impl WriteOnlyRegister
{
	fn write(&self, v: u8)
	{
		todo!();
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

struct Error;

#[cfg(test)]
mod test
{
	use crate::bus::Bus;
	use crate::uart::{ReadOnlyRegister, RegisterAddress};

	use super::Uart;

	#[test]
	fn reading_from_address_0_returns_rbr_value()
	{
		let mut uart = Uart {
			receiver_buffer: ReadOnlyRegister {
				bits: 27,
			},
			..Uart::default()
		};
		let expected = 27u8;
		let actual = uart.read(RegisterAddress::ReceiverBuffer.into()).unwrap();

		assert_eq!(expected, actual);
	}
}
