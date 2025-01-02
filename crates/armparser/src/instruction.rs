#[derive(Debug, PartialEq)]
pub struct Instruction {
    label: Option<String>,
    opcode: String,
    operands: Vec<Operand>,
    operand_count: u32,
}
pub type P<T> = Box<T>;
#[derive(Debug, PartialEq)]
pub enum Operand {
    Register(P<Register>),
    Immediate(P<Immediate>),
    Memory(P<Memory>),
    Label(P<Label>),
}
#[derive(Debug, PartialEq)]
pub struct Register {
    //pub name: String,
    pub number: u32,
}
#[derive(Debug, PartialEq)]
pub struct Immediate {
    pub value: u32,
}
#[derive(Debug, PartialEq)]
pub struct Memory {
    pub base: Option<Register>,
    pub offset: Option<Immediate>,
}
#[derive(Debug, PartialEq)]
pub struct Label {
    pub name: String,
}
