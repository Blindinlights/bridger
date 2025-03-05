use arm64::Operand;
use asm_inst::*;
use either::Either;
pub fn basic_arm2riscv_reg_map(reg: &arm64::Register) -> riscv::Register {
    match reg {
        arm64::Register::General { ty, n } => match ty {
            arm64::General::X | arm64::General::W => riscv::Register::X(*n),
            arm64::General::S | arm64::General::D => riscv::Register::F(*n),
            _ => todo!(),
        },
        arm64::Register::Special(special) => match special {
            arm64::Special::Xzr | arm64::Special::Wzr => riscv::Register::X(0),
            arm64::Special::SP => riscv::Register::X(8),
            arm64::Special::LR => riscv::Register::X(1),
        },
    }
}

fn map_operand_to_reg(op: &Option<Operand>) -> Option<riscv::Register> {
    op.as_ref()
        .and_then(|o| match o {
            Operand::Reg(regoperand) => match regoperand {
                arm64::Regoperand::Reg(register) => Some(register),
                _ => None,
            },
            Operand::Imm { .. } => None,
            Operand::Addressing(_) => None,
            Operand::Label(_) => None,
        })
        .map(basic_arm2riscv_reg_map)
}

pub struct Arm64Translator {
    pub inst: arm64::Instruction,
    pub opcode: arm64::Opcode,
    pub rd: Option<arm64::Regoperand>,
    pub rs1: Option<arm64::Operand>,
    pub rs2: Option<arm64::Operand>,
    pub rs3: Option<arm64::Operand>,
    pub label: Option<String>,
    pub iflag: bool,    // immediate
    pub wflag: bool,    // halfword
    pub fp_wflag: bool, // floating point halfword
    pub riscv_inst: Vec<riscv::Instruction>,
    pub imm_temp_rsg: Option<riscv::Register>,
}
impl Arm64Translator {
    pub fn new(inst: arm64::Instruction) -> Self {
        let opcode = inst.opcode.clone();
        let mut riscv_inst = Vec::new();
        let rd = inst.operand.get(0).cloned().and_then(|x| match x {
            Operand::Reg(reg) => Some(reg),
            _ => None,
        });
        let rs1 = inst.operand.get(1).cloned();
        let rs2 = inst.operand.get(2).cloned();
        let temp_reg = rs2.as_ref().and_then(|o| match o {
            Operand::Reg(_) => None,
            Operand::Imm { imm, shift } => check_imm(&mut riscv_inst, *imm, shift.clone()),
            Operand::Addressing(_) => None,
            Operand::Label(_) => None,
        });
        let rs3 = inst.operand.get(3).cloned();

        let label = match inst.operand.get(0) {
            Some(arm64::Operand::Label(ref l)) => Some(l.to_string()),
            _ => None,
        };
        let iflag = inst.operand.iter().any(|x| match x {
            arm64::Operand::Imm { .. } => true,
            _ => false,
        }) && temp_reg.is_none();

        let wflag = inst.operand.iter().all(|x| match x {
            arm64::Operand::Reg(reg) => reg.is_word(),
            _ => false,
        });
        let fp_wflag = inst.operand.iter().all(|x| match x {
            arm64::Operand::Reg(reg) => reg.is_fword(),
            _ => false,
        });

        Self {
            inst,
            opcode,
            rd,
            rs1,
            rs2,
            rs3,
            label,
            iflag,
            wflag,
            fp_wflag,
            riscv_inst,
            imm_temp_rsg: temp_reg,
        }
    }
    fn map_rd(&self) -> Option<riscv::Register> {
        self.rd
            .as_ref()
            .and_then(|ro| match ro {
                arm64::Regoperand::Reg(register) => Some(register),
                _ => None,
            })
            .map(basic_arm2riscv_reg_map)
    }
    fn map_rs1(&self) -> Option<riscv::Register> {
        map_operand_to_reg(&self.rs1)
    }
    fn map_rs2(&self) -> Option<riscv::Register> {
        map_operand_to_reg(&self.rs2)
    }
    fn map_rs3(&self) -> Option<riscv::Register> {
        map_operand_to_reg(&self.rs3)
    }
    fn map_rs2_reg(&self) -> Option<arm64::Regoperand> {
        self.rs2.as_ref().and_then(|o| match o {
            Operand::Reg(reg) => Some(reg.clone()),
            _ => None,
        })
    }
    fn map_rs3_reg(&self) -> Option<arm64::Regoperand> {
        self.rs3.as_ref().and_then(|o| match o {
            Operand::Reg(reg) => Some(reg.clone()),
            _ => None,
        })
    }

    fn rs2_as_imm(&self) -> u16 {
        let imm = self
            .rs2
            .clone()
            .and_then(|o| match o {
                Operand::Imm { imm, .. } => Some(imm),
                _ => None,
            })
            .unwrap();
        imm
    }

    /// `add` instruction
    /// - add immediate
    /// - add shift register
    /// - add extended register(todo)
    pub fn add(&self, res: &mut Vec<riscv::Instruction>) {
        let opcode = match (self.iflag, self.wflag) {
            (true, true) => riscv::Opcode::Addiw,
            (true, false) => riscv::Opcode::Addi,
            (false, true) => riscv::Opcode::Addw,
            (false, false) => riscv::Opcode::Add,
        };
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        if self.iflag {
            let imm = self.rs2_as_imm();
            let imm = riscv::Immediate::Number(imm as i16);
            res.push(riscv::Instruction::new_i(opcode, rd, rs1, imm.clone()));
            if self.opcode == arm64::Opcode::Adds {
                emit_set_flag(res, rd, rs1, Either::Right(imm));
            }
        } else {
            let rs2 = if let Some(reg) = self.imm_temp_rsg {
                reg
            } else {
                let rs2 = self.map_rs2_reg().unwrap();
                emit_shift_or_extend(res, &rs2)
            };

            res.push(riscv::Instruction::new_r(opcode, rd, rs1, rs2));
            if self.opcode == arm64::Opcode::Adds {
                emit_set_flag(res, rd, rs1, Either::Left(rs2));
            }
        }
    }

    pub fn sub(&self, res: &mut Vec<riscv::Instruction>) {
        let opcode = match (self.iflag, self.wflag) {
            (true, true) => riscv::Opcode::Addiw,
            (true, false) => riscv::Opcode::Addi,
            (false, true) => riscv::Opcode::Subw,
            (false, false) => riscv::Opcode::Sub,
        };
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        if self.iflag {
            let imm = self.rs2_as_imm();
            let imm = riscv::Immediate::Number(-(imm as i16));
            res.push(riscv::Instruction::new_i(opcode, rd, rs1, imm.clone()));
        } else {
            let rs2 = if let Some(reg) = self.imm_temp_rsg {
                reg
            } else {
                let rs2 = self.map_rs2_reg().unwrap();
                emit_shift_or_extend(res, &rs2)
            };
            res.push(riscv::Instruction::new_r(opcode, rd, rs1, rs2));
            if self.opcode == arm64::Opcode::Subs {
                emit_set_flag(res, rd, rs1, Either::Left(rs2));
                res.push(riscv::Instruction::new_i(
                    riscv::Opcode::Xori,
                    riscv::Register::T5,
                    riscv::Register::T5,
                    riscv::Immediate::Number(1),
                ));
            }
        }
    }
    pub fn madd(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        let rs2 = self.map_rs2().unwrap();
        let rs3 = self.map_rs3().unwrap();

        res.push(riscv::Instruction::new_r(riscv::Opcode::Mul, rd, rs1, rs2));
        res.push(riscv::Instruction::new_r(riscv::Opcode::Add, rd, rd, rs3));
    }

    pub fn mul(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        let rs2 = self.map_rs2().unwrap();
        res.push(riscv::Instruction::new_r(riscv::Opcode::Mul, rd, rs1, rs2));
    }

    pub fn sdiv(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        let rs2 = self.map_rs2().unwrap();
        res.push(riscv::Instruction::new_r(riscv::Opcode::Div, rd, rs1, rs2));
    }
    pub fn udiv(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        let rs2 = self.map_rs2().unwrap();
        res.push(riscv::Instruction::new_r(riscv::Opcode::Divu, rd, rs1, rs2));
    }

    pub fn mov(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        let rs1 = self.rs1.as_ref().unwrap();
        match rs1 {
            Operand::Reg(regoperand) => {
                let rs1 = emit_shift_or_extend(res, regoperand);
                res.push(riscv::Instruction::new_r(
                    riscv::Opcode::Add,
                    rd,
                    riscv::Register::ZERO,
                    rs1,
                ));
            }
            Operand::Imm { imm, shift } => {
                let imm = riscv::Immediate::Number(*imm as i16);
                res.push(riscv::Instruction::new_i(
                    riscv::Opcode::Addi,
                    rd,
                    riscv::Register::ZERO,
                    imm,
                ));
            }

            Operand::Label(s) => {
                res.push(riscv::Instruction::new_i(
                    riscv::Opcode::Lui,
                    rd,
                    riscv::Register::ZERO,
                    riscv::Immediate::Label(s.clone()),
                ));
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub fn mov_not(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        self.mov(res);
        //按位取反
        res.push(riscv::Instruction::new_i(
            riscv::Opcode::Xori,
            rd,
            rd,
            riscv::Immediate::Number(-1),
        ));
    }

    pub fn mov_zero(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        self.mov(res);
    }
    pub fn mov_keep(&self, res: &mut Vec<riscv::Instruction>) {
        unimplemented!()
    }

    pub fn branch(&self, res: &mut Vec<riscv::Instruction>) {
        let label = self.label.as_ref().unwrap();
        res.push(riscv::Instruction::new_u(
            riscv::Opcode::Jal,
            riscv::Register::ZERO,
            riscv::Immediate::Label(label.clone()),
        ));
    }

    pub fn shift(&self, res: &mut Vec<riscv::Instruction>) {
        let rd = self.map_rd().unwrap();
        let rs1 = self.map_rs1().unwrap();
        let rs2 = self.rs2.as_ref().unwrap();
        let opcode = match self.opcode {
            arm64::Opcode::Lsl => riscv::Opcode::Sll,
            arm64::Opcode::Lsr => riscv::Opcode::Srl,
            arm64::Opcode::Asr => riscv::Opcode::Sra,
            _ => unreachable!(),
        };

        match rs2 {
            Operand::Imm { imm, .. } => {
                let imm = *imm;
                res.push(riscv::Instruction::new_i(
                    opcode.to_imm(),
                    rd,
                    rs1,
                    riscv::Immediate::Number(imm as i16),
                ));
            }
            Operand::Reg(reg) => {
                let rs2 = emit_shift_or_extend(res, reg);
                res.push(riscv::Instruction::new_r(opcode, rd, rs1, rs2));
            }
            _ => {
                unreachable!()
            }
        }
    }
}

fn shift_imm(
    res: &mut Vec<riscv::Instruction>,
    imm: u16,
    shift: Option<(u8, arm64::Shift)>,
) -> Either<u16, riscv::Register> {
    todo!()
}
pub fn check_imm(
    res: &mut Vec<riscv::Instruction>,
    imm: u16,
    shift: Option<(u8, arm64::Shift)>,
) -> Option<riscv::Register> {
    if imm > 0b111111111111 {
        let rd = riscv::Register::T0;
        let imm = riscv::Immediate::Number(imm as i16);
        res.push(riscv::Instruction::new_i(
            riscv::Opcode::Lui,
            rd,
            riscv::Register::ZERO,
            imm,
        ));
        if let Some((s, sh)) = shift {
            emit_shift_reg(res, rd, (s, sh));
        }

        Some(rd)
    } else {
        None
    }
}

pub fn emit_shift_reg(
    res: &mut Vec<riscv::Instruction>,
    reg: riscv::Register,
    (shift, shift_ty): (u8, arm64::Shift),
) {
    let opcode = match shift_ty {
        arm64::Shift::Lsl => riscv::Opcode::Slli,
        arm64::Shift::Lsr => riscv::Opcode::Srli,
        arm64::Shift::Asr => riscv::Opcode::Srai,
        arm64::Shift::Ror => todo!(),
        arm64::Shift::Uxtb => riscv::Opcode::Andi,
    };
    let imm = riscv::Immediate::Number(shift.into());
    res.push(riscv::Instruction::new_i(opcode, reg, reg, imm));
}
pub fn emit_extend_reg(
    res: &mut Vec<riscv::Instruction>,
    reg: riscv::Register,
    (extend, extend_ty): (u8, arm64::Extend),
) {
    let opcode = match extend_ty {
        arm64::Extend::Uxtb => riscv::Opcode::Andi,
        arm64::Extend::Uxth => riscv::Opcode::Andi,
        arm64::Extend::Uxtw => riscv::Opcode::Andi,
        arm64::Extend::Lsl => riscv::Opcode::Andi,
        arm64::Extend::Uxtx => riscv::Opcode::Andi,
        arm64::Extend::Sxtb => riscv::Opcode::Addiw,
        arm64::Extend::Sxth => riscv::Opcode::Addiw,
        arm64::Extend::Sxtw => riscv::Opcode::Addiw,
        arm64::Extend::Sxtx => riscv::Opcode::Addiw,
    };
    let imm = match opcode {
        riscv::Opcode::Andi => riscv::Immediate::Number(0xff),
        riscv::Opcode::Addiw => riscv::Immediate::Number(0),
        _ => unreachable!(),
    };
    let inst1 = riscv::Instruction::new_i(opcode, reg, reg, imm);
    let inst2 = riscv::Instruction::new_i(
        riscv::Opcode::Slli,
        reg,
        reg,
        riscv::Immediate::Number(extend.into()),
    );
    res.push(inst1);
    res.push(inst2);
}
pub fn emit_set_flag(
    res: &mut Vec<riscv::Instruction>,
    rd: riscv::Register,
    rs1: riscv::Register,
    rm: Either<riscv::Register, riscv::Immediate>,
) {
    emit_set_nz_flag(res, rd);
    // set carry flag
    res.push(riscv::Instruction::new_r(
        riscv::Opcode::Sltu,
        riscv::Register::T5,
        rd,
        rs1,
    ));
    // set overflow flag
    // add: 检查输入符号相同但结果符号不同
    // sub: 检查输入符号不同但结果符号相同
}

pub fn emit_set_nz_flag(res: &mut Vec<riscv::Instruction>, rd: riscv::Register) {
    // set zero flag
    res.push(riscv::Instruction::new_i(
        riscv::Opcode::Sltiu,
        riscv::Register::T3,
        rd,
        riscv::Immediate::Number(1),
    ));
    // set negative flag
    res.push(riscv::Instruction::new_r(
        riscv::Opcode::Slt,
        riscv::Register::T4,
        rd,
        riscv::Register::ZERO,
    ));
}
pub fn emit_shift_or_extend(
    res: &mut Vec<riscv::Instruction>,
    reg: &arm64::Regoperand,
) -> riscv::Register {
    match reg {
        arm64::Regoperand::ShiftReg(register, x) => {
            let riscv_rg = basic_arm2riscv_reg_map(&register);
            emit_shift_reg(res, riscv_rg, x.clone());
            riscv_rg
        }
        arm64::Regoperand::ExtendReg(register, x) => {
            let riscv_rg = basic_arm2riscv_reg_map(&register);
            emit_extend_reg(res, riscv_rg, x.clone());
            riscv_rg
        }
        arm64::Regoperand::Reg(register) => basic_arm2riscv_reg_map(&register),
    }
}
