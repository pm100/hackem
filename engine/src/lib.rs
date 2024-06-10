mod code_loader;

pub struct HackEngine<'a> {
    pub pc: u16,
    pub a: u16,
    pub d: u16,
    pub ram: [u16; 0x8000],
    pub rom: [u16; 0x8000],
    io_intercepts: Vec<IoIntercept<'a>>,
}

struct IoIntercept<'a> {
    start: u16,
    end: u16,
    write_cb: Box<dyn FnMut(u16, u16) + 'a>,
    read_cb: Box<dyn FnMut(u16) -> u16 + 'a>,
}
impl<'a> HackEngine<'a> {
    pub fn new() -> HackEngine<'a> {
        HackEngine {
            pc: 0,
            a: 0,
            d: 0,
            ram: [0; 0x8000],
            rom: [0; 0x8000],
            io_intercepts: Vec::new(),
        }
    }
    pub fn set_io(
        &mut self,
        start: u16,
        end: u16,
        write_cb: impl FnMut(u16, u16) + 'a,
        read_cb: impl FnMut(u16) -> u16 + 'a,
    ) {
        self.io_intercepts.retain(|x| x.start != start);
        self.io_intercepts.push(IoIntercept {
            start: start,
            end: end,
            write_cb: Box::new(write_cb),
            read_cb: Box::new(read_cb),
        });
    }
    pub fn del_io(&mut self, start: u16) {
        self.io_intercepts.retain(|x| x.start != start);
    }
    pub fn run(&mut self) {
        println!("HackEngine is running");
        let mut count = 0;
        loop {
            count = count + 1;
            if count > 100 {
                println!("Too many instructions");
                break;
            }
            if self.execute_instruction() {
                break;
            };
            // println!("pc: {}, a: {}, d: {}", self.pc, self.a, self.d);
        }
    }

    fn alu(x_in: u16, y_in: u16, c: u16) -> u16 {
        let zx = (c >> 5) & 0x1;
        let nx = (c >> 4) & 0x1;
        let zy = (c >> 3) & 0x1;
        let ny = (c >> 2) & 0x1;
        let f = (c >> 1) & 0x1;
        let no = (c >> 0) & 0x1;

        let x = if zx != 0 { 0 } else { x_in };
        //  println!("x: {}", x);
        let x = if nx != 0 { !x } else { x };
        //    println!("x: {}", x);
        let y = if zy != 0 { 0 } else { y_in };
        //   println!("y: {}", y);

        let y = if ny != 0 { !y } else { y };
        //  println!("y: {}", y);

        let out = if f != 0 {
            if no != 0 {
                !u16::wrapping_add(x, y)
            } else {
                u16::wrapping_add(x, y)
            }
        } else {
            if no != 0 {
                !(x & y)
            } else {
                x & y
            }
        };
        //  println!("out: {}", out);
        out
    }
    fn set_ram(&mut self, address: u16, value: u16) {
        for io in &mut self.io_intercepts {
            if address >= io.start && address <= io.end {
                (io.write_cb)(address, value);
                return;
            }
        }
        self.ram[address as usize] = value;
    }
    fn get_ram(&mut self, address: u16) -> u16 {
        for io in &mut self.io_intercepts {
            if address >= io.start && address <= io.end {
                return (io.read_cb)(address);
            }
        }
        self.ram[address as usize]
    }
    pub fn execute_instruction(&mut self) -> bool {
        let instruction = self.rom[self.pc as usize];
        let opcode = instruction >> 15;
        let a = (instruction >> 12) & 0x1;
        let c = (instruction >> 6) & 0x3F;
        let d = (instruction >> 3) & 0x7;
        let j = instruction & 0x7;
        self.pc = self.pc + 1;
        match opcode {
            0 => {
                // A instruction
                self.a = instruction;
                false
            }
            1 => {
                // C instruction
                let y = if a == 0 { self.a } else { self.get_ram(self.a) };
                let alu_out = Self::alu(self.d, y, c);
                if d & 0x1 != 0 {
                    self.set_ram(self.a, alu_out);
                }
                if d & 0x2 != 0 {
                    self.d = alu_out;
                }
                if d & 0x4 != 0 {
                    self.a = alu_out;
                }
                let pc = self.pc;

                if j & 0x1 != 0 {
                    if alu_out > 0 {
                        self.pc = self.a;
                    }
                }
                if j & 0x2 != 0 {
                    if alu_out == 0 {
                        self.pc = self.a;
                    }
                }
                if j & 0x4 != 0 {
                    if alu_out & 0x8000 > 0 {
                        self.pc = self.a;
                    }
                }
                // detects halt loop
                self.pc == pc - 2
            }
            _ => {
                panic!("Invalid opcode");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_alu_core(x: u16, y: u16) {
        assert!(HackEngine::alu(x, y, 0x2a) == 0);
        assert!(HackEngine::alu(x, y, 0x3f) == 1);
        assert!(HackEngine::alu(x, y, 0x3a) == 0xffff);
        assert!(HackEngine::alu(x, y, 0x0c) == x);
        assert!(HackEngine::alu(x, y, 0x30) == y);
        assert!(HackEngine::alu(x, y, 0x0d) == !x);
        assert!(HackEngine::alu(x, y, 0x31) == !y);
        assert!(HackEngine::alu(x, y, 0x0f) == u16::wrapping_sub(0, x));
        assert!(HackEngine::alu(x, y, 0x33) == u16::wrapping_sub(0, y));
        assert!(HackEngine::alu(x, y, 0x1f) == u16::wrapping_add(x, 1));
        assert!(HackEngine::alu(x, y, 0x37) == u16::wrapping_add(y, 1));
        assert!(HackEngine::alu(x, y, 0x0e) == u16::wrapping_sub(x, 1));
        assert!(HackEngine::alu(x, y, 0x32) == u16::wrapping_sub(y, 1));
        assert!(HackEngine::alu(x, y, 0x02) == u16::wrapping_add(x, y));
        assert!(HackEngine::alu(x, y, 0x13) == u16::wrapping_sub(x, y));
        assert!(HackEngine::alu(x, y, 0x07) == u16::wrapping_sub(y, x));
        assert!(HackEngine::alu(x, y, 0x00) == x & y);
        assert!(HackEngine::alu(x, y, 0x15) == x | y);
    }
    #[test]
    fn test_alu() {
        for x in 0..=0xffff {
            for y in 0..=0xff {
                test_alu_core(x, y);
            }
        }
        for y in 0..=0xffff {
            for x in 0..=0xff {
                test_alu_core(x, y);
            }
        }
    }

    // write some tests for the cpu
    #[test]
    fn test_cpu() {
        let mut cpu = HackEngine::new();
        // @2
        // D=A
        // @x
        // M=D
        // @POOP
        // D;JLT
        // @3
        // D=A
        // @y
        // M=D
        // (POP)
        // (HALT)
        // @HALT
        // D;JMP
        cpu.rom[0] = 0x0002;
        cpu.rom[1] = 0x8c10;
        cpu.rom[2] = 0x0011;
        cpu.rom[3] = 0x8308;
        cpu.rom[4] = 0x000a;
        cpu.rom[5] = 0x8304;
        cpu.rom[6] = 0x0003;
        cpu.rom[7] = 0x8c10;
        cpu.rom[8] = 0x0010;
        cpu.rom[9] = 0x8308;
        cpu.rom[10] = 0x000a;
        cpu.rom[11] = 0x8307;

        cpu.run();
        assert!(cpu.ram[17] == 2);
        assert!(cpu.ram[16] == 3);
    }
    #[test]
    fn read_intercept() {
        let mut v1 = 0u16;
        let v2 = 42u16;
        {
            let mut cpu = HackEngine::new();

            cpu.set_io(
                5000,
                5000,
                |_a, v| {
                    println!("v: {}", v);
                    v1 = v
                },
                |_address| v2,
            );

            cpu.rom[0] = 0x0002;
            cpu.rom[1] = 0x8c10;
            cpu.rom[2] = 5000;
            cpu.rom[3] = 0x8308;
            cpu.rom[4] = 0x000a;
            cpu.rom[5] = 0x8304;
            cpu.rom[6] = 0x0003;
            cpu.rom[7] = 0x8c10;
            cpu.rom[8] = 0x0010;
            cpu.rom[9] = 0x8308;
            cpu.rom[10] = 0x000a;
            cpu.rom[11] = 0x8307;
            cpu.run();
        }
        assert_eq!(v1, 2);
    }
}
