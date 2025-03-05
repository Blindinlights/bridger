#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Operands,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operands {
    R {
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    I {
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },

    S {
        rs1: Register,
        rs2: Register,
        imm: Immediate,
    },

    U {
        rd: Register,
        imm: Immediate,
    },

    /// `ret` pre
    Nop,
    Label(String),
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opcode{
    /// Arithmetic
    Add,Addi,Sub,Lui,Auipc,
    Addw,Addiw,Subw,
    /// Shift
    Sll,Slli,Srl,Srli,Sra,Srai,
    Sllw,Slliw,Srlw,Srliw,Sraw,Sraiw,
    /// Logical
    Xor,Xori,And,Andi,Or,Ori,
    /// Compare
    Slt,Slti,Sltu,Sltiu,
    /// Branch
    Beq,Bne,Blt,Bge,Bltu,Bgeu,
    Beqz,Bnez,Blez,Bgez,Bltz,Bgtz,Bgt,Ble,
    Bgtu,Bleu,
    ///  Jump ana Link
    Jal,Jalr,
    /// System
    Ecall,Ebreak,
    /// Synch
    Fence,Fencei,
    /// Load
    Lb,Lh,Lbu,Lhu,Lw,Lwu,Ld,
    /// Store
    Sb,Sh,Sw,Sd,
    /// CSR
    Csrrw,Csrrs,Csrrc,Csrrwi,Csrrsi,Csrrci,
    /// Multiply,Divide and Reminder
    Mul,Mulh,Mulhsu,Mulhu,Mulw,
    Div,Divu,
    Rem,Remu,
    /// Atomic
    Lr,Sc,
    Amoswap,Amoadd,
    Amoxor,Amoand,Amoor,
    Amomin,Amomax,Amominu,Amomaxu,

    /// Floating-point
    Fcvt,Fmv,
    Fadd,Fsub,Fmul,Fdiv,Fsqrt,
    Fmadd,Fmsub,Fnmadd,Fnmsub,
    Fsgnj,Fsgnjn,Fsgnjx,

    /// Pseudoinstruction
    La,Lla,Lga,Mv,
    Nop,
    Not,Neg,Negw,
    J,Ret,Call,Tail,
    Pause,

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    X(u8),
    F(u8),
    PC,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Immediate {
    Number(i16),
    Label(String),
}

impl Register {
    pub fn abi_name(&self) -> &'static str {
        match self {
            Register::X(x) => match x {
                0 => "zero",
                1 => "ra",
                2 => "sp",
                3 => "gp",
                4 => "tp",
                5 => "t0",
                6 => "t1",
                7 => "t2",
                8 => "s0", // 或 "fp"
                9 => "s1",
                10 => "a0",
                11 => "a1",
                12 => "a2",
                13 => "a3",
                14 => "a4",
                15 => "a5",
                16 => "a6",
                17 => "a7",
                18 => "s2",
                19 => "s3",
                20 => "s4",
                21 => "s5",
                22 => "s6",
                23 => "s7",
                24 => "s8",
                25 => "s9",
                26 => "s10",
                27 => "s11",
                28 => "t3",
                29 => "t4",
                30 => "t5",
                31 => "t6",
                _ => unreachable!(),
            },

            Register::F(f) => match f {
                0 => "ft0",
                1 => "ft1",
                2 => "ft2",
                3 => "ft3",
                4 => "ft4",
                5 => "ft5",
                6 => "ft6",
                7 => "ft7",
                8 => "fs0",
                9 => "fs1",
                10 => "fa0",
                11 => "fa1",
                12 => "fa2",
                13 => "fa3",
                14 => "fa4",
                15 => "fa5",
                16 => "fa6",
                17 => "fa7",
                18 => "fs2",
                19 => "fs3",
                20 => "fs4",
                21 => "fs5",
                22 => "fs6",
                23 => "fs7",
                24 => "fs8",
                25 => "fs9",
                26 => "fs10",
                27 => "fs11",
                28 => "ft8",
                29 => "ft9",
                30 => "ft10",
                31 => "ft11",
                _ => unreachable!(),
            },

            Register::PC => "pc",
        }
    }
    pub const ZERO: Register = Register::X(0);
    pub const T0: Register = Register::X(5);
    pub const T1: Register = Register::X(6);
    pub const T2: Register = Register::X(7);
    pub const T3: Register = Register::X(28);
    pub const T4: Register = Register::X(29);
    pub const T5: Register = Register::X(30);
    pub const T6: Register = Register::X(31);
}

impl Instruction {
    pub fn new_i(opcode: Opcode, rd: Register, rs1: Register, imm: Immediate) -> Self {
        Self {
            opcode,
            operands: Operands::I { rd, rs1, imm },
        }
    }
    pub fn new_r(opcode: Opcode, rd: Register, rs1: Register, rs2: Register) -> Self {
        Self {
            opcode,
            operands: Operands::R { rd, rs1, rs2 },
        }
    }
    pub fn new_s(opcode: Opcode, rs1: Register, rs2: Register, imm: Immediate) -> Self {
        Self {
            opcode,
            operands: Operands::S { rs1, rs2, imm },
        }
    }

    pub fn new_u(opcode: Opcode, rd: Register, imm: Immediate) -> Self {
        Self {
            opcode,
            operands: Operands::U { rd, imm },
        }
    }
    pub fn new_ret() -> Self {
        Self {
            opcode: Opcode::Ret,
            operands: Operands::Nop,
        }
    }
    pub fn new_label(opcode: Opcode, label: String) -> Self {
        Self {
            opcode,
            operands: Operands::Label(label),
        }
    }
    pub fn new_nop() -> Self {
        Self {
            opcode: Opcode::Nop,
            operands: Operands::Nop,
        }
    }
}
impl Opcode {
    pub fn to_imm(&self) -> Opcode {
        match self {
            Opcode::Add => Opcode::Addi,
            Opcode::Addw => Opcode::Addiw,
            Opcode::Xor => Opcode::Xori,
            Opcode::And => Opcode::Andi,
            Opcode::Or => Opcode::Ori,
            Opcode::Slt => Opcode::Slti,
            Opcode::Sltu => Opcode::Sltiu,
            Opcode::Sra => Opcode::Srai,
            Opcode::Srl => Opcode::Srli,
            Opcode::Sll => Opcode::Slli,
            Opcode::Sraw => Opcode::Sraiw,
            Opcode::Srlw => Opcode::Srliw,
            Opcode::Sllw => Opcode::Slliw,

            _ => self.clone(),
        }
    }
}
