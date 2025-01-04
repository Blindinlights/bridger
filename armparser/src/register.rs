use crate::{Parse, Rule};

#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    pub reg_type: RegisterType,
    pub reg_num: u8,
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RegisterType {
    Half,
    Full,
    HalfFloat,
    SingleFloat,
    DoubleFloat,
    StackPointer,
    XZR,
    WZR,
}
impl Register {
    pub const fn new(reg_type: RegisterType, reg_num: u8) -> Self {
        Self { reg_type, reg_num }
    }
}
impl Parse for Register {
    fn parse(pair: pest::iterators::Pair<crate::Rule>) -> Result<Self, crate::Err>
    where
        Self: Sized,
    {
        debug_assert_eq!(pair.as_rule(), crate::Rule::register);
        let inner = pair.into_inner().next().expect("register inner");
        match inner.as_rule() {
            Rule::full_reg => match inner.as_str() {
                "zxr" => Ok(XZR),
                "sp" => Ok(SP),
                "fp" => Ok(FP),
                "lr" => Ok(LR),
                _ => {
                    let reg_num = inner.as_str()[1..].parse().unwrap();
                    Ok(Register::new(RegisterType::Full, reg_num))
                }
            },
            Rule::half_reg => match inner.as_str() {
                "wzr" => Ok(WZR),
                _ => {
                    let reg_num = inner.as_str()[1..].parse().unwrap();
                    Ok(Register::new(RegisterType::Half, reg_num))
                }
            },
            Rule::float_reg => {
                let inner = inner.into_inner().next().expect("float reg inner");
                match inner.as_rule() {
                    Rule::float16 => {
                        let reg_num = inner.as_str()[1..].parse().unwrap();
                        Ok(Register::new(RegisterType::HalfFloat, reg_num))
                    }
                    Rule::float32 => {
                        let reg_num = inner.as_str()[1..].parse().unwrap();
                        Ok(Register::new(RegisterType::SingleFloat, reg_num))
                    }
                    Rule::double64 => {
                        let reg_num = inner.as_str()[1..].parse().unwrap();
                        Ok(Register::new(RegisterType::DoubleFloat, reg_num))
                    }
                    _ => unreachable!("float reg inner"),
                }
            }
            _ => unreachable!("register inner"),
        }
    }
}

pub const FP: Register = Register::new(RegisterType::Full, 29);
pub const LR: Register = Register::new(RegisterType::Full, 30);
pub const SP: Register = Register::new(RegisterType::StackPointer, 31);
pub const WZR: Register = Register::new(RegisterType::WZR, 31);
pub const XZR: Register = Register::new(RegisterType::XZR, 31);
macro_rules! build_regs {
    ($($num:expr,$name:ident,$reg_type:ident ),*) => {
        impl Register {
            $(
                pub const $name: Register = Register::new(RegisterType::$reg_type, $num);
            )*
        }
    };
}
macro_rules! build_full_regs {

    ($($num:expr => $name:ident),*) => {


        $(
            build_regs!($num,$name,Full);
        )*
    };
}

macro_rules! build_half_regs {
    ($($num:expr => $name:ident),*) => {
        $(
            build_regs!($num,$name,Half);
        )*
    };
}

macro_rules! build_half_float_regs {
    ($($num:expr => $name:ident),*) => {
        $(
            build_regs!($num,$name,HalfFloat);
        )*
    };
}

macro_rules! build_single_float_regs {
    ($($num:expr => $name:ident),*) => {
        $(
            build_regs!($num,$name,SingleFloat);
        )*
    };
}

macro_rules! build_double_float_regs {
    ($($num:expr => $name:ident),*) => {
        $(
            build_regs!($num,$name,DoubleFloat);
        )*
    };
}

build_full_regs! {
    0 => X0,
    1 => X1,
    2 => X2,
    3 => X3,
    4 => X4,
    5 => X5,
    6 => X6,
    7 => X7,
    8 => X8,
    9 => X9,
    10 => X10,
    11 => X11,
    12 => X12,
    13 => X13,
    14 => X14,
    15 => X15,
    16 => X16,
    17 => X17,
    18 => X18,
    19 => X19,
    20 => X20,
    21 => X21,
    22 => X22,
    23 => X23,
    24 => X24,
    25 => X25,
    26 => X26,
    27 => X27,
    28 => X28,
    29 => X29,
    30 => X30,
    31 => X31
}
build_half_regs! {
    0 => W0,
    1 => W1,
    2 => W2,
    3 => W3,
    4 => W4,
    5 => W5,
    6 => W6,
    7 => W7,
    8 => W8,
    9 => W9,
    10 => W10,
    11 => W11,
    12 => W12,
    13 => W13,
    14 => W14,
    15 => W15,
    16 => W16,
    17 => W17,
    18 => W18,
    19 => W19,
    20 => W20,
    21 => W21,
    22 => W22,
    23 => W23,
    24 => W24,
    25 => W25,
    26 => W26,
    27 => W27,
    28 => W28,
    29 => W29,
    30 => W30,
    31 => W31
}

build_half_float_regs! {
    0 => H0,
    1 => H1,
    2 => H2,
    3 => H3,
    4 => H4,
    5 => H5,
    6 => H6,
    7 => H7,
    8 => H8,
    9 => H9,
    10 => H10,
    11 => H11,
    12 => H12,
    13 => H13,
    14 => H14,
    15 => H15,
    16 => H16,
    17 => H17,
    18 => H18,
    19 => H19,
    20 => H20,
    21 => H21,
    22 => H22,
    23 => H23,
    24 => H24,
    25 => H25,
    26 => H26,
    27 => H27,
    28 => H28,
    29 => H29,
    30 => H30,
    31 => H31
}

build_single_float_regs! {
    0 => S0,
    1 => S1,
    2 => S2,
    3 => S3,
    4 => S4,
    5 => S5,
    6 => S6,
    7 => S7,
    8 => S8,
    9 => S9,
    10 => S10,
    11 => S11,
    12 => S12,
    13 => S13,
    14 => S14,
    15 => S15,
    16 => S16,
    17 => S17,
    18 => S18,
    19 => S19,
    20 => S20,
    21 => S21,
    22 => S22,
    23 => S23,
    24 => S24,
    25 => S25,
    26 => S26,
    27 => S27,
    28 => S28,
    29 => S29,
    30 => S30,
    31 => S31
}

build_double_float_regs! {
    0 => D0,
    1 => D1,
    2 => D2,
    3 => D3,
    4 => D4,
    5 => D5,
    6 => D6,
    7 => D7,
    8 => D8,
    9 => D9,
    10 => D10,
    11 => D11,
    12 => D12,
    13 => D13,
    14 => D14,
    15 => D15,
    16 => D16,
    17 => D17,
    18 => D18,
    19 => D19,
    20 => D20,
    21 => D21,
    22 => D22,
    23 => D23,
    24 => D24,
    25 => D25,
    26 => D26,
    27 => D27,
    28 => D28,
    29 => D29,
    30 => D30,
    31 => D31
}
