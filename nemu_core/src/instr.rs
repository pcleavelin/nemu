//! Instruction Set Implementation
use crate::cpu::Register;

#[derive(Debug)]
pub enum InstructionGroup {
    Move(Move),
    Halt,
}

impl FromIterator<u8> for InstructionGroup {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let Some(group_value) = iter.next() else { return Self::Halt };

        match group_value {
            0xF => Self::Halt,
            0x1 => Self::Move(Move::from_iter(iter)),
            _ => panic!("Should have gotten a valid group value, not {group_value:01x}"),
        }
    }
}

#[derive(Debug)]
pub enum Move {
    RegToReg(Register, Register),
    RegToMem(Register, u32),

    MemToReg(u32, Register),
    MemToMem(u32, u32),
}

impl FromIterator<u8> for Move {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let move_group = iter.next().expect("Somehow ran past memory iterator");

        match move_group {
            0x0 => {
                let reg_src_id = iter.next().expect("Somehow ran past memory iterator");
                let reg_dst_id = iter.next().expect("Somehow ran past memory iterator");

                let reg_src =
                    Register::try_from_id(reg_src_id).expect("Got invalid src register id");
                let reg_dst =
                    Register::try_from_id(reg_dst_id).expect("Got invalid dst register id");

                Self::RegToReg(reg_src, reg_dst)
            }
            _ => panic!("Should have gotten valid move opcode, instead got {move_group:01x}"),
        }
    }
}
