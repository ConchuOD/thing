// SPDX-License-Identifier: GPL-2.0-only
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crate::bus::Bus;
use crate::hart;

#[derive(PartialEq)]
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

macro_rules! gen_mask {
	($h:expr, $l:expr, $typ:ty) => {
		(((!0) - (1_u64.wrapping_shl($l)) + 1) & (!0 & (!0_u64 >> (64 - 1 - ($h)) as u64))) as $typ
	};
}

const OPCODE_MASK: u32 = 0b0111_1111;
const OPCODE_LUI: u32 = 0b0011_0111;
const OPCODE_AUIPC: u32 = 0b0001_0111;
const OPCODE_JAL: u32 = 0b0110_1111;

const OPCODE_STORE: u32 = 0b0100011;
const OPCODE_LOAD: u32 = 0b0000011;

const OPCODE_INT_REG_IMM: u32 = 0b0001_0011;
const OPCODE_INT_REG_REG: u32 = 0b0011_0011;

const RD_SHIFT: u32 = 7;
const RD_WIDTH: u32 = 5;
const RD_MASK: u32 = gen_mask!(RD_SHIFT + RD_WIDTH - 1, RD_SHIFT, u32);

const RS1_SHIFT: u32 = 15;
const RS1_WIDTH: u32 = 5;
const RS1_MASK: u32 = gen_mask!(RS1_SHIFT + RS1_WIDTH - 1, RS1_SHIFT, u32);

const RS2_SHIFT: u32 = 20;
const RS2_WIDTH: u32 = 5;
const RS2_MASK: u32 = gen_mask!(RS2_SHIFT + RS2_WIDTH - 1, RS2_SHIFT, u32);

const IMM_SHIFT_UTYPE: u32 = 12;
const IMM_WIDTH_UTYPE: u32 = 20;
const IMM_MASK_UTYPE: u32 = gen_mask!(IMM_SHIFT_UTYPE + IMM_WIDTH_UTYPE - 1, IMM_SHIFT_UTYPE, u32);

const IMM_SHIFT_ITYPE: u32 = 20;
const IMM_WIDTH_ITYPE: u32 = 12;
const IMM_MASK_ITYPE: u32 = gen_mask!(IMM_SHIFT_ITYPE + IMM_WIDTH_ITYPE - 1, IMM_SHIFT_ITYPE, u32);

const IMM_SHIFT_STYPE: u32 = RD_SHIFT;
const IMM_WIDTH_STYPE: u32 = RD_WIDTH;
const IMM_MASK_STYPE: u32 = gen_mask!(IMM_SHIFT_STYPE + IMM_WIDTH_STYPE - 1, IMM_SHIFT_STYPE, u32);

const IMM2_SHIFT_STYPE: u32 = 25;
const IMM2_WIDTH_STYPE: u32 = 7;
const IMM2_MASK_STYPE: u32 =
	gen_mask!(IMM2_SHIFT_STYPE + IMM2_WIDTH_STYPE - 1, IMM2_SHIFT_STYPE, u32);

const FUNC3_SHIFT_ITYPE: u32 = 12;
const FUNC3_WIDTH_ITYPE: u32 = 3;
const FUNC3_MASK_ITYPE: u32 =
	gen_mask!(FUNC3_SHIFT_ITYPE + FUNC3_WIDTH_ITYPE - 1, FUNC3_SHIFT_ITYPE, u32);

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

const FUNC3_SB: u32 = 0b000;
const FUNC3_SH: u32 = 0b001;
const FUNC3_SW: u32 = 0b010;
const FUNC3_SD: u32 = 0b011;

const FUNC3_LB: u32 = 0b000;
const FUNC3_LH: u32 = 0b001;
const FUNC3_LW: u32 = 0b010;
const FUNC3_LD: u32 = 0b011;

const FUNC7_SHIFT_ITYPE: u32 = IMM2_SHIFT_STYPE;
const FUNC7_WIDTH_ITYPE: u32 = IMM2_WIDTH_STYPE;
const FUNC7_MASK_ITYPE: u32 = IMM2_MASK_STYPE;

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
			_ => println!("unknown opcode {:?}", self.opcode),
		}

		match self.insn_type {
			InsnType::U => {
				self.imm = (input & IMM_MASK_UTYPE) as i32;
				self.rd = input & RD_MASK >> RD_SHIFT;
			},

			InsnType::I => {
				self.imm = ((input & IMM_MASK_ITYPE) >> IMM_SHIFT_ITYPE) as i32;
				self.rd = (input & RD_MASK) >> RD_SHIFT;
				self.rs1 = (input & RS1_MASK) >> RS1_SHIFT;
				self.func3 = (input & FUNC3_MASK_ITYPE) >> FUNC3_SHIFT_ITYPE;
			},

			InsnType::R => {
				self.rd = (input & RD_MASK) >> RD_SHIFT;
				self.rs1 = (input & RS1_MASK) >> RS1_SHIFT;
				self.rs2 = (input & RS2_MASK) >> RS2_SHIFT;
				self.func3 = (input & FUNC3_MASK_ITYPE) >> FUNC3_SHIFT_ITYPE;
				self.func7 = (input & FUNC7_MASK_ITYPE) >> FUNC7_SHIFT_ITYPE;
			},

			InsnType::S => {
				self.rs1 = (input & RS1_MASK) >> RS1_SHIFT;
				self.rs2 = (input & RS2_MASK) >> RS2_SHIFT;
				self.func3 = (input & FUNC3_MASK_ITYPE) >> FUNC3_SHIFT_ITYPE;

				let lower_imm = (input & IMM_MASK_STYPE) >> IMM_SHIFT_STYPE;
				let upper_imm = (input & IMM2_MASK_STYPE) >> IMM2_SHIFT_STYPE;
				self.imm = ((upper_imm << IMM_WIDTH_STYPE) | lower_imm) as i32;
			},

			_ => (),
		}
	}

	fn handle_int_reg_reg_insn(&mut self, hart: &mut hart::Hart)
	{
		match self.func3 {
			FUNC3_ADD => {
				if self.func7 != 0 {
					self.name = String::from("add");
					// ADD adds the value in rs1 to rs2 and stores
					// the result in rd
					// overflows are ignored, the lower XLEN bits
					// get written
					let rs1: u64 = hart.read_register(self.rs1 as usize);
					let rs2: u64 = hart.read_register(self.rs2 as usize);
					let tmp: u64 = rs1.wrapping_add(rs2);

					hart.write_register(self.rd as usize, tmp);
				} else {
					self.name = String::from("sub");
					// SUB subtracts the value in rs2 from rs1
					// and stores the result in rd
					// overflows are ignored, the lower XLEN bits
					// get written
					let rs1: u64 = hart.read_register(self.rs1 as usize);
					let rs2: u64 = hart.read_register(self.rs2 as usize);
					let tmp: u64 = rs1.wrapping_sub(rs2);

					hart.write_register(self.rd as usize, tmp);
				}
			},

			FUNC3_SLTU => {
				self.name = String::from("sltu");
			},

			_ => (),
		}

		println!("Found {:}", self.name);
	}

	fn handle_int_reg_imm_insn(&mut self, hart: &mut hart::Hart)
	{
		match self.func3 {
			FUNC3_ADDI => {
				self.name = String::from("addi");
				// ADDI adds the sign-extended 12-bit immediate
				// to register rs1. Arithmetic overflow is
				// ignored and the result is simply the low XLEN
				// bits of the result.
				let mut tmp: u64 = hart.read_register(self.rs1 as usize);
				let mut imm: i64 = self.imm.try_into().unwrap();
				imm = imm.wrapping_shl(52).wrapping_shr(52);
				tmp = tmp.wrapping_add_signed(imm);
				hart.write_register(self.rd as usize, tmp);
			},

			FUNC3_SLTI => {
				self.name = String::from("slti");
			},

			_ => (),
		}

		println!("Found {:}", self.name);
	}

	fn handle_store_insn(&mut self, hart: &mut hart::Hart)
	{
		// These are all store instructions of varied widths
		match self.func3 {
			FUNC3_SD => {
				self.name = String::from("sd");
				let tmp: u64 = hart.read_register(self.rd as usize);
				hart.write(self.rd as usize, tmp);
			},

			FUNC3_SW => {
				self.name = String::from("sw");
				let tmp: u64 = hart.read_register(self.rs2 as usize) & gen_mask!(31, 0, u64);
				hart.write(self.rd as usize, tmp as u32);
			},

			FUNC3_SH => {
				self.name = String::from("sh");
				let tmp: u64 = hart.read_register(self.rs2 as usize) & gen_mask!(15, 0, u64);
				hart.write(self.rd as usize, tmp as u16);
			},

			FUNC3_SB => {
				self.name = String::from("sb");
				let tmp: u64 = hart.read_register(self.rs2 as usize) & gen_mask!(7, 0, u64);
				hart.write(self.rd as usize, tmp as u8);
			},

			_ => (),
		}

		println!("Found {:}", self.name);
	}

	fn handle_load_insn(&mut self, hart: &mut hart::Hart)
	{
		// These are all store instructions of varied widths
		match self.func3 {
			FUNC3_LD => {
				self.name = String::from("ld");
				let tmp: u64 = hart.read(self.rd as usize).unwrap();
				hart.write_register(self.rd as usize, tmp);
			},

			FUNC3_LW => {
				self.name = String::from("lw");
				let tmp: u32 = hart.read(self.rd as usize).unwrap();
				hart.write_register(self.rs2 as usize, tmp as u64);
			},

			FUNC3_LH => {
				self.name = String::from("lh");
				let tmp: u16 = hart.read(self.rs2 as usize).unwrap();
				hart.write_register(self.rs2 as usize, tmp as u64);
			},

			FUNC3_LB => {
				self.name = String::from("lb");
				let tmp: u8 = hart.read(self.rs2 as usize).unwrap();
				hart.write_register(self.rs2 as usize, tmp as u64);
			},

			_ => (),
		}

		println!("Found {:}", self.name);
	}

	pub fn handle(&mut self, hart: &mut hart::Hart)
	{
		match self.opcode {
			OPCODE_AUIPC => {
				let mut registers = hart.registers;
				self.name = String::from("auipc");
				println!("Found {:}", self.name);
				// @Johan: AUIPC is "add upper immediate to program counter"
				let tmp: i64 = self.imm.try_into().unwrap();
				registers[self.rd as usize] = hart.pc.wrapping_add_signed(tmp);
			},

			OPCODE_INT_REG_REG => {
				self.handle_int_reg_reg_insn(hart);
			},

			OPCODE_INT_REG_IMM => {
				self.handle_int_reg_imm_insn(hart);
			},

			OPCODE_STORE => {
				self.handle_store_insn(hart);
			},

			OPCODE_LOAD => {
				self.handle_load_insn(hart);
			},

			_ => {
				println!("unimplemented instruction {:x}", self.opcode);
			},
		}

		hart.pc += 4;

		return;
	}
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

//// RV32I Base Instruction Set
// imm[31:12] rd 0110111 LUI
// imm[31:12] rd 0010111 AUIPC
// imm[20|10:1|11|19:12] rd 1101111 JAL
// imm[11:0] rs1 000 rd 1100111 JALR
// imm[12|10:5] rs2 rs1 000 imm[4:1|11] 1100011 BEQ
// imm[12|10:5] rs2 rs1 001 imm[4:1|11] 1100011 BNE
// imm[12|10:5] rs2 rs1 100 imm[4:1|11] 1100011 BLT
// imm[12|10:5] rs2 rs1 101 imm[4:1|11] 1100011 BGE
// imm[12|10:5] rs2 rs1 110 imm[4:1|11] 1100011 BLTU
// imm[12|10:5] rs2 rs1 111 imm[4:1|11] 1100011 BGEU
// imm[11:0] rs1 000 rd 0000011 LB
// imm[11:0] rs1 001 rd 0000011 LH
// imm[11:0] rs1 010 rd 0000011 LW
// imm[11:0] rs1 100 rd 0000011 LBU
// imm[11:0] rs1 101 rd 0000011 LHU
// imm[11:5] rs2 rs1 000 imm[4:0] 0100011 SB
// imm[11:5] rs2 rs1 001 imm[4:0] 0100011 SH
// imm[11:5] rs2 rs1 010 imm[4:0] 0100011 SW
// imm[11:0] rs1 000 rd 0010011 ADDI
// imm[11:0] rs1 010 rd 0010011 SLTI
// imm[11:0] rs1 011 rd 0010011 SLTIU
// imm[11:0] rs1 100 rd 0010011 XORI
// imm[11:0] rs1 110 rd 0010011 ORI
// imm[11:0] rs1 111 rd 0010011 ANDI
// 0000000 shamt rs1 001 rd 0010011 SLLI
// 0000000 shamt rs1 101 rd 0010011 SRLI
// 0100000 shamt rs1 101 rd 0010011 SRAI
// 0000000 rs2 rs1 000 rd 0110011 ADD
// 0100000 rs2 rs1 000 rd 0110011 SUB
// 0000000 rs2 rs1 001 rd 0110011 SLL
// 0000000 rs2 rs1 010 rd 0110011 SLT
// 0000000 rs2 rs1 011 rd 0110011 SLTU
// 0000000 rs2 rs1 100 rd 0110011 XOR
// 0000000 rs2 rs1 101 rd 0110011 SRL
// 0100000 rs2 rs1 101 rd 0110011 SRA
// 0000000 rs2 rs1 110 rd 0110011 OR
// 0000000 rs2 rs1 111 rd 0110011 AND
// 0000 pred succ 00000 000 00000 0001111 FENCE
// 0000 0000 0000 00000 001 00000 0001111 FENCE.I
// 000000000000 00000 000 00000 1110011 ECALL
// 000000000001 00000 000 00000 1110011 EBREAK
// csr rs1 001 rd 1110011 CSRRW
// csr rs1 010 rd 1110011 CSRRS
// csr rs1 011 rd 1110011 CSRRC
// csr zimm 101 rd 1110011 CSRRWI
// csr zimm 110 rd 1110011 CSRRSI
// csr zimm 111 rd 1110011 CSRRCI
//
//// RV64I Base Instruction Set (in addition to RV32I)
// imm[11:0] rs1 110 rd 0000011 LWU
// imm[11:0] rs1 011 rd 0000011 LD
// imm[11:5] rs2 rs1 011 imm[4:0] 0100011 SD
// 000000 shamt rs1 001 rd 0010011 SLLI
// 000000 shamt rs1 101 rd 0010011 SRLI
// 010000 shamt rs1 101 rd 0010011 SRAI
// imm[11:0] rs1 000 rd 0011011 ADDIW
// 0000000 shamt rs1 001 rd 0011011 SLLIW
// 0000000 shamt rs1 101 rd 0011011 SRLIW
// 0100000 shamt rs1 101 rd 0011011 SRAIW
// 0000000 rs2 rs1 000 rd 0111011 ADDW
// 0100000 rs2 rs1 000 rd 0111011 SUBW
// 0000000 rs2 rs1 001 rd 0111011 SLLW
// 0000000 rs2 rs1 101 rd 0111011 SRLW
// 0100000 rs2 rs1 101 rd 0111011 SRAW
//
//// RV32M Standard Extension
// 0000001 rs2 rs1 000 rd 0110011 MUL
// 0000001 rs2 rs1 001 rd 0110011 MULH
// 0000001 rs2 rs1 010 rd 0110011 MULHSU
// 0000001 rs2 rs1 011 rd 0110011 MULHU
// 0000001 rs2 rs1 100 rd 0110011 DIV
// 0000001 rs2 rs1 101 rd 0110011 DIVU
// 0000001 rs2 rs1 110 rd 0110011 REM
// 0000001 rs2 rs1 111 rd 0110011 REMU
//
//// RV64M Standard Extension (in addition to RV32M)
// 0000001 rs2 rs1 000 rd 0111011 MULW
// 0000001 rs2 rs1 100 rd 0111011 DIVW
// 0000001 rs2 rs1 101 rd 0111011 DIVUW
// 0000001 rs2 rs1 110 rd 0111011 REMW
// 0000001 rs2 rs1 111 rd 0111011 REMUW
//// RV32A Standard Extension
// 00010 aq rl 00000 rs1 010 rd 0101111 LR.W
// 00011 aq rl rs2 rs1 010 rd 0101111 SC.W
// 00001 aq rl rs2 rs1 010 rd 0101111 AMOSWAP.W
// 00000 aq rl rs2 rs1 010 rd 0101111 AMOADD.W
// 00100 aq rl rs2 rs1 010 rd 0101111 AMOXOR.W
// 01100 aq rl rs2 rs1 010 rd 0101111 AMOAND.W
// 01000 aq rl rs2 rs1 010 rd 0101111 AMOOR.W
// 10000 aq rl rs2 rs1 010 rd 0101111 AMOMIN.W
// 10100 aq rl rs2 rs1 010 rd 0101111 AMOMAX.W
// 11000 aq rl rs2 rs1 010 rd 0101111 AMOMINU.W
// 11100 aq rl rs2 rs1 010 rd 0101111 AMOMAXU.W
//
//// RV64A Standard Extension (in addition to RV32A)
// 00010 aq rl 00000 rs1 011 rd 0101111 LR.D
// 00011 aq rl rs2 rs1 011 rd 0101111 SC.D
// 00001 aq rl rs2 rs1 011 rd 0101111 AMOSWAP.D
// 00000 aq rl rs2 rs1 011 rd 0101111 AMOADD.D
// 00100 aq rl rs2 rs1 011 rd 0101111 AMOXOR.D
// 01100 aq rl rs2 rs1 011 rd 0101111 AMOAND.D
// 01000 aq rl rs2 rs1 011 rd 0101111 AMOOR.D
// 10000 aq rl rs2 rs1 011 rd 0101111 AMOMIN.D
// 10100 aq rl rs2 rs1 011 rd 0101111 AMOMAX.D
// 11000 aq rl rs2 rs1 011 rd 0101111 AMOMINU.D
// 11100 aq rl rs2 rs1 011 rd 0101111 AMOMAXU.D
