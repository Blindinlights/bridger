use smallvec::SmallVec;

use crate::Register;

#[derive(Debug, Clone)]
pub struct Arm64Inst {
    pub opcode: String,
    pub rd: Option<Register>,
    pub rs: SmallVec<[Register; 3]>,
    pub imm: Option<u64>,
    pub label: Option<String>,
    pub offset: Option<u64>,
}
