pub mod error;
pub mod opcode;
#[cfg(test)]
pub mod tests;
use error::PrintError;
use opcode::Opcode;
use pest_derive::Parser;

macro_rules! location {
    () => {
        concat!(file!(), ":", line!())
    };
}

#[derive(Parser)]
#[grammar = "arm64.pest"] // 使用前面定义的pest语法文件
pub struct ARM64Parser;

// 基础数据类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    HalfReg(u8), // w0-w31
    FullReg(u8), // x0-x31
    SpecialReg(SpecialRegister),
    FloatReg(FloatRegister),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatRegister {
    Half(u8),   // h0-h31
    Single(u8), // s0-s31
    Double(u8), // d0-d31
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialRegister {
    SP,  // Stack Pointer
    FP,  // Frame Pointer
    LR,  // Link Register
    XZR, // Zero Register (64-bit)
    WZR, // Zero Register (32-bit)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftType {
    LSL, // Logical Shift Left
    LSR, // Logical Shift Right
    ASR, // Arithmetic Shift Right
    ROR, // Rotate Right
    RRX, // Rotate Right with Extend
}

#[derive(Debug, Clone, PartialEq)]
pub struct Immediate(u64);

#[derive(Debug, Clone, PartialEq)]
pub struct ShiftedRegister {
    // The register to be shifted
    reg: Register,
    // The type of shift operation to perform (LSL, LSR, ASR, ROR, RRX)
    shift_type: ShiftType,
    // Optional amount to shift by - can be immediate value or register
    shift_amount: Option<ShiftAmount>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftAmount {
    // Immediate shift amount (constant)
    Immediate(Immediate),
    // Register containing shift amount
    Register(Register),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisterList {
    registers: Vec<Register>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisterRange {
    start: Register,
    end: Register,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Indirect {
    base: Register,
    offset: Option<Offset>,
    writeback: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Offset {
    Immediate(Immediate),
    Register(Register),
    ShiftedRegister(ShiftedRegister),
    ProcLoad(ProcLoad),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcLoad {
    mode: String,
    target: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Register(Register),
    Immediate(Immediate),
    Address(Immediate),
    LabelTarget(String),
    Indirect(Indirect),
    RegisterList(RegisterList),
    ShiftedRegister(ShiftedRegister),
    ProcLoad(ProcLoad),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
    operands: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Label(String),
    Directive(String),
    Instruction(Instruction),
}
type Err = crate::error::ArmParserError;
impl ARM64Parser {}

pub trait Parse {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err>
    where
        Self: Sized;
}

impl Parse for Register {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::register);
        let inner = pair.into_inner().next().expect("No inner pair");
        match inner.as_rule() {
            Rule::half_reg => Ok(Register::HalfReg(inner.as_str()[1..].parse()?)),
            Rule::full_reg => Ok(Register::FullReg(inner.as_str()[1..].parse()?)),
            Rule::float_reg => {
                let inner = inner.into_inner().next().expect("No inner pair");

                let fg = match inner.as_rule() {
                    Rule::float16 => FloatRegister::Half(inner.as_str()[1..].parse()?),
                    Rule::float32 => FloatRegister::Single(inner.as_str()[1..].parse()?),
                    Rule::double64 => FloatRegister::Double(inner.as_str()[1..].parse()?),
                    _ => unreachable!("Invalid float register"),
                };
                Ok(Register::FloatReg(fg))
            }

            _ => unreachable!("Invalid register"),
        }
    }
}

impl Parse for Immediate {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::immediate);
        fn parse_int(s: &str) -> Result<Immediate, Err> {
            if s.starts_with("0x") {
                Ok(Immediate(u64::from_str_radix(&s[2..], 16)?))
            } else {
                Ok(Immediate(s.parse()?))
            }
        }

        let literal = pair.as_str();
        let first_char = literal.chars().next().expect("No first char");
        match first_char {
            '#' => parse_int(&literal[1..]),
            _ => parse_int(literal),
        }
    }
}

impl Parse for ShiftType {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::shift_type);
        match pair.as_str() {
            "lsl" => Ok(ShiftType::LSL),
            "lsr" => Ok(ShiftType::LSR),
            "asr" => Ok(ShiftType::ASR),
            "ror" => Ok(ShiftType::ROR),
            "rrx" => Ok(ShiftType::RRX),
            _ => unreachable!("Invalid shift type"),
        }
    }
}

impl Parse for ShiftedRegister {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::shifted_register);
        let mut inner = pair.into_inner();
        let register = Register::parse(inner.next().expect("No inner pair"))?;
        let shift_type = ShiftType::parse(inner.next().expect("No inner pair"))?;
        let shift_amount = inner
            .next()
            .map(|pair| -> Result<ShiftAmount, Err> {
                let inner = pair.into_inner().next().expect("No inner pair");
                match inner.as_rule() {
                    Rule::immediate => Ok(ShiftAmount::Immediate(Immediate::parse(inner)?)),
                    Rule::register => Ok(ShiftAmount::Register(Register::parse(inner)?)),
                    _ => unreachable!("Invalid shift amount"),
                }
            })
            .transpose()?;
        Ok(ShiftedRegister {
            reg: register,
            shift_type,
            shift_amount,
        })
    }
}

impl Parse for Offset {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::offset);

        let inner = pair.into_inner().next().expect("No inner pair");
        match inner.as_rule() {
            Rule::immediate => Ok(Offset::Immediate(Immediate::parse(inner)?)),
            Rule::register => Ok(Offset::Register(Register::parse(inner)?)),
            Rule::shifted_register => Ok(Offset::ShiftedRegister(ShiftedRegister::parse(inner)?)),
            Rule::proc_load => Ok(Offset::ProcLoad(ProcLoad::parse(inner)?)),
            _ => unreachable!("Invalid offset"),
        }
    }
}

impl Parse for Indirect {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::indirect);
        let mut inner = pair.into_inner();
        let base = Register::parse(inner.next().expect("No inner pair"))?;
        let offset = inner
            .next()
            .map(|pair| -> Result<Offset, Err> { Ok(Offset::parse(pair)?) })
            .transpose()?;
        let writeback = inner.next().is_some();
        Ok(Indirect {
            base,
            offset,
            writeback,
        })
    }
}
impl Parse for ProcLoad {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::proc_load);
        let mut inner = pair.into_inner();
        let mode = inner.next().expect("No inner pair").as_str().to_string();
        let target = inner.next().expect("No inner pair").as_str().to_string();
        Ok(ProcLoad { mode, target })
    }
}

impl Parse for RegisterList {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::reglist);
        let mut registers = Vec::with_capacity(30);

        pair.into_inner()
            .map(|pair| {
                debug_assert_eq!(pair.as_rule(), Rule::register_item);
                pair.into_inner().next().expect("No inner pair")
            })
            .for_each(|pair| match pair.as_rule() {
                Rule::register => {
                    registers.push(Register::parse(pair).expect("Invalid register"));
                }
                Rule::register_range => {
                    let range = RegisterRange::parse(pair).expect("Invalid register range");
                    let list = range.to_reg_list().expect("Invalid register range");
                    registers.extend(list.registers);
                }
                _ => unreachable!("Invalid register list"),
            });
        registers.shrink_to_fit();
        Ok(RegisterList { registers })
    }
}

impl Parse for RegisterRange {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::register_range);
        let mut inner = pair.into_inner();
        let start = Register::parse(inner.next().expect("No inner pair"))?;
        let end = Register::parse(inner.next().expect("No inner pair"))?;
        Ok(RegisterRange { start, end })
    }
}

impl Parse for Operand {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::operand);
        let inner = pair.into_inner().next().expect("No inner pair");
        match inner.as_rule() {
            Rule::register => Ok(Operand::Register(Register::parse(inner)?)),
            Rule::immediate => Ok(Operand::Immediate(Immediate::parse(inner)?)),
            Rule::address => Ok(Operand::Address(Immediate::parse(inner)?)),
            Rule::label_target => Ok(Operand::LabelTarget(inner.as_str().to_string())),
            Rule::indirect => Ok(Operand::Indirect(Indirect::parse(inner)?)),
            Rule::reglist => Ok(Operand::RegisterList(RegisterList::parse(inner)?)),
            Rule::shifted_register => Ok(Operand::ShiftedRegister(ShiftedRegister::parse(inner)?)),
            Rule::proc_load => Ok(Operand::ProcLoad(ProcLoad::parse(inner)?)),
            _ => unreachable!("Invalid operand"),
        }
    }
}

impl RegisterRange {
    fn to_reg_list(&self) -> Result<RegisterList, Err> {
        let mut registers = Vec::new();
        match (&self.start, &self.end) {
            (Register::FullReg(start), Register::FullReg(end)) => {
                for i in *start..=*end {
                    registers.push(Register::FullReg(i));
                }
            }
            (Register::HalfReg(start), Register::HalfReg(end)) => {
                for i in *start..=*end {
                    registers.push(Register::HalfReg(i));
                }
            }
            (
                Register::FloatReg(FloatRegister::Half(start)),
                Register::FloatReg(FloatRegister::Half(end)),
            ) => {
                for i in *start..=*end {
                    registers.push(Register::FloatReg(FloatRegister::Half(i)));
                }
            }
            (
                Register::FloatReg(FloatRegister::Single(start)),
                Register::FloatReg(FloatRegister::Single(end)),
            ) => {
                for i in *start..=*end {
                    registers.push(Register::FloatReg(FloatRegister::Single(i)));
                }
            }
            (
                Register::FloatReg(FloatRegister::Double(start)),
                Register::FloatReg(FloatRegister::Double(end)),
            ) => {
                for i in *start..=*end {
                    registers.push(Register::FloatReg(FloatRegister::Double(i)));
                }
            }

            _ => {
                todo!()
            }
        }
        Ok(RegisterList { registers })
    }
}
