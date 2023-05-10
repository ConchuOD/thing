#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(non_camel_case_types)]

enum Registers
{
	zero,
	ra,
	sp,
	gp,
	tp,
	t0,
	t1,
	t2,
	s0,
	s1,
	a0,
	a1,
	a2,
	a3,
	a4,
	a5,
	a6,
	a7,
	s2,
	s3,
	s4,
	s5,
	s6,
	s7,
	s8,
	s9,
	s10,
	s11,
	t3,
	t4,
	t5,
	t6,
}

#[derive(PartialEq)]
pub enum InsnType
{
	InvalidType,
	RType,
	IType,
	SType,
	BType,
	UType,
	JType,
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
	pub insn_type: InsnType,
}

macro_rules! gen_mask {
	($h:expr, $l:expr) => {
		(((!0) - (1_u32.wrapping_shl($l)) + 1) & (!0 & (!0_32 >> (32 - 1 - ($h)) as u32)))
	};
}

const OPCODE_MASK: u32 = 0b0111_1111;
const OPCODE_LUI: u32 = 0b0011_0111;
const OPCODE_AUIPC: u32 = 0b0001_0111;
const OPCODE_JAL: u32 = 0b0110_1111;

const OPCODE_ARITH: u32 = 0b0001_0011; // TODO: fix naming

const IMM_SHIFT_UTYPE: u32 = 12;
const IMM_WIDTH_UTYPE: u32 = 20;
const IMM_MASK_UTYPE: u32 = gen_mask!(IMM_SHIFT_UTYPE + IMM_WIDTH_UTYPE - 1, IMM_SHIFT_UTYPE);

const IMM_SHIFT_ITYPE: u32 = 20;
const IMM_WIDTH_ITYPE: u32 = 12;
const IMM_MASK_ITYPE: u32 = gen_mask!(IMM_SHIFT_ITYPE + IMM_WIDTH_ITYPE - 1, IMM_SHIFT_ITYPE);

const FUNC3_SHIFT_ITYPE: u32 = 12;
const FUNC3_WIDTH_ITYPE: u32 = 3;
const FUNC3_MASK_ITYPE: u32 =
	gen_mask!(FUNC3_SHIFT_ITYPE + FUNC3_WIDTH_ITYPE - 1, FUNC3_SHIFT_ITYPE);

// this should be an enum, right?
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

const RD_SHIFT: u32 = 7;
const RD_WIDTH: u32 = 5;
const RD_MASK: u32 = gen_mask!(RD_SHIFT + RD_WIDTH - 1, RD_SHIFT);

const RS1_SHIFT: u32 = 15;
const RS1_WIDTH: u32 = 5;
const RS1_MASK: u32 = gen_mask!(RS1_SHIFT + RS1_WIDTH - 1, RS1_SHIFT);

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
			insn_type: InsnType::InvalidType,
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
				self.insn_type = InsnType::UType;
			},

			OPCODE_JAL => {
				self.insn_type = InsnType::JType;
			},

			OPCODE_ARITH => {
				self.insn_type = InsnType::IType;
			},

			_ => print!("unknown opcode {:?}\n", self.opcode),
		}

		match self.insn_type {
			InsnType::UType => {
				self.imm = (input & IMM_MASK_UTYPE) as i32;
				self.rd = input & RD_MASK >> RD_SHIFT;
			},

			InsnType::IType => {
				self.imm = ((input & IMM_MASK_ITYPE) >> IMM_SHIFT_ITYPE) as i32;
				self.rd = (input & RD_MASK) >> RD_SHIFT;
				self.rs1 = (input & RS1_MASK) >> RS1_SHIFT;
				self.func3 = (input & FUNC3_MASK_ITYPE) >> FUNC3_SHIFT_ITYPE;
			},

			_ => (),
		}
	}

	fn arith(&mut self, registers: &mut [u64], pc: &mut u64)
	{
		match self.func3 {
			FUNC3_ADD => {
				self.name = String::from("addi");
				// ADDI adds the sign-extended 12-bit immediate
				// to register rs1. Arithmetic overflow is
				// ignored and the result is simply the low XLEN
				// bits of the result.
				let mut tmp: u64 = registers[self.rs1 as usize];
				let mut imm: i64 = self.imm.try_into().unwrap();
				imm = imm.wrapping_shl(52).wrapping_shr(52);
				tmp = tmp.wrapping_add_signed(imm);
				registers[self.rd as usize] = tmp;
			},

			_ => (),
		}

		print!("Found {:}\n", self.name);
	}

	fn do_insn(&mut self, registers: &mut [u64], pc: &mut u64)
	{
		if self.opcode == OPCODE_AUIPC {
			self.name = String::from("auipc");
			print!("Found {:}\n", self.name);
			// @Johan: AUIPC is "add upper immediate to program counter"
			let tmp: i64 = self.imm.try_into().unwrap();
			registers[self.rd as usize] = pc.wrapping_add_signed(tmp);
		} else if self.opcode == OPCODE_ARITH {
			self.arith(registers, pc);
		}

		*pc += 4;
		return;
	}

	pub fn handle(&mut self, registers: &mut [u64], pc: &mut u64)
	{
		if self.insn_type == InsnType::InvalidType {
			*pc += 4;
			print!("unimplemented instruction {:x}\n", self.opcode);
			return;
		}

		self.do_insn(registers, pc);

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
