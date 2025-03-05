#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Vec<Operand>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opcode {

    /// Arithmetic
    Add,Adds,Sub,Subs,Mul,Div,Umaddl,

    /// Bitwise
    And,Or,Xor,Not,

    /// Shift
    Lsl,Lsr,Asr,

    Cmp,

    /// Move
    Mov,Mvn,Mvk,

    /// Load/Store
    Ldr,Str,

    /// Branch
    B,Ble,Blt,Bge,Bgt,Beq,Bne,Bpl,Bhi,Cbnz,Csel,Cset,Bl,

    /// Floating-point
    Fmov,
    Ucvtf,Scvtf,
    Fcmp,Fcmpe,
    Fadd,Fsub,Fmul,Fdiv,Fneg,Fsqrt,
    Fmsub,Fnmadd,Fnmul,Fmuladd,

    /// Atomic
    Ldaxr,Stlxr,Ldar,Stlr,
    //TODO: Atomic Operations
    Nop,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    /// Register
    Reg(Regoperand),
    ///Imm
    Imm {
        imm: u16,
        shift: Option<(u8, Shift)>,
    },

    /// Addressing
    Addressing(Addressing),
    /// Label
    Label(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Regoperand {
    Reg(Register),
    ShiftReg(Register, (u8, Shift)),
    ExtendReg(Register, (u8, Extend)),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Shift {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Uxtb,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Extend {
    Uxtb,
    Uxth,
    Uxtw,
    Lsl,
    Uxtx,
    Sxtb,
    Sxth,
    Sxtw,
    Sxtx,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Addressing {
    BaseRegister(Register),
    Offset {
        offset: u64,
        reg: Register,
        index: Option<Index>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Index {
    Pre,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    General { ty: General, n: u8 },
    Special(Special),
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy,PartialEq, Eq, Hash)]
pub enum General {

    /// General purpose registers
    X,W,

    /// Vector Register
    B,H,S,D,Q,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Special {
    Xzr,
    Wzr,
    SP,
    LR,
}

impl Register {
    pub fn is_word(&self) -> bool {
        match self {
            Register::General { ty, .. } => match ty {
                General::W => true,
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_fword(&self) -> bool {
        match self {
            Register::General { ty, .. } => match ty {
                General::S => true,
                _ => false,
            },
            _ => false,
        }
    }
}
impl Regoperand {
    pub fn is_word(&self) -> bool {
        match self {
            Regoperand::Reg(reg) => reg.is_word(),
            Regoperand::ShiftReg(reg, _) => reg.is_word(),
            Regoperand::ExtendReg(reg, _) => reg.is_word(),
        }
    }
    pub fn is_fword(&self) -> bool {
        match self {
            Regoperand::Reg(reg) => reg.is_fword(),
            Regoperand::ShiftReg(reg, _) => reg.is_fword(),
            Regoperand::ExtendReg(reg, _) => reg.is_fword(),
        }
    }
}
