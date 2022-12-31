use instr::ReadMem;

pub mod bitflag;
pub mod cpu;
pub mod instr;

pub struct Snapshot<'machine> {
    pub next_instr: Option<instr::Instruction>,
    pub registers: cpu::CpuRegisters,
    pub mem_block: &'machine [u8],
}

impl<'machine> Snapshot<'machine> {
    pub fn pretty(&self) -> String {
        let registers = format!(
            r#"
----- Registers -----
|  IP:  0x{:08x}  |
|                   |
|  A:   0x{:08x}  |
|  B:   0x{:08x}  |
|                   |
|  X:   0x{:08x}  |
|  Y:   0x{:08x}  |
----- Registers -----

----- Next Instruction -----
{:#?}
----------------------------
"#,
            self.registers.instruction_pointer,
            self.registers.a,
            self.registers.b,
            self.registers.x,
            self.registers.y,
            self.next_instr
        );
        registers
    }
}

pub struct Machine {
    pub cpu: cpu::Cpu,
}

#[allow(clippy::new_without_default)]
impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: cpu::Cpu::new(),
        }
    }

    pub fn run_cycle(&mut self) {
        self.cpu.cycle();
    }

    pub fn snapshot(&self) -> Snapshot {
        let parsed_instr = match instr::Instruction::read(cpu::MemIter::new(
            self.cpu.registers.instruction_pointer as usize,
            self.cpu.mem.as_slice(),
        )) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("{e}");
                None
            }
        };

        Snapshot {
            next_instr: parsed_instr.map(|parsed| parsed.instr),
            registers: self.cpu.registers,
            mem_block: self.cpu.mem.as_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_wrap_around_proper_delta_ip() {
        let mut machine = Machine::new();

        machine.cpu.registers.instruction_pointer = 0xFFFF_FFFF;
        machine.run_cycle();

        assert_eq!(machine.cpu.registers.instruction_pointer, 0);
    }

    #[test]
    fn idk() {
        let mut machine = Machine::new();

        // machine.run_cycle();
        machine.cpu.registers.a = 0xFFF1_1FFF;

        machine.cpu.mem[0] = 0x1;
        machine.cpu.mem[1] = 0x8;
        machine.cpu.mem[2] = 0x0;
        machine.cpu.mem[3] = 0x2;

        let pretty = machine.snapshot().pretty();

        panic!("{pretty}");
    }
}
