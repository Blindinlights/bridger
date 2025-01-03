use crate::{ARM64Parser, Rule};
use pest::Parser;

#[test]
fn simple_test() {
    let input = "mov x0, 0x23\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}

#[test]
fn shift_register_test() {
    let input = "mov x0, x1, x0 lsl #2\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}

#[test]
fn register_test() {
    let input = "mov x0, x1\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}

#[test]
fn label_target_test() {
    let input = "b label\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
#[test]
fn inderict_addressing_test() {
    let input = "ldr x0, [x1]\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
#[test]
fn reg_list_test() {
    let input = "push {x0, x1, x2}\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
#[test]
fn reg_list_range_test() {
    let input = "push {x0-x2}\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
#[test]
fn reg_list_range_with_sp_test() {
    let input = "push {x0-x2, sp}\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}

#[test]
fn indirect_test() {
    let input = "ldr x0, [x1, #4]\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
#[test]
fn indirect_writeback_test() {
    let input = "ldr x0, [x1, #4]!\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
#[test]
fn dot_label_test() {
    let input = "b .label\n";
    let pairs = ARM64Parser::parse(Rule::line, input).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        println!("{:#?}", pair);
    }
}
