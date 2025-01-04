pub mod error;
pub mod opcode;
pub mod register;
#[cfg(test)]
pub mod tests;
use opcode::{Opcode, OPCODES};
use pest_derive::Parser;
use register::Register;

#[derive(Parser)]
#[grammar = "arm64.pest"] // 使用前面定义的pest语法文件
pub struct ARM64Parser;

// 基础数据类型定义

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftType {
    LSL, // Logical Shift Left
    LSR, // Logical Shift Right
    ASR, // Arithmetic Shift Right
    ROR, // Rotate Right
    RRX, // Rotate Right with Extend
}

#[derive(Debug, Clone, PartialEq)]
pub struct Immediate(i64);

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
    pub regs: Vec<Register>,
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

impl Parse for Immediate {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::immediate);
        fn parse_int(s: &str) -> Result<Immediate, Err> {
            if s.starts_with("0x") {
                Ok(Immediate(i64::from_str_radix(&s[2..], 16)?))
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
                    registers.extend(list.regs);
                }
                _ => unreachable!("Invalid register list"),
            });
        registers.shrink_to_fit();
        Ok(RegisterList { regs: registers })
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
impl Parse for Opcode {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::opcode);
        let opcode = OPCODES.get(pair.as_str()).ok_or(Err::InvalidOpcode)?;
        Ok(Opcode(opcode.0))
    }
}

impl Parse for Instruction {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::operation);
        let mut inner = pair.into_inner();
        let opcode = Opcode::parse(inner.next().expect("No inner pair"))?;
        let operands = inner
            .map(|pair| -> Result<Operand, Err> { Ok(Operand::parse(pair)?) })
            .collect::<Result<Vec<Operand>, Err>>()?;
        Ok(Instruction { opcode, operands })
    }
}

impl RegisterRange {
    fn to_reg_list(&self) -> Result<RegisterList, Err> {
        let mut registers = Vec::with_capacity(16);
        if self.start.reg_type != self.end.reg_type {
            return Err(Err::InvalidRegisterRange);
        }
        let reg_type = self.start.reg_type;
        for i in self.start.reg_num..=self.end.reg_num {
            registers.push(Register::new(reg_type, i));
        }
        Ok(RegisterList { regs: registers })
    }
}
