//! Instruction Set Implementation
use crate::cpu::Register;

pub trait ReadMem {
    type Item;

    fn read(iter: impl Iterator<Item = Self::Item>) -> Result<ParsedInstruction, String>;
}

pub struct ParsedInstruction {
    pub(crate) instr: Instruction,
    pub(crate) delta_ip: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Move(Move),
    Halt,
}

impl ReadMem for Instruction {
    type Item = u8;

    fn read(mut iter: impl Iterator<Item = Self::Item>) -> Result<ParsedInstruction, String> {
        let group_value = iter.next().ok_or("Somehow ran past iterator")?;

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
            },

            _ => {
                return Err(format!(
                    "Should have gotten a valid group value, not {group_value:01x}"
                ));
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    RegToReg(Register, Register),
    RegToMem(Register, u32),

    MemToReg(u32, Register),
    MemToMem(u32, u32),
}

impl ReadMem for Move {
    type Item = u8;

    fn read(iter: impl Iterator<Item = Self::Item>) -> Result<ParsedInstruction, String> {
        let mut iter = iter.enumerate();

        let (_, move_group) = iter.next().ok_or("Somehow ran past memory iterator")?;

        match move_group {
            0x0 => {
                let (_, reg_src_id) = iter.next().ok_or("Somehow ran past memory iterator")?;
                let (index, reg_dst_id) = iter.next().ok_or("Somehow ran past memory iterator")?;

                let reg_src = Register::try_from_id(reg_src_id)?;
                let reg_dst = Register::try_from_id(reg_dst_id)?;

                Ok(ParsedInstruction {
                    instr: Instruction::Move(Self::RegToReg(reg_src, reg_dst)),
                    delta_ip: (index as u32) + 1,
                })
            }
            _ => Err(format!(
                "Should have gotten valid move opcode, instead got {move_group:01x}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
