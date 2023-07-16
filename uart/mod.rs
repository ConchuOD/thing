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

impl bus::Bus for Uart
{
	fn read<T>(&mut self, address: usize) -> Result<T, bus::Error>
	where
		T: crate::lebytes::LeBytes,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
		todo!()
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

impl Uart
{
	fn read(&self, address: RegisterAddress) -> Result<u8, Error>
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

	fn write(&self, address: RegisterAddress, value: u8) -> Result<(), Error>
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

enum RegisterAddress
{
	ReceiverBuffer,
	TransmitterHolding,
	InterruptEnable,
	InterruptIdent,
	LineControl,
	ModemControl,
	LineStatus,
	ModemStatus,
	Scratch,
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

struct Error;
