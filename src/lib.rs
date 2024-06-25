#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod hacksys;
mod key_lookup;
#[cfg(target_arch = "wasm32")]
mod wasm;
use crate::app::CURRENT_KEY;
pub use app::HackEmulator;
use web_time::{Duration, Instant};
mod code_loader;
use anyhow::{bail, Result};
pub struct HackEngine {
    pub pc: u16,
    pub a: u16,
    pub d: u16,
    pub ram: [u16; 0x8000],
    pub rom: [u16; 0x8000],
    pub halt_addr: u16,
    pub speed: f32,
    inst_count: u64,
    last_count: u64,
}
#[derive(Debug, PartialEq)]
pub(crate) enum StopReason {
    ScreenUpdate(u16),
    Count,
    Error(String),
    SysHalt,
    HardLoop,
    None,
}
impl HackEngine {
    pub fn new() -> HackEngine {
        HackEngine {
            pc: 0,
            a: 0,
            d: 0,
            ram: [0; 0x8000],
            rom: [0; 0x8000],
            halt_addr: 0,
            speed: 0.0,
            inst_count: 0,
            last_count: 0,
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
        if address >= 0x8000 {
            println!("Invalid address {:04x} at {:04x}", address, self.pc);
            //  return StopReason::Error(format!("Invalid address {} at {}", address, self.pc));
        }

        match address {
            0x0000..=0x3fff => {
                self.ram[address as usize] = value;
            }
            0x4000..=0x5fff => {
                // screen
                let old = self.ram[address as usize];
                self.ram[address as usize] = value;
            }
            0x6000 => {
                // keyboard - meaningless here
                self.ram[address as usize] = value;
            }
            _ => {
                self.ram[address as usize] = value;
            }
        }
    }
    fn get_ram(&mut self, address: u16) -> u16 {
        if address == 0x6000 {
            // keyboard
            // read from keyboard
            unsafe {
                //  println!("Current key: {}", CURRENT_KEY as u16);
                return CURRENT_KEY as u16;
            }
        }
        self.ram[address as usize]
    }
    pub(crate) fn execute_instructions(&mut self, run_time: Duration) -> StopReason {
        let now = Instant::now();
        self.speed = 0.0;
        let mut counter = 0;
        let inst_count_snap = self.inst_count;
        loop {
            counter = counter + 1;
            if counter > 1000 {
                let time = Instant::now() - now;
                if time > run_time {
                    self.speed =
                        (self.inst_count - inst_count_snap) as f32 / time.as_secs_f32() / 1000000.0;
                    return StopReason::Count;
                }
                counter = 0;
            }
            self.inst_count += 1;

            let instruction = self.rom[self.pc as usize];
            if self.halt_addr != 0 && self.pc == self.halt_addr + 1 {
                return StopReason::SysHalt;
            }
            let opcode = instruction >> 15;
            let a = (instruction >> 12) & 0x1;
            let c = (instruction >> 6) & 0x3F;
            let d = (instruction >> 3) & 0x7;
            let j = instruction & 0x7;
            self.pc = self.pc + 1;
            match opcode {
                0 => {
                    // A instruction
                    //trace!("0x{:04x}  {:04x}", self.pc - 1, instruction);
                    self.a = instruction;
                }
                1 => {
                    // C instruction
                    // let m = if a < 0x8000 {
                    //     format!("{:04x}", self.get_ram(a))
                    // } else {
                    //     "????".to_string()
                    // };
                    // trace!(
                    //     "0x{:04x}: {:04x} A={:04x} D={:04x} M={}",
                    //     self.pc - 1,
                    //     instruction,
                    //     self.a,
                    //     self.d,
                    //     m
                    // )
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
                        if (alu_out as i16) > 0 {
                            self.pc = self.a;
                        }
                    }
                    if j & 0x2 != 0 {
                        if alu_out == 0 {
                            self.pc = self.a;
                        }
                    }
                    if j & 0x4 != 0 {
                        if (alu_out as i16) < 0 {
                            self.pc = self.a;
                        }
                    }
                    // detects halt loop
                    if pc > 2 && self.pc == pc - 2 {
                        return StopReason::HardLoop;
                    }
                }
                _ => {
                    return StopReason::Error("Invalid opcode".to_string());
                }
            }
            if run_time == Duration::ZERO {
                return StopReason::Count;
            }
        }
    }
}
#[macro_export]
macro_rules! trace {
    ($fmt:literal, $($arg:expr),*) => {
        #[cfg(debug_assertions)]
        {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                log::warn!($fmt, $($arg),*);
            }
        }
    };
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            if cfg!(test){
                println!($msg);
            } else {
                log::warn!($msg);
            }
        }
    };
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
        //  let mut ram = [0; 0x8000];
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

        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[17] == 2);
        assert!(cpu.ram[16] == 3);
    }
    #[test]
    fn read_intercept() {
        let mut v1 = 0u16;
        let v2 = 42u16;
        {
            let mut cpu = HackEngine::new();

            // cpu.set_io(
            //     5000,
            //     5000,
            //     |_a, v| {
            //         println!("v: {}", v);
            //         v1 = v
            //     },
            //     |_address| v2,
            // );

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
            // cpu.run();
        }
        //assert_eq!(v1, 2);
    }
    #[test]
    fn test_jumps_1_gt() {
        let mut cpu = HackEngine::new();

        // at the end of test ram[1]
        //  == 1 means branch not taken
        //  == 0 means branch taken
        // D=1
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe301;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_1_eq() {
        let mut cpu = HackEngine::new();
        // D=1
        // @L1
        // D;JEQ
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe302;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_1_ge() {
        let mut cpu = HackEngine::new();
        // D=1
        // @L1
        // D;Jge
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe303;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_1_ne() {
        let mut cpu = HackEngine::new();
        // D=1
        // @L1
        // D;Jlt
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe305;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_1_le() {
        let mut cpu = HackEngine::new();
        // D=1
        // @L1
        // D;Jlt
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe306;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_1_jmp() {
        let mut cpu = HackEngine::new();
        // D=1
        // @L1
        // D;Jmp
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe307;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_0_gt() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe301;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_0_eq() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;Jeq
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe302;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_0_ge() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGe
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe303;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_1_lt() {
        let mut cpu = HackEngine::new();
        // D=1
        // @L1
        // D;Jlt
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xefd0;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe304;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_0_lt() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;Jlt
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe304;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_0_ne() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;Jne
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe305;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_0_le() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe306;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_0_jmp() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xea90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe307;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_neg1_gt() {
        let mut cpu = HackEngine::new();
        // D=-1
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe301;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_neg1_eq() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;Jeq
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe302;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_neg1_ge() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGe
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe303;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 1);
    }
    #[test]
    fn test_jumps_neg1_lt() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;Jlt
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe304;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_neg1_ne() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;Jne
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe305;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_neg1_le() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe306;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
    #[test]
    fn test_jumps_neg1_jmp() {
        let mut cpu = HackEngine::new();
        // D=0
        // @L1
        // D;JGT
        // @1
        // M=1
        // (L1)
        // @L1
        // D;JMP
        cpu.rom[0] = 0xee90;
        cpu.rom[1] = 0x0005;
        cpu.rom[2] = 0xe307;
        cpu.rom[3] = 0x0001;
        cpu.rom[4] = 0xefc8;
        cpu.rom[5] = 0x0005;
        cpu.rom[6] = 0xe307;
        loop {
            if cpu.execute_instruction() {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }
}
