//! Instruction Set Implementation
use crate::cpu::{MemIter, Register};

pub trait ReadMem {
    type Item;

    fn read(iter: impl MemIter) -> Result<ParsedInstruction, String>;
}

pub struct ParsedInstruction {
    pub(crate) instr: Instruction,
    pub(crate) delta_ip: u32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    Move(Move),
    Halt,
}

impl ReadMem for Instruction {
    type Item = u8;

    fn read(mut iter: impl MemIter) -> Result<ParsedInstruction, String> {
        let group_value = iter.next8();

        Ok(match group_value {
            0x0 => ParsedInstruction {
                instr: Self::Halt,
                delta_ip: 1,
            },
            0x1 => {
                let parsed = Move::read(iter)?;

                ParsedInstruction {
                    instr: parsed.instr,
                    delta_ip: parsed.delta_ip + 1,
                }
            }

            _ => {
                return Err(format!(
                    "Should have gotten a valid group value, not {group_value:01x}"
                ));
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Move {
    RegToReg(Register, Register),

    RegToMem32(Register, u32),
    RegToMem16(Register, u32),
    RegToMem8(Register, u32),

    MemToReg32(u32, Register),
    MemToReg16(u32, Register),
    MemToReg8(u32, Register),

    MemToMem32(u32, u32),
    MemToMem16(u32, u32),
    MemToMem8(u32, u32),
}

impl ReadMem for Move {
    type Item = u8;

    fn read(mut iter: impl MemIter) -> Result<ParsedInstruction, String> {
        let move_group = iter.next8();

        match (move_group & 0xC0) >> 6 {
            0 => {
                let operand_src = iter.next8();
                let operand_dest = iter.next8();

                let reg_src = Register::try_from_id(operand_src)?;
                let reg_dst = Register::try_from_id(operand_dest)?;

                Ok(ParsedInstruction {
                    instr: Instruction::Move(Self::RegToReg(reg_src, reg_dst)),
                    delta_ip: iter.travelled() as u32,
                })
            }

            1 => {
                let reg_src = Register::try_from_id(iter.next8())?;
                let addr_dst = iter.next32();

                let move_instr = match (move_group & 0x30) >> 4 {
                    0 => Self::RegToMem8(reg_src, addr_dst),
                    1 => Self::RegToMem16(reg_src, addr_dst),
                    2 | 3 => Self::RegToMem32(reg_src, addr_dst),
                    _ => unreachable!("there only can be 4 possiblities"),
                };

                Ok(ParsedInstruction {
                    instr: Instruction::Move(move_instr),
                    delta_ip: iter.travelled() as u32,
                })
            }
            2 => {
                let addr_src = iter.next32();
                let reg_dst = Register::try_from_id(iter.next8())?;

                let move_instr = match (move_group & 0x30) >> 4 {
                    0 => Self::MemToReg8(addr_src, reg_dst),
                    1 => Self::MemToReg16(addr_src, reg_dst),
                    2 | 3 => Self::MemToReg32(addr_src, reg_dst),
                    _ => unreachable!("there can only be 4 possibilites"),
                };

                Ok(ParsedInstruction {
                    instr: Instruction::Move(move_instr),
                    delta_ip: iter.travelled() as u32,
                })
            }
            3 => {
                let addr_src = iter.next32();
                let addr_dst = iter.next32();

                let move_instr = match (move_group & 0x30) >> 4 {
                    0 => Self::MemToMem8(addr_src, addr_dst),
                    1 => Self::MemToMem16(addr_src, addr_dst),
                    2 | 3 => Self::MemToMem32(addr_src, addr_dst),
                    _ => unreachable!("there can only be 4 possibilites"),
                };

                Ok(ParsedInstruction {
                    instr: Instruction::Move(move_instr),
                    delta_ip: iter.travelled() as u32,
                })
            }
            _ => Err(format!(
                "Should have gotten valid move opcode, instead got {move_group:08b}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mov {
        use super::*;
        use crate::{cpu::MemIterator, Machine};

        #[test]
        fn read_mem() {
            let reg_to_reg = vec![0b0000_0000u8, 0, 0];

            let reg_to_mem32 = vec![0b0110_0000u8, 0, 0, 0, 0, 0];
            let reg_to_mem16 = vec![0b0101_0000u8, 0, 0, 0, 0, 0];
            let reg_to_mem8 = vec![0b0100_0000u8, 0, 0, 0, 0, 0];

            let mem_to_reg32 = vec![0b1010_0000u8, 0, 0, 0, 0, 0];
            let mem_to_reg16 = vec![0b1001_0000u8, 0, 0, 0, 0, 0];
            let mem_to_reg8 = vec![0b1000_0000u8, 0, 0, 0, 0, 0];

            let mem_to_mem32 = vec![0b1110_0000u8, 0, 0, 0, 0, 0, 0, 0, 0];
            let mem_to_mem16 = vec![0b1101_0000u8, 0, 0, 0, 0, 0, 0, 0, 0];
            let mem_to_mem8 = vec![0b1100_0000u8, 0, 0, 0, 0, 0, 0, 0, 0];

            let reg_to_reg_instr =
                Move::read(MemIterator::new(0, reg_to_reg.as_slice())).expect("should read");
            let reg_to_mem32_instr =
                Move::read(MemIterator::new(0, reg_to_mem32.as_slice())).expect("should read");
            let reg_to_mem16_instr =
                Move::read(MemIterator::new(0, reg_to_mem16.as_slice())).expect("should read");
            let reg_to_mem8_instr =
                Move::read(MemIterator::new(0, reg_to_mem8.as_slice())).expect("should read");
            let mem_to_reg32_instr =
                Move::read(MemIterator::new(0, mem_to_reg32.as_slice())).expect("should read");
            let mem_to_reg16_instr =
                Move::read(MemIterator::new(0, mem_to_reg16.as_slice())).expect("should read");
            let mem_to_reg8_instr =
                Move::read(MemIterator::new(0, mem_to_reg8.as_slice())).expect("should read");
            let mem_to_mem32_instr =
                Move::read(MemIterator::new(0, mem_to_mem32.as_slice())).expect("should read");
            let mem_to_mem16_instr =
                Move::read(MemIterator::new(0, mem_to_mem16.as_slice())).expect("should read");
            let mem_to_mem8_instr =
                Move::read(MemIterator::new(0, mem_to_mem8.as_slice())).expect("should read");

            assert_eq!(
                reg_to_reg_instr.instr,
                Instruction::Move(Move::RegToReg(Register::A, Register::A))
            );
            assert_eq!(
                reg_to_mem32_instr.instr,
                Instruction::Move(Move::RegToMem32(Register::A, 0))
            );
            assert_eq!(
                reg_to_mem16_instr.instr,
                Instruction::Move(Move::RegToMem16(Register::A, 0))
            );
            assert_eq!(
                reg_to_mem8_instr.instr,
                Instruction::Move(Move::RegToMem8(Register::A, 0))
            );
            assert_eq!(
                mem_to_reg32_instr.instr,
                Instruction::Move(Move::MemToReg32(0, Register::A))
            );
            assert_eq!(
                mem_to_reg16_instr.instr,
                Instruction::Move(Move::MemToReg16(0, Register::A))
            );
            assert_eq!(
                mem_to_reg8_instr.instr,
                Instruction::Move(Move::MemToReg8(0, Register::A))
            );
            assert_eq!(
                mem_to_mem32_instr.instr,
                Instruction::Move(Move::MemToMem32(0, 0))
            );
            assert_eq!(
                mem_to_mem16_instr.instr,
                Instruction::Move(Move::MemToMem16(0, 0))
            );
            assert_eq!(
                mem_to_mem8_instr.instr,
                Instruction::Move(Move::MemToMem8(0, 0))
            );
        }

        #[test]
        fn move_reg_to_reg() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::RegToReg(Register::A, Register::B));
            machine.cpu.registers.a = 42;
            machine.cpu.registers.b = 2;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.registers.a, machine.cpu.registers.b);
            assert_eq!(machine.cpu.registers.b, 42);
        }

        #[test]
        fn move_reg_to_mem32() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::RegToMem32(Register::A, 0x0));
            machine.cpu.registers.a = 0x0403_0201;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.mem[0], 0x01);
            assert_eq!(machine.cpu.mem[1], 0x02);
            assert_eq!(machine.cpu.mem[2], 0x03);
            assert_eq!(machine.cpu.mem[3], 0x04);
        }

        #[test]
        fn move_reg_to_mem16() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::RegToMem16(Register::A, 0x0));
            machine.cpu.registers.a = 0x0403_0201;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.mem[0], 0x01);
            assert_eq!(machine.cpu.mem[1], 0x02);
            assert_eq!(machine.cpu.mem[2], 0x00);
            assert_eq!(machine.cpu.mem[3], 0x00);
        }

        #[test]
        fn move_reg_to_mem8() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::RegToMem8(Register::A, 0x0));
            machine.cpu.registers.a = 0x0403_0201;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.mem[0], 0x01);
            assert_eq!(machine.cpu.mem[1], 0x00);
            assert_eq!(machine.cpu.mem[2], 0x00);
            assert_eq!(machine.cpu.mem[3], 0x00);
        }

        #[test]
        fn move_mem_to_reg32() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::MemToReg32(0, Register::A));
            machine.cpu.registers.a = 0xFFFF_FFFF;

            machine.cpu.mem[0] = 0x01;
            machine.cpu.mem[1] = 0x02;
            machine.cpu.mem[2] = 0x03;
            machine.cpu.mem[3] = 0x04;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.registers.a, 0x0403_0201);
        }

        #[test]
        fn move_mem_to_reg16() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::MemToReg16(0, Register::A));
            machine.cpu.registers.a = 0xFFFF_FFFF;

            machine.cpu.mem[0] = 0x01;
            machine.cpu.mem[1] = 0x02;
            machine.cpu.mem[2] = 0x03;
            machine.cpu.mem[3] = 0x04;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.registers.a, 0xFFFF_0201);
        }

        #[test]
        fn move_mem_to_reg8() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::MemToReg8(0, Register::A));
            machine.cpu.registers.a = 0xFFFF_FFFF;

            machine.cpu.mem[0] = 0x01;
            machine.cpu.mem[1] = 0x02;
            machine.cpu.mem[2] = 0x03;
            machine.cpu.mem[3] = 0x04;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.registers.a, 0xFFFF_FF01);
        }

        #[test]
        fn move_mem_to_mem32() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::MemToMem32(0x0, 0x4));

            machine.cpu.mem[0] = 0x01;
            machine.cpu.mem[1] = 0x02;
            machine.cpu.mem[2] = 0x03;
            machine.cpu.mem[3] = 0x04;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.mem[4], 0x01);
            assert_eq!(machine.cpu.mem[5], 0x02);
            assert_eq!(machine.cpu.mem[6], 0x03);
            assert_eq!(machine.cpu.mem[7], 0x04);
        }

        #[test]
        fn move_mem_to_mem16() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::MemToMem16(0x0, 0x4));

            machine.cpu.mem[0] = 0x01;
            machine.cpu.mem[1] = 0x02;
            machine.cpu.mem[2] = 0x03;
            machine.cpu.mem[3] = 0x04;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.mem[4], 0x01);
            assert_eq!(machine.cpu.mem[5], 0x02);
            assert_eq!(machine.cpu.mem[6], 0x00);
            assert_eq!(machine.cpu.mem[7], 0x00);
        }

        #[test]
        fn move_mem_to_mem8() {
            let mut machine = Machine::new();
            let instr = Instruction::Move(Move::MemToMem8(0x0, 0x4));

            machine.cpu.mem[0] = 0x01;
            machine.cpu.mem[1] = 0x02;
            machine.cpu.mem[2] = 0x03;
            machine.cpu.mem[3] = 0x04;

            machine.cpu.do_instruction(instr);

            assert_eq!(machine.cpu.mem[4], 0x01);
            assert_eq!(machine.cpu.mem[5], 0x00);
            assert_eq!(machine.cpu.mem[6], 0x00);
            assert_eq!(machine.cpu.mem[7], 0x00);
        }
    }
}
