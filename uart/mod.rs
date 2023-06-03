use crate::{
	bus::{self, Bus},
	lebytes::LeBytes,
};

const UART_REGISTER_COUNT: usize = 8;

struct Uart
{
	registers: Vec<u8>,
}

impl Uart
{
	fn new() -> Self
	{
		return Self {
			registers: vec![0u8; UART_REGISTER_COUNT],
		};
	}
}

impl Bus for Uart
{
	fn read<T>(&mut self, address: usize) -> Result<T, bus::Error>
	where
		T: LeBytes,
		[(); <T as LeBytes>::SIZE]:,
	{
		let up_to_address = address + <T as LeBytes>::SIZE;
		if up_to_address > self.registers.len() {
			return Err(bus::Error::new(
				bus::ErrorKind::OutOfBounds,
				&format!(
					"uart: memory length: {}, max read address: {}",
					self.registers.len(),
					up_to_address - 1,
				),
			));
		}
		return Ok(T::from_le_bytes(
			self.registers[address..address + <T as LeBytes>::SIZE]
				.try_into()
				.unwrap(),
		));
	}

	fn write<T, U>(
		&mut self, address: U, value: T,
	) -> std::result::Result<(), bus::Error>
	where
		U: Into<usize>,
		T: crate::lebytes::LeBytes,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
		let address = address.into();

		let up_to_address = address + <T as LeBytes>::SIZE;
		if up_to_address > self.registers.len() {
			return Err(bus::Error::new(
				bus::ErrorKind::OutOfBounds,
				&(format!(
					"uart: memory length: {}, max write address: {}",
					self.registers.len(),
					up_to_address - 1,
				)),
			));
		}

		let write_range = address..up_to_address;
		if write_range.contains(&RegisterAddress::RecieverBuffer.into()) {
			self.registers[usize::from(RegisterAddress::LineStatus)] = 1;
		}
		assert_eq!(self.registers, vec![0, 0, 0, 0, 0, 1, 0, 0]);
		self.registers.splice(write_range, value.to_le_bytes());

		return Ok(());
	}
}

enum RegisterAddress
{
	RecieverBuffer,
	LineStatus,
}

impl From<RegisterAddress> for usize
{
	fn from(val: RegisterAddress) -> Self
	{
		return match val {
			RegisterAddress::RecieverBuffer => 0usize,
			RegisterAddress::LineStatus => 5usize,
		};
	}
}

#[cfg(test)]
mod test
{
	mod bus
	{
		mod read
		{
			use crate::{
				bus::{self, Bus},
				uart::Uart,
			};

			#[test]
			fn u8()
			{
				let mut uart = Uart::new();
				uart.registers = vec![1, 2, 3];
				assert_eq!(1, uart.read::<u8>(0).unwrap());
			}

			#[test]
			fn u64()
			{
				let mut uart = Uart::new();
				uart.registers = vec![255, 255, 255, 255, 255, 255, 255, 255];
				assert_eq!(
                    0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111,
                    uart.read::<u64>(0).unwrap()
                );
			}

			#[test]
			fn out_of_bounds_returns_error()
			{
				let mut uart = Uart::new();
				let msg = format!(
					"uart: memory length: {}, max read address: {}",
					8, 8
				);
				let expected =
					Err(bus::Error::new(bus::ErrorKind::OutOfBounds, &msg));

				let actual = uart.read::<u8>(8);

				assert_eq!(
					actual, expected,
					"expected {:?}, but was {:?}",
					expected, actual
				);
			}
		}
		mod write
		{
			use crate::bus::{self, Bus};
			use crate::uart::Uart;

			#[test]
			fn u64()
			{
				let mut uart = Uart::new();
				uart.write(0usize, 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001u64).unwrap();
				assert_eq!(vec![1, 1, 1, 1, 1, 1, 1, 1], uart.registers);
			}

			#[test]
			fn out_of_bounds_returns_error()
			{
				let mut uart = Uart::new();
				let msg = format!(
					"uart: memory length: {}, max write address: {}",
					8, 8
				);
				let expected =
					Err(bus::Error::new(bus::ErrorKind::OutOfBounds, &msg));

				let actual = uart.write(8usize, 1u8);

				assert_eq!(expected, actual);
			}
		}
	}
	mod line_status_register
	{
		use crate::bus::Bus;
		use crate::uart::RegisterAddress;
		use crate::uart::Uart;

		#[test]
		fn writing_data_to_reciever_buffer_register_sets_data_ready_bit()
		{
			let mut uart = Uart::new();
			let expected = vec![1, 0, 0, 0, 0, 1, 0, 0];

			uart.write(RegisterAddress::RecieverBuffer, 1u8).unwrap();

			assert_eq!(expected, uart.registers);
		}
	}
}
