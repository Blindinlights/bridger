pub mod error;
#[cfg(test)]
pub mod tests;

use pest::{error::Error, Parser};
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "arm64.pest"] // 使用前面定义的pest语法文件
struct ARM64Parser;

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
pub enum Immediate {
    Decimal(i64),
    Hex(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShiftedRegister {
    // The register to be shifted
    register: Register,
    // The type of shift operation to perform (LSL, LSR, ASR, ROR, RRX)
    shift_type: ShiftType,
    // Optional amount to shift by - can be immediate value or register
    shift_amount: Option<ShiftAmount>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftAmount {
    // Immediate shift amount (constant)
    Immediate(u32),
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
    opcode: String,
    operands: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Label(String),
    Directive(String),
    Instruction(Instruction),
}
type Err = crate::error::ArmParserError;
impl ARM64Parser {
    pub(crate) fn parse_register(pair: pest::iterators::Pair<Rule>) -> Result<Register, Err> {
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
    pub(crate) fn parse_immediate(pair: pest::iterators::Pair<Rule>) -> Result<Immediate, Err> {
        let literal = pair.as_str();
        let first_char = literal.chars().next().expect("No first char");
        match first_char {
            '#' => Ok(Immediate::Decimal(literal[1..].parse()?)),
            '0' => Ok(Immediate::Hex(u64::from_str_radix(&literal[2..], 16)?)),
            _ => Ok(Immediate::Decimal(literal.parse()?)),
        }
    }
    pub(crate) fn parse_shift_type(pair: pest::iterators::Pair<Rule>) -> Result<ShiftType, Err> {
        let inner = pair.into_inner().next().expect("No inner pair");
        match inner.as_str() {
            "lsl" => Ok(ShiftType::LSL),
            "lsr" => Ok(ShiftType::LSR),
            "asr" => Ok(ShiftType::ASR),
            "ror" => Ok(ShiftType::ROR),
            "rrx" => Ok(ShiftType::RRX),
            _ => unreachable!("Invalid shift type"),
        }
    }
    pub(crate) fn parse_shift_register(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<ShiftedRegister, Err> {
        let mut inner = pair.into_inner();
        let register = ARM64Parser::parse_register(inner.next().expect("No inner pair"))?;
        let shift_type = ARM64Parser::parse_shift_type(inner.next().expect("No inner pair"))?;
        let shift_amount = inner
            .next()
            .map(|pair| -> Result<ShiftAmount, Err> {
                let inner = pair.into_inner().next().expect("No inner pair");
                match inner.as_rule() {
                    Rule::immediate => Ok(ShiftAmount::Immediate(inner.as_str().parse()?)),
                    Rule::register => {
                        Ok(ShiftAmount::Register(ARM64Parser::parse_register(inner)?))
                    }
                    _ => unreachable!("Invalid shift amount"),
                }
            })
            .transpose()?;
        Ok(ShiftedRegister {
            register,
            shift_type,
            shift_amount,
        })
    }
}
