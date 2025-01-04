#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode(&'static str);
macro_rules! opcode {
    (
        $(#[$attr:meta])*
        $name:ident
    ) => {
        $(#[$attr])*
        pub const $name: Opcode = Opcode(stringify!($name));
    };
}
macro_rules! opcodes {
    (
        $(
            $(#[$attr:meta])*
            $name:ident
        ),*
    ) => {
        $(
            opcode! {
                $(#[$attr])*
                $name
            }
        )*
    };
}

opcodes! {
    ADD, SUB, MUL, DIV, MOD, AND, OR, XOR, NOT, SHL, SHR, ROL, ROR, CMP, TEST, MOV, LDR, STR, JMP, CALL, RET
}
