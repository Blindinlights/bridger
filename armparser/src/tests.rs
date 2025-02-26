use crate::{ARM64Parser, Rule, *};
use expect_test::{expect, Expect};
use pest::Parser;
use register::*;
use std::fmt::Debug;
fn parse_format<'a, N: Parse<'a> + Debug>(input: &[&'a str], rule: Rule, expect: Expect) {
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
fn check_regs(input: &[&str], expect: &[Register]) {
    let res = input
        .into_iter()
        .map(|l| {
            let parsed = ARM64Parser::parse(Rule::register, l)
                .unwrap()
                .next()
                .unwrap();
            let parsed = Register::parse(parsed).unwrap();
            parsed
        })
        .collect::<Vec<Register>>();
    assert_eq!(res, expect);
}
fn parse_src(src: &str) {
    let res = parse_asm(src).unwrap();
    res.into_iter().for_each(|l| {
        println!("{:?}\n", l);
    });
}
#[test]
fn general_register() {
    check_regs(
        &[
            "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12",
        ],
        &[
            Register::X0,
            Register::X1,
            Register::X2,
            Register::X3,
            Register::X4,
            Register::X5,
            Register::X6,
            Register::X7,
            Register::X8,
            Register::X9,
            Register::X10,
            Register::X11,
            Register::X12,
        ],
    );
}
#[test]
fn float_register() {
    check_regs(
        &[
            "d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "d10", "d11", "d12",
        ],
        &[
            Register::D0,
            Register::D1,
            Register::D2,
            Register::D3,
            Register::D4,
            Register::D5,
            Register::D6,
            Register::D7,
            Register::D8,
            Register::D9,
            Register::D10,
            Register::D11,
            Register::D12,
        ],
    );
}
#[test]
fn immediate() {
    parse_format::<Immediate>(
        &["#0", "#0x0", "-1", "0x12345678", "0xABCDEF"],
        Rule::immediate,
        expect![[r#"
            Immediate(0)
            Immediate(0)
            Immediate(-1)
            Immediate(305419896)
            Immediate(11259375)
        "#]],
    );
}
#[test]
fn shift_type() {
    parse_format::<ShiftType>(
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
    parse_format::<ShiftedRegister>(
        &[
            "x0 lsl #0",
            "w0 lsr #0",
            "x0 asr #0",
            "w0 ror #0",
            "x0 rrx x0",
        ],
        Rule::shifted_register,
        expect![[r#"
            ShiftedRegister { reg: Register { reg_type: Full, reg_num: 0 }, shift_type: LSL, shift_amount: Some(Immediate(Immediate(0))) }
            ShiftedRegister { reg: Register { reg_type: Half, reg_num: 0 }, shift_type: LSR, shift_amount: Some(Immediate(Immediate(0))) }
            ShiftedRegister { reg: Register { reg_type: Full, reg_num: 0 }, shift_type: ASR, shift_amount: Some(Immediate(Immediate(0))) }
            ShiftedRegister { reg: Register { reg_type: Half, reg_num: 0 }, shift_type: ROR, shift_amount: Some(Immediate(Immediate(0))) }
            ShiftedRegister { reg: Register { reg_type: Full, reg_num: 0 }, shift_type: RRX, shift_amount: Some(Register(Register { reg_type: Full, reg_num: 0 })) }
            "#]],
    );
}
#[test]
fn indirect() {
    parse_format::<Indirect>(
        &[
            "[x0]",
            "[x0, #0]",
            "[x0, w0]",
            "[x0, w0]!",
            "[x0, w0 lsl #0]",
            "[x0,:got_lo12:__stack_chk_guard]",
        ],
        Rule::indirect,
        expect![[r#"
            Indirect { base: Register { reg_type: Full, reg_num: 0 }, offset: None, writeback: false }
            Indirect { base: Register { reg_type: Full, reg_num: 0 }, offset: Some(Immediate(Immediate(0))), writeback: false }
            Indirect { base: Register { reg_type: Full, reg_num: 0 }, offset: Some(Register(Register { reg_type: Half, reg_num: 0 })), writeback: false }
            Indirect { base: Register { reg_type: Full, reg_num: 0 }, offset: Some(Register(Register { reg_type: Half, reg_num: 0 })), writeback: true }
            Indirect { base: Register { reg_type: Full, reg_num: 0 }, offset: Some(ShiftedRegister(ShiftedRegister { reg: Register { reg_type: Half, reg_num: 0 }, shift_type: LSL, shift_amount: Some(Immediate(Immediate(0))) })), writeback: false }
            Indirect { base: Register { reg_type: Full, reg_num: 0 }, offset: Some(ProcLoad(ProcLoad { mode: "got_lo12", target: "__stack_chk_guard" })), writeback: false }
            "#]],
    );
}

fn check_reg_list(input: &[&str], expect: &[&[Register]]) {
    let res = input
        .into_iter()
        .map(|l| {
            let parsed = ARM64Parser::parse(Rule::reglist, l)
                .unwrap()
                .next()
                .unwrap();

            let res = RegisterList::parse(parsed).unwrap().regs;
            res
        })
        .collect::<Vec<Vec<Register>>>();
    assert_eq!(res, expect);
}
#[test]
fn reg_list() {
    check_reg_list(
        &[
            "{x0}",
            "{x0, x1}",
            "{x0, x1, x2}",
            "{x0, x1, x2, x3}",
            "{x0-x3}",
            "{x0-x3,x4}",
        ],
        &[
            &[Register::X0],
            &[Register::X0, Register::X1],
            &[Register::X0, Register::X1, Register::X2],
            &[Register::X0, Register::X1, Register::X2, Register::X3],
            &[Register::X0, Register::X1, Register::X2, Register::X3],
            &[
                Register::X0,
                Register::X1,
                Register::X2,
                Register::X3,
                Register::X4,
            ],
        ],
    );
}

#[test]
fn opcode() {
    parse_format::<Opcode>(
        &["add", "sub", "mul", "div", "mov", "load", "store", "jmp"],
        Rule::opcode,
        expect![[r#"
            Opcode("add")
            Opcode("sub")
            Opcode("mul")
            Opcode("div")
            Opcode("mov")
            Opcode("load")
            Opcode("store")
            Opcode("jmp")
        "#]],
    );
}
#[test]
fn instruction() {
    parse_format::<Instruction>(
        &[
            "add x0, x1, x2",
            "stp x29, x30, [sp, -48]!",
            "add x0, x0, :lo12:.LC2",
            "bl	puts",
            "li	x10, 0",
            "ldp x29, x30, [sp], 16",
        ],
        Rule::operation,
        expect![[r#"
            Instruction { opcode: Opcode("add"), operands: [Register(Register { reg_type: Full, reg_num: 0 }), Register(Register { reg_type: Full, reg_num: 1 }), Register(Register { reg_type: Full, reg_num: 2 })] }
            Instruction { opcode: Opcode("stp"), operands: [Register(Register { reg_type: Full, reg_num: 29 }), Register(Register { reg_type: Full, reg_num: 30 }), Indirect(Indirect { base: Register { reg_type: Full, reg_num: 31 }, offset: Some(Immediate(Immediate(-48))), writeback: true })] }
            "#]],
    );
}
#[test]
fn parse_asms() {
    let input = r#"add x0,x1,x3
        std x29,x30, [sp,-48]!
        add x0,x0,:lo12:.LC2"#;
    parse_src(&input);
}
