use asm_inst::{arm64::Arm64Inst, riscv::RiscvInst};

pub type EmitRiscv = fn(&Arm64Inst) -> Vec<RiscvInst>;

fn add(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}

fn sub(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}

fn mov(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}

fn load_store(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}

fn load_store_pair(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}
fn ret(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}
fn branch(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}

fn load_store_reg(_inst: &Arm64Inst) -> Vec<RiscvInst> {
    todo!()
}
