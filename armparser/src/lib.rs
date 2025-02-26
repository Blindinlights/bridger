pub mod error;
pub mod opcode;
pub mod register;
#[cfg(test)]
pub mod tests;
use error::ArmParserError;
use opcode::{Opcode, OPCODES};
use pest::Parser;
use pest_derive::Parser;
use register::Register;

pub fn parse_asm<'i>(src: &'i str) -> Result<Vec<Line<'i>>, ArmParserError> {
    let res = ARM64Parser::parse(Rule::line, &src)?;
    let res = res.map(|p| {
        println!("{}\n", p.as_str());
    });
    todo!();
}

#[derive(Parser)]
#[grammar = "arm64.pest"] // 使用前面定义的pest语法文件
pub struct ARM64Parser;

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
pub struct Indirect<'a> {
    base: Register,
    offset: Option<Offset<'a>>,
    writeback: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Offset<'a> {
    Immediate(Immediate),
    Register(Register),
    ShiftedRegister(ShiftedRegister),
    ProcLoad(ProcLoad<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcLoad<'a> {
    mode: &'a str,
    target: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand<'a> {
    Register(Register),
    Immediate(Immediate),
    Address(Immediate),
    LabelTarget(&'a str),
    Indirect(Indirect<'a>),
    RegisterList(RegisterList),
    ShiftedRegister(ShiftedRegister),
    ProcLoad(ProcLoad<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction<'a> {
    opcode: Opcode,
    operands: Vec<Operand<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line<'a> {
    Label(&'a str),
    Directive(&'a str),
    Instruction(Instruction<'a>),
}
type Err = crate::error::ArmParserError;
impl ARM64Parser {}

pub trait Parse<'a> {
    fn parse(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Err>
    where
        Self: Sized;
}

impl Parse<'_> for Immediate {
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

impl Parse<'_> for ShiftType {
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

impl Parse<'_> for ShiftedRegister {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::shifted_register);
        let mut inner = pair.into_inner();
        let reg = Register::parse(inner.next().expect("No inner pair"))?;
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
            reg,
            shift_type,
            shift_amount,
        })
    }
}

impl<'a> Parse<'a> for Offset<'a> {
    fn parse(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Err> {
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

impl<'a> Parse<'a> for Indirect<'a> {
    fn parse(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Err> {
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
impl<'a> Parse<'a> for ProcLoad<'a> {
    fn parse(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::proc_load);
        let mut inner = pair.into_inner();
        let mode = inner.next().expect("No inner pair").as_str();
        let target = inner.next().expect("No inner pair").as_str();
        Ok(ProcLoad { mode, target })
    }
}

impl Parse<'_> for RegisterList {
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

impl Parse<'_> for RegisterRange {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::register_range);
        let mut inner = pair.into_inner();
        let start = Register::parse(inner.next().expect("No inner pair"))?;
        let end = Register::parse(inner.next().expect("No inner pair"))?;
        Ok(RegisterRange { start, end })
    }
}

impl<'a> Parse<'a> for Operand<'a> {
    fn parse(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::operand);
        let inner = pair.into_inner().next().expect("No inner pair");
        match inner.as_rule() {
            Rule::register => Ok(Operand::Register(Register::parse(inner)?)),
            Rule::immediate => Ok(Operand::Immediate(Immediate::parse(inner)?)),
            Rule::address => Ok(Operand::Address(Immediate::parse(inner)?)),
            Rule::label_target => Ok(Operand::LabelTarget(inner.as_str())),
            Rule::indirect => Ok(Operand::Indirect(Indirect::parse(inner)?)),
            Rule::reglist => Ok(Operand::RegisterList(RegisterList::parse(inner)?)),
            Rule::shifted_register => Ok(Operand::ShiftedRegister(ShiftedRegister::parse(inner)?)),
            Rule::proc_load => Ok(Operand::ProcLoad(ProcLoad::parse(inner)?)),
            _ => unreachable!("Invalid operand"),
        }
    }
}
impl Parse<'_> for Opcode {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::opcode);
        let opcode = OPCODES.get(pair.as_str()).ok_or(Err::InvalidOpcode)?;
        Ok(Opcode(opcode.0))
    }
}

impl<'a> Parse<'a> for Instruction<'a> {
    fn parse(pair: pest::iterators::Pair<'a, Rule>) -> Result<Self, Err> {
        debug_assert_eq!(pair.as_rule(), Rule::operation);
        let mut inner = pair.into_inner();
        let opcode = Opcode::parse(inner.next().expect("No inner pair"))?;
        let operands = inner
            .map(|pair| -> Result<Operand, Err> { Ok(Operand::parse(pair)?) })
            .collect::<Result<Vec<Operand>, Err>>()?;
        Ok(Instruction { opcode, operands })
    }
}

impl<'a, 'b> Parse<'b> for Line<'a>
where
    'b: 'a,
{
    fn parse(pair: pest::iterators::Pair<'b, Rule>) -> Result<Self, Err>
    where
        Self: Sized,
    {
        debug_assert_eq!(pair.as_rule(), Rule::line);
        let inner = pair.into_inner().next().expect("No inner pair");
        match inner.as_rule() {
            Rule::directive => Ok(Line::Directive(inner.as_str())),
            Rule::operation => Ok(Line::Instruction(Instruction::parse(inner)?)),
            Rule::label => Ok(Line::Label(inner.as_str())),
            r => {
                println!("{:?}", r);
                unreachable!("invalid Line")
            }
        }
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
