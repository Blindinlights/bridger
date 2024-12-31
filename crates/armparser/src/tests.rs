use expect_test::{expect, Expect};

use crate::lexer::tokenize;

fn check_lexing(src: &str, expect: Expect) {
    let actual: String = tokenize(src)
        .map(|token| format!("{:?}\n", token))
        .collect();
    expect.assert_eq(&actual)
}

#[test]
fn test_lexing() {
    check_lexing(
        "mov r0, r1",
        expect![[r#"
            Token { kind: Identifier, len: 3 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 2 }
            Token { kind: Comma, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 2 }
        "#]],
    );
}
#[test]
fn test_string_literal_1() {
    check_lexing(
        r#""hello, world!""#,
        expect![[r#"
            Token { kind: StringLiteral, len: 15 }
        "#]],
    );
}
#[test]
fn test_string_literal_2() {
    check_lexing(
        r#"
.text
.pushsection    .idmap.text, "a""#,
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
            Token { kind: Dot, len: 1 }
            Token { kind: Identifier, len: 4 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Dot, len: 1 }
            Token { kind: Identifier, len: 11 }
            Token { kind: Whitespace, len: 4 }
            Token { kind: Dot, len: 1 }
            Token { kind: Identifier, len: 5 }
            Token { kind: Dot, len: 1 }
            Token { kind: Identifier, len: 4 }
            Token { kind: Comma, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: StringLiteral, len: 3 }
        "#]],
    );
}

#[test]
fn test_number_literal() {
    check_lexing(
        r#"
1234
0x1234
0b1010
01234
"#,
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Int(Decimal)), len: 4 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Int(Hex)), len: 6 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Int(Binary)), len: 6 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Int(Octal)), len: 5 }
            Token { kind: Whitespace, len: 1 }
        "#]],
    );
}

#[test]
fn test_number_float_literal() {
    check_lexing(
        r#"
1234.5678
1.0e-10
0.1e10
0.1e+10
"#,
        expect![[r#"
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Float), len: 9 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Float), len: 7 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Float), len: 6 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Float), len: 7 }
            Token { kind: Whitespace, len: 1 }
        "#]],
    );
}

#[test]
fn test_char_literal() {
    check_lexing(
        "'a' ' ' '\\n'",
        expect![[r#"
            Token { kind: Literal(Char), len: 3 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Char), len: 3 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Literal(Char), len: 4 }
        "#]],
    );
}
