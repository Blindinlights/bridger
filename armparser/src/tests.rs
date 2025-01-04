use std::fmt::Debug;

use crate::{ARM64Parser, Rule, *};
use expect_test::{expect, Expect};
use pest::Parser;
fn parse_fomat<N: Parse + Debug>(input: &[&str], rule: Rule, expect: Expect) {
    let res = input
        .into_iter()
        .map(|l| {
            let parsed = ARM64Parser::parse(rule, l).unwrap().next().unwrap();
            let parsed = N::parse(parsed).unwrap();
            format!("{:?}\n", parsed)
        })
        .collect::<String>();
    expect.assert_eq(&res);
}
#[test]
fn general_register() {
    parse_fomat::<Register>(
        &["x0", "w0"],
        Rule::register,
        expect![[r#"
            FullReg(0)
            HalfReg(0)
        "#]],
    );
}
#[test]
fn float_register() {
    parse_fomat::<Register>(
        &["d0", "s0"],
        Rule::register,
        expect![[r#"
            FloatReg(Double(0))
            FloatReg(Single(0))
        "#]],
    );
}
#[test]
fn immediate() {
    parse_fomat::<Immediate>(
        &["#0", "#0x0", "-1", "0x12345678", "0xABCDEF"],
        Rule::immediate,
        expect![[r#"
            Decimal(0)
            Hex(0)
            Decimal(-1)
            Hex(305419896)
            Hex(11259375)
        "#]],
    );
}
#[test]
fn shift_type() {
    parse_fomat::<ShiftType>(
        &["lsl", "lsr", "asr", "ror"],
        Rule::shift_type,
        expect![[r#"
            LSL
            LSR
            ASR
            ROR
        "#]],
    );
}
#[test]
fn shift_reg() {
    parse_fomat::<ShiftedRegister>(
        &[
            "x0 lsl #0",
            "w0 lsr #0",
            "x0 asr #0",
            "w0 ror #0",
            "x0 rrx x0",
        ],
        Rule::shifted_register,
        expect![[r#"
            ShiftedRegister { reg: FullReg(0), shift_type: LSL, shift_amount: Some(Immediate(Decimal(0))) }
            ShiftedRegister { reg: HalfReg(0), shift_type: LSR, shift_amount: Some(Immediate(Decimal(0))) }
            ShiftedRegister { reg: FullReg(0), shift_type: ASR, shift_amount: Some(Immediate(Decimal(0))) }
            ShiftedRegister { reg: HalfReg(0), shift_type: ROR, shift_amount: Some(Immediate(Decimal(0))) }
            ShiftedRegister { reg: FullReg(0), shift_type: RRX, shift_amount: Some(Register(FullReg(0))) }
            "#]],
    );
}
#[test]
fn indirect() {
    parse_fomat::<Indirect>(
        &[
            "[x0]",
            "[x0, #0]",
            "[x0, w0]",
            "[x0, w0 lsl #0]",
            "[x0,:got_lo12:__stack_chk_guard]",
        ],
        Rule::indirect,
        expect![[r#"
            Indirect { base: FullReg(0), offset: None, writeback: false }
            Indirect { base: FullReg(0), offset: Some(Immediate(Decimal(0))), writeback: false }
            Indirect { base: FullReg(0), offset: Some(Register(HalfReg(0))), writeback: false }
            Indirect { base: FullReg(0), offset: Some(ShiftedRegister(ShiftedRegister { reg: HalfReg(0), shift_type: LSL, shift_amount: Some(Immediate(Decimal(0))) })), writeback: false }
            Indirect { base: FullReg(0), offset: Some(ProcLoad(ProcLoad { mode: "got_lo12", target: "__stack_chk_guard" })), writeback: false }
            "#]],
    );
}

#[test]
fn reg_list() {
    parse_fomat::<RegisterList>(
        &[
            "{x0}",
            "{x0, x1}",
            "{x0, x1, x2}",
            "{x0, x1, x2, x3}",
            "{x0-x5}",
            "{x0-x3,x7}",
        ],
        Rule::reglist,
        expect![[r#"
            RegisterList { registers: [FullReg(0)] }
            RegisterList { registers: [FullReg(0), FullReg(1)] }
            RegisterList { registers: [FullReg(0), FullReg(1), FullReg(2)] }
            RegisterList { registers: [FullReg(0), FullReg(1), FullReg(2), FullReg(3)] }
            RegisterList { registers: [FullReg(0), FullReg(1), FullReg(2), FullReg(3), FullReg(4), FullReg(5)] }
            RegisterList { registers: [FullReg(0), FullReg(1), FullReg(2), FullReg(3), FullReg(7)] }
        "#]],
    );
}
