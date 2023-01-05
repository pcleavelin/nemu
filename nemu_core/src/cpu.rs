// Copyright (C) 2023  Patrick Cleavelin <patrick@spacegirl.nl>

//! Main CPU related logic/data structures

use crate::{
    bitflag::Bitflag,
    instr::{self, Instruction, ReadMem},
};

const MAX_MEM: usize = 0x1000_0000;

pub const ZERO: u8 = 0b0000_0001;

#[derive(Debug, PartialEq, Clone, Copy)]
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

pub trait MemIter {
    fn next8(&mut self) -> u8;
    fn next16(&mut self) -> u16;
    fn next32(&mut self) -> u32;

    fn travelled(&self) -> usize;
}

// TODO: move somewhere else
pub struct MemIterator<'mem> {
    index: usize,
    travelled: usize,

    mem: &'mem [u8],
}

impl<'mem> MemIterator<'mem> {
    pub(crate) fn new(start: usize, mem: &'mem [u8]) -> Self {
        Self {
            index: start,
            travelled: 0,
            mem,
        }
    }
}

impl<'mem> MemIter for MemIterator<'mem> {
    fn next32(&mut self) -> u32 {
        if self.index >= self.mem.len() {
            self.index = 0;
        }

        let v = self.mem[self.index.wrapping_add(0)] as u32
            | ((self.mem[self.index.wrapping_add(1)] as u32) << 8)
            | ((self.mem[self.index.wrapping_add(2)] as u32) << 16)
            | ((self.mem[self.index.wrapping_add(3)] as u32) << 24);

        self.index += 4;
        self.travelled += 4;

        v
    }

    fn next16(&mut self) -> u16 {
        if self.index >= self.mem.len() {
            self.index = 0;
        }

        let v = self.mem[self.index.wrapping_add(0)] as u16
            | ((self.mem[self.index.wrapping_add(1)] as u16) << 8);

        self.index += 2;
        self.travelled += 2;

        v
    }

    fn next8(&mut self) -> u8 {
        if self.index >= self.mem.len() {
            self.index = 0;
        }

        let v = self.mem[self.index];
        self.index += 1;
        self.travelled += 1;

        v
    }

    fn travelled(&self) -> usize {
        self.travelled
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
        let parsed_instr = Instruction::read(MemIterator::new(ip, self.mem.as_slice()));

        match parsed_instr {
            Ok(parsed) => {
                self.registers.instruction_pointer = self
                    .registers
                    .instruction_pointer
                    .wrapping_add(parsed.delta_ip);

                self.do_instruction(parsed.instr);
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    pub fn do_instruction(&mut self, instr: Instruction) {
        match instr {
            Instruction::Halt => {}
            Instruction::Move(move_instr) => self.do_move_instruction(move_instr),
        }
    }

    fn do_move_instruction(&mut self, move_instr: instr::Move) {
        match move_instr {
            instr::Move::RegToReg(reg_src, reg_dst) => {
                self.set_reg32(reg_dst, self.get_reg(reg_src));
            }
            instr::Move::RegToMem32(reg_src, addr) => {
                self.write_mem32(addr, self.get_reg(reg_src));
            }
            instr::Move::RegToMem16(reg_src, addr) => {
                self.write_mem16(addr, (self.get_reg(reg_src) & 0xFFFF) as u16);
            }
            instr::Move::RegToMem8(reg_src, addr) => {
                self.write_mem8(addr, (self.get_reg(reg_src) & 0xFF) as u8);
            }

            instr::Move::MemToReg32(addr, reg_dst) => {
                self.set_reg32(reg_dst, self.read_mem32(addr));
            }
            instr::Move::MemToReg16(addr, reg_dst) => {
                self.set_reg16(reg_dst, self.read_mem16(addr));
            }
            instr::Move::MemToReg8(addr, reg_dst) => {
                self.set_reg8(reg_dst, self.read_mem8(addr));
            }

            instr::Move::MemToMem32(addr_src, addr_dest) => {
                self.write_mem32(addr_dest, self.read_mem32(addr_src));
            }
            instr::Move::MemToMem16(addr_src, addr_dest) => {
                self.write_mem16(addr_dest, self.read_mem16(addr_src));
            }
            instr::Move::MemToMem8(addr_src, addr_dest) => {
                self.write_mem8(addr_dest, self.read_mem8(addr_src));
            }
        }
    }

    fn get_reg(&self, reg: Register) -> u32 {
        match reg {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::X => self.registers.x,
            Register::Y => self.registers.y,
            Register::Ip => self.registers.instruction_pointer,
        }
    }

    fn set_reg32(&mut self, reg: Register, value: u32) {
        match reg {
            Register::A => self.registers.a = value,
            Register::B => self.registers.b = value,
            Register::X => self.registers.x = value,
            Register::Y => self.registers.y = value,
            Register::Ip => self.registers.instruction_pointer = value,
        }
    }

    fn set_reg16(&mut self, reg: Register, value: u16) {
        match reg {
            Register::A => self.registers.a = (self.registers.a & 0xFFFF_0000) | (value as u32),
            Register::B => self.registers.b = (self.registers.b & 0xFFFF_0000) | (value as u32),
            Register::X => self.registers.x = (self.registers.x & 0xFFFF_0000) | (value as u32),
            Register::Y => self.registers.y = (self.registers.y & 0xFFFF_0000) | (value as u32),
            Register::Ip => {
                self.registers.instruction_pointer =
                    (self.registers.instruction_pointer & 0xFFFF_0000) | (value as u32)
            }
        }
    }

    fn set_reg8(&mut self, reg: Register, value: u8) {
        match reg {
            Register::A => self.registers.a = (self.registers.a & 0xFFFF_FF00) | (value as u32),
            Register::B => self.registers.b = (self.registers.b & 0xFFFF_FF00) | (value as u32),
            Register::X => self.registers.x = (self.registers.x & 0xFFFF_FF00) | (value as u32),
            Register::Y => self.registers.y = (self.registers.y & 0xFFFF_FF00) | (value as u32),
            Register::Ip => {
                self.registers.instruction_pointer =
                    (self.registers.instruction_pointer & 0xFFFF_FF00) | (value as u32)
            }
        }
    }

    fn read_mem32(&self, addr: u32) -> u32 {
        let mut iter = MemIterator::new(addr as usize, self.mem.as_slice());

        iter.next32()
    }

    fn read_mem16(&self, addr: u32) -> u16 {
        let mut iter = MemIterator::new(addr as usize, self.mem.as_slice());

        iter.next16()
    }

    fn read_mem8(&self, addr: u32) -> u8 {
        self.mem[addr as usize]
    }

    fn write_mem32(&mut self, addr: u32, value: u32) {
        self.mem[addr as usize] = (value & 0xFF) as u8;
        self.mem[(addr.wrapping_add(1)) as usize] = ((value & 0xFF00) >> 8) as u8;
        self.mem[(addr.wrapping_add(2)) as usize] = ((value & 0xFF_0000) >> 16) as u8;
        self.mem[(addr.wrapping_add(3)) as usize] = ((value & 0xFF00_0000) >> 24) as u8;
    }

    fn write_mem16(&mut self, addr: u32, value: u16) {
        self.mem[addr as usize] = (value & 0xFF) as u8;
        self.mem[(addr.wrapping_add(1)) as usize] = ((value & 0xFF00) >> 8) as u8;
    }

    fn write_mem8(&mut self, addr: u32, value: u8) {
        self.mem[addr as usize] = value;
    }
}
