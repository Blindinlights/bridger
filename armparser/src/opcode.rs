use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! define_opcodes {
    ($($name:ident = $value:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Opcode(pub &'static str);

        impl Opcode {
            $(
                pub const $name: Opcode = Opcode($value);
            )*

            pub fn name(&self) -> &'static str {
                self.0
            }
        }

        // 创建全局静态HashMap
        pub static OPCODES: LazyLock<HashMap<&'static str, Opcode>> = LazyLock::new(|| {
            let mut map = HashMap::new();
            $(
                map.insert($value, Opcode::$name);
            )*
            map
        });
    };
}

define_opcodes! {
    ADD = "add",
    SUB = "sub",
    MUL = "mul",
    DIV = "div",
    MOV = "mov",
    LOAD = "load",
    STORE = "store",
    JMP = "jmp",

    STP = "stp",

    B = "b",
    BL = "bl",

    LI = "li",
    LDR = "ldr",
    STR = "str",
    LDP = "ldp",
}
