//! Main CPU related logic/data structures

use crate::{
    bitflag::Bitflag,
    instr::{Instruction, ReadMem},
};

const MAX_MEM: usize = 0x1000_0000;

pub const ZERO: u8 = 0b0000_0001;

#[derive(Debug, Clone, Copy)]
pub enum Register {
    A,
    B,
    X,
    Y,
    Ip,
}

impl Register {
    pub(crate) fn try_from_id(id: u8) -> Result<Self, String> {
        match id {
            0x0 => Ok(Self::A),
            0x1 => Ok(Self::B),
            0x2 => Ok(Self::X),
            0x3 => Ok(Self::Y),
            _ => Err(format!("Got invalid register id: 0x{id:01x}")),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct CpuRegisters {
    pub instruction_pointer: u32,

    pub a: u32,
    pub b: u32,

    pub x: u32,
    pub y: u32,

    pub flags: Bitflag<u8>,
}

// TODO: move somewhere else
pub struct MemIter<'mem> {
    index: usize,

    mem: &'mem [u8],
}

impl<'mem> MemIter<'mem> {
    pub(crate) fn new(start: usize, mem: &'mem [u8]) -> Self {
        Self { index: start, mem }
    }
}

impl<'mem> Iterator for MemIter<'mem> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.mem.len() {
            self.index = 0;
        }

        let v = self.mem[self.index];
        self.index += 1;

        Some(v)
    }
}

pub struct Cpu {
    pub registers: CpuRegisters,
    pub mem: Box<[u8; MAX_MEM]>,
}

#[allow(clippy::new_without_default)]
impl Cpu {
    pub fn new() -> Self {
        let slice = vec![0u8; MAX_MEM].into_boxed_slice();
        let ptr = Box::into_raw(slice) as *mut [u8; MAX_MEM];
        let mem = unsafe { Box::from_raw(ptr) };

        Self {
            registers: CpuRegisters::default(),
            mem,
        }
    }

    pub fn cycle(&mut self) {
        let ip = self.registers.instruction_pointer as usize;

        // TODO: this needs to increment IP
        let parsed_instr = Instruction::read(MemIter::new(ip, self.mem.as_slice()));

        match parsed_instr {
            Ok(parsed) => {
                self.registers.instruction_pointer = self
                    .registers
                    .instruction_pointer
                    .wrapping_add(parsed.delta_ip);
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
