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
	pub insn_type: InsnType,
}

const OPCODE_MASK: u32 = 0b0111_1111;
const OPCODE_LUI: u32 = 0b0011_0111;
const OPCODE_AUIPC: u32 = 0b0001_0111;
const OPCODE_JAL: u32 = 0b0110_1111;

const IMM_SHIFT_UTYPE: usize = 12;
const IMM_MASK_UTYPE: u32 = 0b1111_1111_1111_1111_1111_0000_0000_0000;
const RD_MASK: u32 = 0b0000_0000_0000_0000_0000_1111_1000_0000;

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
			insn_type: InsnType::InvalidType,
		};
	}
}

impl Insn
{
	fn parse(&mut self, input: u32)
	{
		match input | OPCODE_MASK {
			OPCODE_LUI | OPCODE_AUIPC => {
				self.insn_type = InsnType::UType;
			},

			OPCODE_JAL => {
				self.insn_type = InsnType::JType;
			},

			_ => (),
		}

		match self.insn_type {
			InsnType::UType => {
				self.imm = (input | IMM_MASK_UTYPE) as i32;
				self.rd = input | RD_MASK;
				self.opcode = (input | OPCODE_MASK) << IMM_SHIFT_UTYPE;
			},
			_ => (),
		}
	}

	fn do_insn(self, registers: &mut [u64], pc: &mut u64)
	{
		if self.opcode == OPCODE_AUIPC {
			// @Johan: AUIPC is "add upper immediate to program counter"
			let tmp: i64 = self.imm.try_into().unwrap();
			registers[self.rd as usize] = pc.wrapping_add_signed(tmp);
		}

		*pc += 4;
		return;
	}

	pub fn handle(self, registers: &mut [u64], pc: &mut u64)
	{
		if self.insn_type == InsnType::InvalidType {
			*pc += 4;
			print!("unimplemented instruction {:}\n", self.name);
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
