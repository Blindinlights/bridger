pub mod arm64;
pub mod riscv;
#[derive(Debug, Clone)]
pub struct Register {
    pub name: String,
    pub size: u32,
}
#[derive(Debug, Clone)]
pub struct Immediate(u64);

impl Immediate {
    pub fn new(value: u64) -> Self {
        Immediate(value)
    }
    pub fn check_arm64(value: u64) -> bool {
        todo!()
    }
    pub fn check_riscv(value: u64) -> bool {
        todo!()
    }
}
