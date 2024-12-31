#[derive(Debug, PartialEq)]
pub enum Register {
    X(u8),
    W(u8),
    SP,
    PC,
}
