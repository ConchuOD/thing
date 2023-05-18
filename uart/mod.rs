use crate::{
	bus::{self, Bus},
	lebytes::LeBytes,
};

const UART_REGISTER_SIZE: usize = 8;

struct Uart
{
	registers: Vec<u8>,
}

impl Uart
{
	fn new() -> Self
	{
		return Self {
			registers: vec![0u8; UART_REGISTER_SIZE],
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

	fn write<T>(
		&mut self, address: usize, value: T,
	) -> std::result::Result<(), bus::Error>
	where
		T: crate::lebytes::LeBytes,
		[(); <T as crate::lebytes::LeBytes>::SIZE]:,
	{
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
		self.registers.splice(write_range, value.to_le_bytes());
		return Ok(());
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
			fn u8()
			{
				let mut uart = Uart::new();
				uart.write(0, 1u8).unwrap();
				assert_eq!(vec![1, 0, 0, 0, 0, 0, 0, 0], uart.registers);
			}

			#[test]
			fn u64()
			{
				let mut uart = Uart::new();
				uart.write(0, 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001u64).unwrap();
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

				let actual = uart.write(8, 1u8);

				assert_eq!(expected, actual);
			}
		}
	}
}
