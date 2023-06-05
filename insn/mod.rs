// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::bus::Bus;
use crate::field_get;
use crate::gen_mask;
use crate::platform::Platform;
use crate::sign_extend;
use debug_print::debug_println;

use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, PartialEq)]
pub enum InsnType
{
	Invalid,
	R,
	I,
	S,
	B,
	U,
	J,
}

#[derive(Debug)]
pub struct Insn
{
	pub name: String,
	pub opcode: u32,
	pub rd: u32,
	pub rs1: u32,
	pub rs2: u32,
	pub imm: i32,
	pub func3: u32,
	pub func7: u32,
	pub insn_type: InsnType,
}

macro_rules! insn_mask {
	($yo:ident) => {{
		let start = concat_idents!($yo, _SHIFT);
		let width = concat_idents!($yo, _WIDTH);
		gen_mask!(start + width - 1, start, u32)
	}};
}

const OPCODE_LOAD: u32 = 0b000_0011;
const OPCODE_MISCMEM: u32 = 0b000_1111;
const OPCODE_INT_REG_IMM: u32 = 0b0001_0011;
const OPCODE_AUIPC: u32 = 0b001_0111;
const OPCODE_INT_REG_IMM_32: u32 = 0b001_1011;
const OPCODE_STORE: u32 = 0b010_0011;
const OPCODE_ATOMIC: u32 = 0b010_1111;
const OPCODE_INT_REG_REG: u32 = 0b011_0011;
const OPCODE_LUI: u32 = 0b011_0111;
const OPCODE_INT_REG_REG_32: u32 = 0b011_1011;
const OPCODE_BRANCH: u32 = 0b110_0011;
const OPCODE_JALR: u32 = 0b110_0111;
const OPCODE_JAL: u32 = 0b110_1111;
const OPCODE_SYSTEM: u32 = 0b111_0011;
const OPCODE_MASK: u32 = 0b111_1111;

const RD_SHIFT: u32 = 7;
const RD_WIDTH: u32 = 5;
const RD_MASK: u32 = insn_mask!(RD);

const RS1_SHIFT: u32 = 15;
const RS1_WIDTH: u32 = 5;
const RS1_MASK: u32 = insn_mask!(RS1);

const RS2_SHIFT: u32 = 20;
const RS2_WIDTH: u32 = 5;
const RS2_MASK: u32 = insn_mask!(RS2);

const IMM_UTYPE_SHIFT: u32 = 12;
const IMM_UTYPE_WIDTH: u32 = 20;
const IMM_UTYPE_MASK: u32 = insn_mask!(IMM_UTYPE);

const IMM_ITYPE_SHIFT: u32 = 20;
const IMM_ITYPE_WIDTH: u32 = 12;
const IMM_ITYPE_MASK: u32 = insn_mask!(IMM_ITYPE);

const IMM4_0_STYPE_SHIFT: u32 = RD_SHIFT;
const IMM4_0_STYPE_WIDTH: u32 = RD_WIDTH;
const IMM4_0_STYPE_MASK: u32 = insn_mask!(IMM4_0_STYPE);

const IMM11_5_STYPE_SHIFT: u32 = 25;
const IMM11_5_STYPE_WIDTH: u32 = 7;
const IMM11_5_STYPE_MASK: u32 = insn_mask!(IMM11_5_STYPE);

const IMM10_1_JTYPE_SHIFT: u32 = 21;
const IMM10_1_JTYPE_WIDTH: u32 = 10;
const IMM10_1_JTYPE_MASK: u32 = insn_mask!(IMM10_1_JTYPE);

const IMM11_JTYPE_SHIFT: u32 = 20;
const IMM11_JTYPE_WIDTH: u32 = 1;
const IMM11_JTYPE_MASK: u32 = insn_mask!(IMM11_JTYPE);

const IMM19_12_JTYPE_SHIFT: u32 = 12;
const IMM19_12_JTYPE_WIDTH: u32 = 8;
const IMM19_12_JTYPE_MASK: u32 = insn_mask!(IMM19_12_JTYPE);

const IMM20_JTYPE_SHIFT: u32 = 31;
const IMM20_JTYPE_WIDTH: u32 = 1;
const IMM20_JTYPE_MASK: u32 = insn_mask!(IMM20_JTYPE);

const IMM4_1_BTYPE_SHIFT: u32 = 8;
const IMM4_1_BTYPE_WIDTH: u32 = 4;
const IMM4_1_BTYPE_MASK: u32 = insn_mask!(IMM4_1_BTYPE);
const IMM11_BTYPE_SHIFT: u32 = 7;
const IMM11_BTYPE_WIDTH: u32 = 1;
const IMM11_BTYPE_MASK: u32 = insn_mask!(IMM11_BTYPE);
const IMM10_5_BTYPE_SHIFT: u32 = 25;
const IMM10_5_BTYPE_WIDTH: u32 = 6;
const IMM10_5_BTYPE_MASK: u32 = insn_mask!(IMM10_5_BTYPE);
const IMM12_BTYPE_SHIFT: u32 = 31;
const IMM12_BTYPE_WIDTH: u32 = 1;
const IMM12_BTYPE_MASK: u32 = insn_mask!(IMM12_BTYPE);

const FUNC3_SHIFT: u32 = 12;
const FUNC3_WIDTH: u32 = 3;
const FUNC3_MASK: u32 = insn_mask!(FUNC3);

// this should be an enum, right? (or not, there's dupes!)
const FUNC3_ADDI: u32 = 0b000;
const FUNC3_SLTI: u32 = 0b010;
const FUNC3_SLTIU: u32 = 0b011;
const FUNC3_XORI: u32 = 0b100;
const FUNC3_ORI: u32 = 0b110;
const FUNC3_ANDI: u32 = 0b111;

const FUNC3_SLLI: u32 = 0b001;
const FUNC3_SRLI: u32 = 0b101;
const FUNC3_SRAI: u32 = 0b101;
const FUNC3_ADD: u32 = 0b000;
const FUNC3_SUB: u32 = 0b000;
const FUNC3_SLL: u32 = 0b001;
const FUNC3_SLT: u32 = 0b010;
const FUNC3_SLTU: u32 = 0b011;
const FUNC3_XOR: u32 = 0b100;
const FUNC3_SRL: u32 = 0b101;
const FUNC3_SRA: u32 = 0b101;
const FUNC3_OR: u32 = 0b110;
const FUNC3_AND: u32 = 0b111;
const FUNC3_ADDIW: u32 = 0b000;
const FUNC3_SLLIW: u32 = 0b001;
const FUNC3_SRLIW: u32 = 0b101;
const FUNC3_SRAIW: u32 = 0b101;
const FUNC3_ADDW: u32 = 0b000;
const FUNC3_SUBW: u32 = 0b000;
const FUNC3_SLLW: u32 = 0b001;
const FUNC3_SRLW: u32 = 0b101;
const FUNC3_SRAW: u32 = 0b101;

const FUNC3_SB: u32 = 0b000;
const FUNC3_SH: u32 = 0b001;
const FUNC3_SW: u32 = 0b010;
const FUNC3_SD: u32 = 0b011;

const FUNC3_LB: u32 = 0b000;
const FUNC3_LH: u32 = 0b001;
const FUNC3_LW: u32 = 0b010;
const FUNC3_LD: u32 = 0b011;
const FUNC3_LBU: u32 = 0b100;
const FUNC3_LHU: u32 = 0b101;
const FUNC3_LWU: u32 = 0b110;

const FUNC3_CSRRW: u32 = 0b001;
const FUNC3_CSRRS: u32 = 0b010;
const FUNC3_CSRRC: u32 = 0b011;
const FUNC3_CSRRWI: u32 = 0b101;
const FUNC3_CSRRSI: u32 = 0b110;
const FUNC3_CSRRCI: u32 = 0b111;

const FUNC3_BEQ: u32 = 0b000;
const FUNC3_BNE: u32 = 0b001;
const FUNC3_BLT: u32 = 0b100;
const FUNC3_BGE: u32 = 0b101;
const FUNC3_BLTU: u32 = 0b110;
const FUNC3_BGEU: u32 = 0b111;

const FUNC3_RV32_ATOMIC: u32 = 0b010;
const FUNC3_RV64_ATOMIC: u32 = 0b011;

const FUNC7_SHIFT: u32 = IMM11_5_STYPE_SHIFT;
const FUNC7_WIDTH: u32 = IMM11_5_STYPE_WIDTH;
const FUNC7_MASK: u32 = IMM11_5_STYPE_MASK;

const FUNC7_SLLI: u32 = 0b0000000;
const FUNC7_SRLI: u32 = 0b0000000;
const FUNC7_SRAI: u32 = 0b0100000;
const FUNC7_ADD: u32 = 0b0000000;
const FUNC7_SUB: u32 = 0b0100000;
const FUNC7_SLL: u32 = 0b0000000;
const FUNC7_SLT: u32 = 0b0000000;
const FUNC7_SLTU: u32 = 0b0000000;
const FUNC7_XOR: u32 = 0b0000000;
const FUNC7_SRL: u32 = 0b0000000;
const FUNC7_SRA: u32 = 0b0100000;
const FUNC7_OR: u32 = 0b0000000;
const FUNC7_AND: u32 = 0b0000000;

const FUNC7_LR: u32 = 0b0001000;
const FUNC7_SC: u32 = 0b0001100;
const FUNC7_AMOSWAP: u32 = 0b0000100;
const FUNC7_AMOADD: u32 = 0b0000000;
const FUNC7_AMOXOR: u32 = 0b0010000;
const FUNC7_AMOAND: u32 = 0b0110000;
const FUNC7_AMOOR: u32 = 0b0100000;
const FUNC7_AMOMIN: u32 = 0b1000000;
const FUNC7_AMOMAX: u32 = 0b1010000;
const FUNC7_AMOMINU: u32 = 0b1100000;
const FUNC7_AMOMAXU: u32 = 0b1110000;

impl Default for Insn
{
	fn default() -> Insn
	{
		return Insn {
			name: String::from("tba"),
			opcode: 0x0,
			rd: 0x0,
			rs1: 0x0,
			rs2: 0x0,
			imm: 0x0,
			func3: 0x0,
			func7: 0x0,
			insn_type: InsnType::Invalid,
		};
	}
}

impl Insn
{
	fn parse(&mut self, input: u32)
	{
		self.opcode = input & OPCODE_MASK;

		match self.opcode {
			OPCODE_LUI | OPCODE_AUIPC => {
				self.insn_type = InsnType::U;
			},

			OPCODE_JAL => {
				self.insn_type = InsnType::J;
			},

			OPCODE_JALR => {
				self.insn_type = InsnType::I;
			},

			OPCODE_INT_REG_IMM => {
				self.insn_type = InsnType::I;
			},

			OPCODE_INT_REG_REG => {
				self.insn_type = InsnType::R;
			},

			OPCODE_LOAD => {
				self.insn_type = InsnType::S;
			},

			OPCODE_STORE => {
				self.insn_type = InsnType::S;
			},

			OPCODE_SYSTEM => {
				self.insn_type = InsnType::I;
			},

			OPCODE_MISCMEM => {
				self.insn_type = InsnType::I;
			},

			OPCODE_BRANCH => {
				self.insn_type = InsnType::B;
			},

			OPCODE_ATOMIC => {
				self.insn_type = InsnType::R;
			},

			OPCODE_INT_REG_IMM_32 => {
				let func3 = field_get!(input, FUNC3, u32);
				if func3 == 0 {
					self.insn_type = InsnType::I;
				} else {
					self.insn_type = InsnType::R;
				}
			},

			OPCODE_INT_REG_REG_32 => {
				todo!("reg reg 32 .insn 0x{:x}", input);
			},

			_ => {
				todo!("opcode 0b{:b} .insn 0x{:x}", self.opcode, input);
			},
		}

		match self.insn_type {
			InsnType::U => {
				self.imm = (input & IMM_UTYPE_MASK) as i32;
				self.rd = field_get!(input, RD, u32);

				self.imm = sign_extend!(self.imm, 31, i32);
			},

			InsnType::I => {
				self.imm = field_get!(input, IMM_ITYPE, i32);
				self.rd = field_get!(input, RD, u32);
				self.rs1 = field_get!(input, RS1, u32);
				self.func3 = field_get!(input, FUNC3, u32);

				self.imm = sign_extend!(self.imm, 11, i32);
			},

			InsnType::R => {
				self.rd = field_get!(input, RD, u32);
				self.rs1 = field_get!(input, RS1, u32);
				self.rs2 = field_get!(input, RS2, u32);
				self.func3 = field_get!(input, FUNC3, u32);
				self.func7 = field_get!(input, FUNC7, u32);
			},

			InsnType::S => {
				self.rs1 = field_get!(input, RS1, u32);
				self.rs2 = field_get!(input, RS2, u32);
				self.func3 = field_get!(input, FUNC3, u32);

				let imm4_0 = field_get!(input, IMM4_0_STYPE, u32);
				let imm11_5 = field_get!(input, IMM11_5_STYPE, u32);

				self.imm = ((imm11_5 << IMM4_0_STYPE_WIDTH) | imm4_0) as i32;
				self.imm = sign_extend!(self.imm, 11, i32);
			},

			InsnType::B => {
				self.rs1 = field_get!(input, RS1, u32);
				self.rs2 = field_get!(input, RS2, u32);
				self.func3 = field_get!(input, FUNC3, u32);

				let imm_4_1 = field_get!(input, IMM4_1_BTYPE, u32);
				let imm_10_5 = field_get!(input, IMM10_5_BTYPE, u32);
				let imm_11 = field_get!(input, IMM11_BTYPE, u32);
				let imm_12 = field_get!(input, IMM12_BTYPE, u32);

				self.imm |= (imm_4_1 << 1) as i32;
				self.imm |= (imm_10_5 << 5) as i32;
				self.imm |= (imm_11 << 11) as i32;
				self.imm |= (imm_12 << 12) as i32;
				self.imm = sign_extend!(self.imm, 12, i32);
			},

			InsnType::J => {
				self.rd = field_get!(input, RD, u32);

				let imm_10_1 = field_get!(input, IMM10_1_JTYPE, u32);
				let imm_11 = field_get!(input, IMM11_JTYPE, u32);
				let imm_19_12 = field_get!(input, IMM19_12_JTYPE, u32);
				let imm_20 = field_get!(input, IMM20_JTYPE, u32);

				self.imm |= (imm_10_1 << 1) as i32;
				self.imm |= (imm_11 << 11) as i32;
				self.imm |= (imm_19_12 << 12) as i32;
				self.imm |= (imm_20 << 20) as i32;
				self.imm = sign_extend!(self.imm, 20, i32);
			},

			_ => (),
		}
	}

	fn handle_int_reg_reg_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;

		let rs1: u64 = hart.read_register(self.rs1 as usize);
		let rs2: u64 = hart.read_register(self.rs2 as usize);

		match self.func3 {
			FUNC3_ADD => {
				if self.func7 == FUNC7_ADD {
					self.name = String::from("add");
					// ADD adds the value in rs1 to rs2 and stores
					// the result in rd
					// overflows are ignored, the lower XLEN bits
					// get written
					let tmp: u64 = rs1.wrapping_add(rs2);
					hart.write_register(self.rd as usize, tmp);
				} else {
					self.name = String::from("sub");
					// SUB subtracts the value in rs2 from rs1
					// and stores the result in rd
					// overflows are ignored, the lower XLEN bits
					// get written
					let tmp: u64 = rs1.wrapping_sub(rs2);
					hart.write_register(self.rd as usize, tmp);
				}
			},

			_ => todo!("reg reg: {:}", self.func3),
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_int_reg_reg32_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;

		let rs1: u64 = hart.read_register(self.rs1 as usize);
		let rs2: u64 = hart.read_register(self.rs2 as usize);

		// shifts encode the "shamt" in the bottom 6 bits of the imm
		// field. It's the bottom 5 for rv32, but the 5th bit is always
		// defined as 0 there.
		let shift: u32 = (rs2 & gen_mask!(5, 0, u64)) as u32;
	}

	fn handle_int_reg_imm_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;

		// All of these functions take the sign-extended 12-bit
		// immediate, and use it perform some calculation register rs1.
		// Arithmetic overflow is ignored and the result is simply the
		// low XLEN bits of the result.
		let mut src: u64 = hart.read_register(self.rs1 as usize);
		let imm: i64 = self.imm as i64;

		// shifts encode the "shamt" in the bottom 6 bits of the imm
		// field. It's the bottom 5 for rv32, but the 5th bit is always
		// defined as 0 there.
		let shift: u32 = ((imm as u64) & gen_mask!(5, 0, u64)) as u32;

		match self.func3 {
			FUNC3_ADDI => {
				if self.imm == 0 && self.rs1 == 0 && self.rd == 0 {
					self.name = String::from("nop");
					return;
				} else if self.imm == 0 {
					self.name = String::from("mv");
				} else {
					self.name = String::from("addi");
				}

				src = src.wrapping_add_signed(imm);
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_ANDI => {
				self.name = String::from("andi");
				src &= imm as u64;
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_ORI => {
				self.name = String::from("ori");
				src |= imm as u64;
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_XORI => {
				self.name = String::from("xori");
				src ^= imm as u64;
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_SLTI => {
				self.name = String::from("slti");
				let tmp: i64 = src as i64;

				if tmp < imm {
					hart.write_register(self.rd as usize, 1);
				} else {
					hart.write_register(self.rd as usize, 0);
				}
			},

			FUNC3_SLTIU => {
				self.name = String::from("sltiu");

				if src < (imm as u64) {
					hart.write_register(self.rd as usize, 1);
				} else {
					hart.write_register(self.rd as usize, 0);
				}
			},

			FUNC3_SLLI => {
				self.name = String::from("slli");
				src = src.wrapping_shl(shift);
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_SRLI => {
				// if bit 10 is set, shift the sign bit down
				let is_srai = (imm as u64) & gen_mask!(10, 10, u64);
				if is_srai != 0 {
					self.name = String::from("srli");
					src = src.wrapping_shr(shift);
				} else {
					self.name = String::from("srai");
					src = (src as i64).wrapping_shr(shift) as u64;
				}

				hart.write_register(self.rd as usize, src);
			},

			_ => todo!("reg imm: {:}", self.func3),
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_int_reg_imm32_insn(
		&mut self, platform: &Arc<RwLock<&mut Platform>>,
	)
	{
		let hart = &mut (platform.write().unwrap()).hart;
		let mut src: u64 = hart.read_register(self.rs1 as usize);
		let imm: i64 = self.imm as i64;

		// shifts encode the "shamt" in the bottom 6 bits of the imm
		// field. It's the bottom 5 for rv32, but the 5th bit is always
		// defined as 0 there.
		let shift: u32 = ((imm as u64) & gen_mask!(5, 0, u64)) as u32;

		match self.func3 {
			FUNC3_ADDIW => {
				if self.imm == 0 {
					self.name = String::from("sextw");
				} else {
					self.name = String::from("addiw");
				}

				// ADDIW adds the sign-extended 12-bit immediate
				// to register rs1 & stores the sign extension
				// of a 32-bit result in rd. Arithmetic overflow
				// is ignored and the result is simply the low
				// 32-bit of the result sign extended to 64-bits
				src = src.wrapping_add_signed(imm);
				src &= gen_mask!(31, 0, u64);
				src = src as i32 as i64 as u64;
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_SLLIW => {
				// like slli, but with 32-bit values/results
				// TODO: verify that "32-bit signed result"
				// does not mean that it should be sign extended
				// out to 64-bits
				self.name = String::from("slliw");
				let tmp_src = (src & gen_mask!(31, 0, u64)) as u32;
				src = tmp_src.wrapping_shl(shift) as u64;
				hart.write_register(self.rd as usize, src);
			},

			FUNC3_SRLIW => {
				let is_sraiw = (imm as u64) & gen_mask!(10, 10, u64);
				// like srli, but with 32-bit values/results
				// TODO: verify that "32-bit signed result"
				// does not mean that it should be sign extended
				// out to 64-bits
				let tmp_src = (src & gen_mask!(31, 0, u64)) as u32;
				if is_sraiw == 0 {
					self.name = String::from("srliw");
					src = tmp_src.wrapping_shr(shift) as u64;
				} else {
					self.name = String::from("sraiw");
					src = (tmp_src as i32).wrapping_shr(shift) as u32 as u64;
				}

				hart.write_register(self.rd as usize, src);
			},
			_ => todo!("reg imm32: {:}", self.func3),
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_store_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		// These are all store instructions of varied widths
		// Stores add a sign-extended 12-bit immediate to rs1, forming
		// a memory address. The value in rs2 is put at this memory
		// address.
		//

		let platform_read = platform.read().unwrap();
		let offset: i64 = self.imm.try_into().unwrap();
		let hart = &platform_read.hart;
		let base: u64 = hart.read_register(self.rs1 as usize);
		let address: u64 = base.wrapping_add_signed(offset);
		let mut tmp: u64 = hart.read_register(self.rs2 as usize);
		drop(platform_read);
		let platform_write = &mut platform.write().unwrap();
		let hart_id = platform_write.hart.id;

		match self.func3 {
			FUNC3_SD => {
				self.name = String::from("sd");
				let _ = platform_write.write_from_hart(
					hart_id,
					address as usize,
					tmp,
				);
			},

			FUNC3_SW => {
				self.name = String::from("sw");
				tmp &= gen_mask!(31, 0, u64);
				let _ = platform_write.write_from_hart(
					hart_id,
					address as usize,
					tmp as u32,
				);
			},

			FUNC3_SH => {
				self.name = String::from("sh");
				tmp &= gen_mask!(15, 0, u64);
				let _ = platform_write.write_from_hart(
					hart_id,
					address as usize,
					tmp as u16,
				);
			},

			FUNC3_SB => {
				self.name = String::from("sb");
				tmp &= gen_mask!(7, 0, u64);
				let _ = platform_write.write_from_hart(
					hart_id,
					address as usize,
					tmp as u8,
				);
			},

			_ => todo!("store: {:}", self.func3),
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_load_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		// These are all load instructions of varied widths.
		// Loads add a sign-extended 12-bit immediate to rs1, forming
		// a memory address. The value at this memory address is put in
		// the register in rd.
		let platform_read = platform.read().unwrap();
		let offset: i64 = self.imm.try_into().unwrap();
		let hart = &platform_read.hart;
		let base: u64 = hart.read_register(self.rs1 as usize);
		let address: u64 = base.wrapping_add_signed(offset);
		drop(platform_read);
		let platform_bus = &mut platform.write().unwrap();

		match self.func3 {
			FUNC3_LD => {
				self.name = String::from("ld");
				let tmp: u64 = platform_bus.read(address as usize).unwrap();
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, tmp);
			},

			FUNC3_LW => {
				self.name = String::from("lw");
				let tmp: u32 = platform_bus.read(address as usize).unwrap();
				let extended: u64 = tmp as i32 as i64 as u64;
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, extended);
			},

			FUNC3_LH => {
				self.name = String::from("lh");
				let tmp: u16 = platform_bus.read(address as usize).unwrap();
				let extended: u64 = tmp as i16 as i32 as u64;
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, extended);
			},

			FUNC3_LB => {
				self.name = String::from("lb");
				let tmp: u8 = platform_bus.read(address as usize).unwrap();
				let extended: u64 = tmp as i8 as i64 as u64;
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, extended);
			},

			FUNC3_LWU => {
				self.name = String::from("lwu");
				let tmp: u32 = platform_bus.read(address as usize).unwrap();
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, tmp as u64);
			},

			FUNC3_LHU => {
				self.name = String::from("lhu");
				let tmp: u16 = platform_bus.read(address as usize).unwrap();
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, tmp as u64);
			},

			FUNC3_LBU => {
				self.name = String::from("lbu");
				let tmp: u8 = platform_bus.read(address as usize).unwrap();
				let hart = &mut (platform_bus).hart;
				hart.write_register(self.rd as usize, tmp as u64);
			},

			_ => {
				todo!("load: {:}", self.func3);
			},
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_csr_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;

		// The "funky" thing to look out for with these CSR things,
		// is that they are I-type instructions, so use the "imm"
		// field to store a csr number. Those that use immediates
		// specifically use unsigned ones & those appear in the
		// rs1 field of a regular I-type.
		let imm: usize = (self.imm as usize) & gen_mask!(11, 0, usize);
		match self.func3 {
			FUNC3_CSRRW => {
				// Quoting the spec:
				// CSRRW reads the old value of the CSR,
				// zero-extends the value to XLEN bits,
				// then writes it to integer register rd.
				// The initial value in rs1 is written to
				// the CSR. If rd=x0, then the instruction
				// shall not read the CSR and shall not cause
				// any of the side effects that might occur on
				// a CSR read.
				self.name = String::from("csrww");
				let to_write: u64 = hart.read_register(self.rs1 as usize);
				if self.rd != 0 {
					let csr_old: u64 = hart.read_csr(imm);
					hart.write_register(self.rd as usize, csr_old);
				}
				hart.write_csr(self.rd as usize, to_write);
			},

			FUNC3_CSRRWI => {
				// Like CSRRW, but uses an intermediate from
				// rs1 instead of reading from a register,
				// limiting it to the lower 5 bits.
				self.name = String::from("csrrwi");
				let to_write: u64 = self.rs1 as u64;
				if self.rd != 0 {
					let csr_old: u64 = hart.read_csr(imm);
					hart.write_register(self.rd as usize, csr_old);
				}
				hart.write_csr(self.rd as usize, to_write);
			},

			FUNC3_CSRRS => {
				// Quoting the spec:
				// CSRRS reads the value of the CSR, zero
				// extends the value to XLEN bits, and writes
				// it to integer register rd. The initial value
				// in integer register rs1 is treated as a bit
				// mask that specifies bit positions to be set
				// in the CSR. Any bit that is high in rs1 will
				// cause the corresponding bit to be set in the
				// CSR, if that CSR bit is writeable.
				self.name = String::from("csrws");
				let csr_val: u64 = hart.read_csr(imm);
				if self.rs1 != 0 {
					let mask = hart.read_register(self.rs1 as usize);
					hart.write_csr(imm, csr_val | mask);
				}
				hart.write_register(self.rd as usize, csr_val);
			},

			FUNC3_CSRRSI => {
				// Like CSRRS, but uses an intermediate from
				// rs1 instead of reading from a register,
				// limiting it to the lower 5 bits.
				self.name = String::from("csrrsi");
				let mask: u64 = self.rs1 as u64;
				let csr_val: u64 = hart.read_csr(imm);
				if mask != 0 {
					hart.write_csr(imm, csr_val | mask);
				}
				hart.write_register(self.rd as usize, csr_val);
			},

			FUNC3_CSRRC => {
				// Quoting the spec:
				// CSRRC instruction reads the value of the CSR
				// zero extends the value to XLEN bits, and
				// writes it to integer register rd.
				// The initial value in integer register rs1 is
				// treated as a bit mask that specifies bit
				// positions to be cleared in the CSR. Any bit
				// that is high in rs1 will cause the
				// corresponding bit to be cleared in the CSR,
				// if that CSR bit is writeable.
				// Other bits in the CSR are unaffected.
				self.name = String::from("csrrc");
				let csr_val: u64 = hart.read_csr(imm);
				let mask = !hart.read_register(self.rs1 as usize);
				hart.write_csr(imm, csr_val & mask);
				hart.write_register(self.rd as usize, csr_val);
			},

			FUNC3_CSRRCI => {
				// Like CSRRC, but uses an intermediate from
				// rs1 instead of reading from a register,
				// limiting it to the lower 5 bits.
				self.name = String::from("csrrci");
				let csr_val: u64 = hart.read_csr(imm);
				let mask: u64 = !(self.rs1 as u64);
				if mask != u64::MAX {
					hart.write_csr(imm, csr_val & mask);
				}
				hart.write_register(self.rd as usize, csr_val);
			},

			_ => todo!("csr: {:}", self.func3),
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_jump_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;

		match self.opcode {
			OPCODE_JAL => {
				self.name = String::from("jal");
				let tmp: i64 = self.imm as i64;
				let target: u64 = hart.pc.wrapping_add_signed(tmp);

				debug_println!(
					"Jumping to {:x} (imm: {:x}) from {:x}",
					target,
					tmp,
					hart.pc
				);

				hart.write_register(self.rd as usize, hart.pc + 4);
				hart.pc = target;
			},

			OPCODE_JALR => {
				self.name = String::from("jalr");
				let tmp: i64 = self.imm as i64;
				let base: u64 = hart.read_register(self.rs1 as usize);
				let mut target: u64 = base.wrapping_add_signed(tmp);
				target &= gen_mask!(63, 1, u64);

				debug_println!(
					"Jumping to {:x} (base: {:x} imm: {:x}) from {:x}",
					target,
					base,
					tmp,
					hart.pc
				);

				hart.write_register(self.rd as usize, hart.pc + 4);
				hart.pc = target;
			},

			_ => todo!("jump"),
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_branch_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;
		let src1: u64 = hart.read_register(self.rs1 as usize);
		let src2: u64 = hart.read_register(self.rs2 as usize);
		let mut offset: i32 = 0;

		match self.func3 {
			FUNC3_BEQ => {
				self.name = String::from("beq");
				if src1 == src2 {
					offset = self.imm;
				}
			},

			FUNC3_BNE => {
				self.name = String::from("beq");
				if src1 != src2 {
					offset = self.imm;
				}
			},

			FUNC3_BLT => {
				self.name = String::from("blt");
				if (src1 as i64) < (src2 as i64) {
					offset = self.imm;
				}
			},

			FUNC3_BLTU => {
				self.name = String::from("bltu");
				if src1 < src2 {
					offset = self.imm;
				}
			},

			FUNC3_BGE => {
				self.name = String::from("bge");
				if (src1 as i64) >= (src2 as i64) {
					offset = self.imm;
				}
			},

			FUNC3_BGEU => {
				self.name = String::from("bgeu");
				if src1 >= src2 {
					offset = self.imm;
				}
			},

			_ => {
				todo!("branch w/ func3 {:b}", self.func3);
			},
		}

		if offset != 0 {
			offset = sign_extend!(offset, 12, i32);
			hart.pc = hart.pc.wrapping_add_signed(offset as i64);
			debug_println!("Branching to {:x}", hart.pc);
		} else {
			debug_println!("Branch not taken @ {:?} {:b}", hart.pc, self.func3);
			hart.pc += 4;
		}
	}

	fn handle_ui_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let hart = &mut (platform.write().unwrap()).hart;

		match self.opcode {
			OPCODE_AUIPC => {
				self.name = String::from("auipc");
				let tmp: i64 = self.imm.try_into().unwrap();
				hart.write_register(
					self.rd as usize,
					hart.pc.wrapping_add_signed(tmp),
				);

				debug_println!(
					"auipc: added {:x} to {:x} and stored in {:x}",
					self.imm,
					hart.pc,
					self.rd
				);
			},

			OPCODE_LUI => {
				self.name = String::from("lui");
				let tmp: i64 = self.imm.try_into().unwrap();
				hart.write_register(self.rd as usize, tmp as u64);

				debug_println!("lui: put {:x} in {:x}", self.imm, self.rd);
			},

			_ => todo!("upper imm"),
		}
	}

	fn handle_atomic_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let func5 = self.func7 & gen_mask!(6, 2, u32);
		if func5 == FUNC7_LR {
			self.handle_lr_insn(platform);
		} else if func5 == FUNC7_SC {
			self.handle_sc_insn(platform);
		} else if self.func3 == FUNC3_RV32_ATOMIC {
			self.handle_atomic_rv32_insn(platform);
		} else {
			self.handle_atomic_rv64_insn(platform);
		}

		debug_println!("Found {:}", self.name);
	}

	fn handle_sc_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		self.name = String::from("sc");
		let platform_bus = &mut platform.write().unwrap();
		let hart_id = platform_bus.hart.id;
		let address: u64 = platform_bus.hart.read_register(self.rs1 as usize);
		let val: u64 = platform_bus.hart.read_register(self.rs2 as usize);
		let mut write_size = 4;

		if self.func3 == 0b010 {
			write_size = 2;
		}

		// If we do not have a reservation, then abort leaving a
		// non-zero value in rd.
		if !platform_bus.check_invalidate_reservation_set(
			hart_id,
			address as usize,
			write_size,
		) {
			platform_bus.hart.write_register(self.rd as usize, 1);
			return;
		}

		if self.func3 == 0b010 {
			let val = (val & gen_mask!(31, 0, u64)) as u32;
			let _ =
				platform_bus.write_from_hart(hart_id, address as usize, val);
		} else {
			let _ =
				platform_bus.write_from_hart(hart_id, address as usize, val);
		}

		platform_bus.hart.write_register(self.rd as usize, 0);
	}

	fn handle_lr_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		self.name = String::from("lr");
		let platform_bus = &mut platform.write().unwrap();
		let hart_id = platform_bus.hart.id;
		let address: u64 = platform_bus.hart.read_register(self.rs1 as usize);
		let mut read_size = 4;
		let val: u64;

		if self.func3 == 0b010 {
			read_size = 2;
			let tmp: u32 = platform_bus.read(address as usize).unwrap();
			val = tmp as i32 as i64 as u64;
		} else {
			val = platform_bus.read(address as usize).unwrap();
		}

		platform_bus.claim_reservation_set(
			hart_id,
			address as usize,
			read_size,
		);
		platform_bus.hart.write_register(self.rd as usize, val);
	}

	fn handle_atomic_rv64_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let platform_bus = &mut platform.write().unwrap();

		// Quoting the spec:
		// AMO instructions atomically load a data value from the
		// address in rs1, place the value into register rd, apply a
		// binary operator to the loaded value and the original value
		// in rs2, then store the result back to the address in rs1
		// I am just ignoring aq/rl here, because this system is super
		// trivial, and a lock is taken for all memory access anyway
		let address: u64 = platform_bus.hart.read_register(self.rs1 as usize);
		let mut val: u64 = platform_bus.read(address as usize).unwrap();
		platform_bus.hart.write_register(self.rd as usize, val);
		let other_val: u64 = platform_bus.hart.read_register(self.rs2 as usize);

		match self.func7 & gen_mask!(6, 2, u32) {
			FUNC7_AMOADD => {
				self.name = String::from("amoadd");
				val += other_val;
			},

			FUNC7_AMOAND => {
				self.name = String::from("amoand");
				val &= other_val;
			},

			FUNC7_AMOOR => {
				self.name = String::from("amoor");
				val |= other_val;
			},

			FUNC7_AMOXOR => {
				self.name = String::from("amoadd");
				val ^= other_val;
			},

			FUNC7_AMOSWAP => {
				self.name = String::from("amoswap");
				val = other_val;
			},

			_ => todo!("atomic {:b}", (self.func7 & gen_mask!(6, 2, u32)) >> 2),
		}

		let hart_id = platform_bus.hart.id;
		let _ = platform_bus.write_from_hart(hart_id, address as usize, val);

		debug_println!("Found {:}", self.name);
	}

	fn handle_atomic_rv32_insn(&mut self, platform: &Arc<RwLock<&mut Platform>>)
	{
		let platform_bus = &mut platform.write().unwrap();

		// Quoting the spec:
		// AMO instructions atomically load a data value from the
		// address in rs1, place the value into register rd, apply a
		// binary operator to the loaded value and the original value
		// in rs2, then store the result back to the address in rs1
		// I am just ignoring aq/rl here, because this system is super
		// trivial, and a lock is taken for all memory access anyway
		let address: u64 = platform_bus.hart.read_register(self.rs1 as usize);
		let mut val: u32 = platform_bus.read(address as usize).unwrap();
		let rd: u64 = val as i32 as i64 as u64;
		platform_bus.hart.write_register(self.rd as usize, rd);
		// check this to make sure the mask is okay to do
		let other_val: u32 =
			(platform_bus.hart.read_register(self.rs2 as usize)
				& gen_mask!(31, 0, u64)) as u32;

		match self.func7 & gen_mask!(6, 2, u32) {
			FUNC7_AMOADD => {
				self.name = String::from("amoadd");
				val += other_val;
			},

			FUNC7_AMOAND => {
				self.name = String::from("amoand");
				val &= other_val;
			},

			FUNC7_AMOOR => {
				self.name = String::from("amoor");
				val |= other_val;
			},

			FUNC7_AMOXOR => {
				self.name = String::from("amoadd");
				val ^= other_val;
			},

			FUNC7_AMOSWAP => {
				self.name = String::from("amoswap");
				val = other_val;
			},

			_ => todo!("atomic {:b}", (self.func7 & gen_mask!(6, 2, u32)) >> 2),
		}

		let hart_id = platform_bus.hart.id;
		let _ = platform_bus.write_from_hart(hart_id, address as usize, val);

		debug_println!("Found {:}", self.name);
	}

	fn increment_pc(&self, platform: &Arc<RwLock<&mut Platform>>)
	{
		match self.opcode {
			OPCODE_JAL | OPCODE_JALR | OPCODE_BRANCH => (),

			_ => {
				let hart = &mut (platform.write().unwrap()).hart;
				hart.pc += 4;
			},
		}
	}

	pub fn handle(&mut self, platform: &mut Platform)
	{
		let arc = Arc::new(std::sync::RwLock::new(platform));

		match self.opcode {
			OPCODE_LUI | OPCODE_AUIPC => {
				self.handle_ui_insn(&arc);
			},

			OPCODE_INT_REG_REG => {
				self.handle_int_reg_reg_insn(&arc);
			},

			OPCODE_INT_REG_IMM => {
				self.handle_int_reg_imm_insn(&arc);
			},

			OPCODE_STORE => {
				self.handle_store_insn(&arc);
			},

			OPCODE_LOAD => {
				self.handle_load_insn(&arc);
			},

			OPCODE_SYSTEM => {
				self.handle_csr_insn(&arc);
			},

			OPCODE_JAL | OPCODE_JALR => {
				self.handle_jump_insn(&arc);
			},

			OPCODE_MISCMEM => {
				debug_println!("fence.i");
			},

			OPCODE_BRANCH => {
				self.handle_branch_insn(&arc);
			},

			OPCODE_INT_REG_IMM_32 => {
				self.handle_int_reg_imm32_insn(&arc);
			},

			OPCODE_ATOMIC => {
				self.handle_atomic_insn(&arc);
			},

			_ => {
				debug_println!("unimplemented instruction {:x}", self.opcode);
				dump_unimplemented_insn(self, &arc);
				panic!();
			},
		}

		self.increment_pc(&arc);

		return;
	}
}

fn dump_unimplemented_insn(insn: &Insn, platform: &Arc<RwLock<&mut Platform>>)
{
	let hart = &mut (platform.write().unwrap()).hart;
	debug_println!(
		"insn {:?}\n hart registers {:?}\n pc {:x}",
		insn,
		hart.registers,
		hart.pc
	);
}

impl From<u32> for Insn
{
	fn from(input: u32) -> Self
	{
		let mut insn: Insn = Insn::default();

		insn.parse(input);

		return insn;
	}
}
