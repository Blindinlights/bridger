use smallvec::SmallVec;

use crate::{Immediate, Register};

pub struct RiscvInst {
    pub opcode: String,
    pub rd: Option<Register>,
    pub rs: SmallVec<[Register; 3]>,
    pub imm: Option<Immediate>,
    pub label: Option<String>,
}
