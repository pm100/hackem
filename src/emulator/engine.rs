use std::collections::BTreeMap;

use crate::ui::app::RuntimeError;

use crate::ui::widgets::screen::CURRENT_KEY;
use anyhow::{bail, Result};
use web_time::{Duration, Instant};
pub(crate) struct BreakPoint {
    pub enabled: bool,
}

pub struct WatchPoint {
    pub read: bool,
    pub write: bool,
    pub enabled: bool,
}

pub struct HackEngine {
    pub pc: u16,
    pub a: u16,
    pub d: u16,
    pub ram: [u16; 0x8000],
    pub rom: [u16; 0x8000],
    pub halt_addr: u16,
    pub speed: f32,
    pub rom_words_loaded: usize,
    pub ram_words_loaded: usize,
    inst_count: u64,
    pub break_points: BTreeMap<u16, BreakPoint>,
    pub watch_points: BTreeMap<u16, WatchPoint>,
    pub triggered_watchpoint: Option<u16>,
}
#[derive(Debug, PartialEq)]
pub(crate) enum StopReason {
    RefreshUI,
    SysHalt,
    HardLoop,
    BreakPoint,
    WatchPoint,
}
impl Default for HackEngine {
    fn default() -> Self {
        Self::new()
    }
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
            rom_words_loaded: 0,
            ram_words_loaded: 0,
            inst_count: 0,
            break_points: BTreeMap::new(),
            watch_points: BTreeMap::new(),
            triggered_watchpoint: None,
        }
    }
    fn alu(x_in: u16, y_in: u16, c: u16) -> u16 {
        let zx = (c >> 5) & 0x1;
        let nx = (c >> 4) & 0x1;
        let zy = (c >> 3) & 0x1;
        let ny = (c >> 2) & 0x1;
        let f = (c >> 1) & 0x1;
        let no = c & 0x1;

        let x = if zx != 0 { 0 } else { x_in };
        let x = if nx != 0 { !x } else { x };

        let y = if zy != 0 { 0 } else { y_in };
        let y = if ny != 0 { !y } else { y };

        if f != 0 {
            if no != 0 {
                !u16::wrapping_add(x, y)
            } else {
                u16::wrapping_add(x, y)
            }
        } else if no != 0 {
            !(x & y)
        } else {
            x & y
        }
    }
    pub fn set_ram(&mut self, address: u16, value: u16) -> Result<bool> {
        if address >= 0x8000 {
            //  println!("Invalid address {:04x} at {:04x}", address, self.pc);
            bail!(RuntimeError::InvalidWriteAddress(address));
        }

       let ui_stop =  match address {
            0x0000..=0x3fff => {
                self.ram[address as usize] = value;
                false
            }
            0x4000..=0x5fff => {
                // screen
                self.ram[address as usize] = value;
                true
            }
            0x6000 => {
                // keyboard - write ignored
                false
            }
            // 0x7fff => {
            //    // eprint!("{}", value);
            // }
            _ => {
                self.ram[address as usize] = value;
                address == 0x7FFF // write to output stream
            }
        };
        if let Some(wp) = self.watch_points.get(&address) {
            if wp.write && wp.enabled {
                self.triggered_watchpoint = Some(address);
            }
        }
        Ok(ui_stop)
    }
    pub fn get_ram(&mut self, address: u16) -> Result<u16> {
        if address >= 0x8000 {
            bail!(RuntimeError::InvalidReadAddress(address));
        }
        if address == 0x6000 {
            unsafe {
                return Ok(CURRENT_KEY as u16);
            }
        }
        if let Some(wp) = self.watch_points.get(&address) {
            if wp.read && wp.enabled {
                self.triggered_watchpoint = Some(address);
            }
        }
        Ok(self.ram[address as usize])
    }
    pub(crate) fn execute_instructions(&mut self, run_time: Duration) -> Result<StopReason> {
        let start_time = Instant::now();
        self.speed = 0.0;
        let mut counter = 0;
        let inst_count_snap = self.inst_count;
        loop {
            if self.pc >= 0x8000 {
                bail!(RuntimeError::InvalidPC(self.pc));
            }

            counter += 1;
            // every chunk of instructions check to see if we should refresh the UI
            // by returning to the caller
            if counter > 1000 {
                let time = Instant::now() - start_time;
                if time > run_time {
                    self.speed =
                        (self.inst_count - inst_count_snap) as f32 / time.as_secs_f32() / 1000000.0;
                    return Ok(StopReason::RefreshUI);
                }
                counter = 0;
            }
            self.inst_count += 1;

            let instruction = self.rom[self.pc as usize];

            // did we hit a call to Sys.halt?

            if self.halt_addr != 0 && self.pc == self.halt_addr + 1 {
                return Ok(StopReason::SysHalt);
            }
            let opcode = instruction >> 15;
            let old_pc = self.pc;
let mut ui_stop = false;
            self.pc += 1;
            match opcode {
                0 => {
                    // A instruction
                    // trace!("0x{:04x}  {:04x}", self.pc - 1, instruction);
                    self.a = instruction;
                }
                1 => {
                    // C instruction
                    let a = (instruction >> 12) & 0x1;
                    let c = (instruction >> 6) & 0x3F;
                    let d = (instruction >> 3) & 0x7;
                    let j = instruction & 0x7;

                    // cannot use A as a jump address and a ram read write address in the same instruction

                    if j != 0 && d & 0x1 != 0 {
                        bail!(RuntimeError::InvalidInstruction);
                    }

                    // let m = if a < 0x8000 {
                    //     format!("{:04x}", self.get_ram(a).unwrap())
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
                    //);
                    let y = if a == 0 {
                        self.a
                    } else {
                        self.get_ram(self.a)?
                    };

                    let alu_out = Self::alu(self.d, y, c);

                    // M
                    if d & 0x1 != 0 {
                        ui_stop = self.set_ram(self.a, alu_out)?;
                    }
                    // D
                    if d & 0x2 != 0 {
                        self.d = alu_out;
                    }
                    // A (update is deferred til after jmp is tested)
                    // see http://nand2tetris-questions-and-answers-forum.52.s1.nabble.com/Subtle-different-behaviors-between-the-CPU-emulator-and-the-proposed-CPU-design-td4034781.html
                    let new_a = if d & 0x4 != 0 { alu_out } else { self.a };

                    let pc = self.pc;

                    if j & 0x1 != 0 && (alu_out as i16) > 0 {
                        self.pc = self.a;
                    }
                    if j & 0x2 != 0 && alu_out == 0 {
                        self.pc = self.a;
                    }
                    if j & 0x4 != 0 && (alu_out as i16) < 0 {
                        self.pc = self.a;
                    }

                    self.a = new_a;
                    // detects halt loop
                    // (halt)
                    // @halt
                    // 0;JMP

                    if pc > 2 && self.pc == pc - 2 {
                        return Ok(StopReason::HardLoop);
                    }
                }
                _ => {
                    bail!(RuntimeError::InvalidInstruction);
                }
            }
            if let Some(bp) = self.break_points.get(&old_pc) {
                if bp.enabled {
                    return Ok(StopReason::BreakPoint);
                }
            }
            if self.triggered_watchpoint.take().is_some() {
                return Ok(StopReason::WatchPoint);
            }
            if run_time == Duration::ZERO || ui_stop{
                return Ok(StopReason::RefreshUI);
            }
        }
    }
    pub fn get_registers(&self) -> (u16, u16, u16) {
        (self.pc, self.a, self.d)
    }

    pub fn add_breakpoint(&mut self, address: u16) {
        self.break_points
            .insert(address, BreakPoint { enabled: true });
    }

    pub fn remove_breakpoint(&mut self, address: u16) {
        self.break_points.remove(&address);
    }

    pub fn remove_all_breakpoints(&mut self) {
        self.break_points.clear();
    }

    pub fn add_watchpoint(&mut self, address: u16, read: bool, write: bool) {
        self.watch_points.insert(
            address,
            WatchPoint {
                read,
                write,
                enabled: true,
            },
        );
    }

    pub fn remove_watchpoint(&mut self, address: u16) {
        self.watch_points.remove(&address);
    }

    pub fn remove_all_watchpoints(&mut self) {
        self.watch_points.clear();
    }

    /// Disassemble a single 16-bit Hack instruction word into a mnemonic string.
    pub fn disassemble_one(word: u16) -> String {
        if word >> 15 == 0 {
            // A-instruction: @value
            return format!("@{}", word & 0x7FFF);
        }

        // C-instruction: dest=comp;jump
        let a_bit = (word >> 12) & 0x1;
        let comp = (word >> 6) & 0x3F;
        let dest = (word >> 3) & 0x7;
        let jump = word & 0x7;

        let comp_str = if a_bit == 0 {
            match comp {
                0b101010 => "0",
                0b111111 => "1",
                0b111010 => "-1",
                0b001100 => "D",
                0b110000 => "A",
                0b001101 => "!D",
                0b110001 => "!A",
                0b001111 => "-D",
                0b110011 => "-A",
                0b011111 => "D+1",
                0b110111 => "A+1",
                0b001110 => "D-1",
                0b110010 => "A-1",
                0b000010 => "D+A",
                0b010011 => "D-A",
                0b000111 => "A-D",
                0b000000 => "D&A",
                0b010101 => "D|A",
                _ => "???",
            }
        } else {
            match comp {
                0b101010 => "0",
                0b111111 => "1",
                0b111010 => "-1",
                0b001100 => "D",
                0b110000 => "M",
                0b001101 => "!D",
                0b110001 => "!M",
                0b001111 => "-D",
                0b110011 => "-M",
                0b011111 => "D+1",
                0b110111 => "M+1",
                0b001110 => "D-1",
                0b110010 => "M-1",
                0b000010 => "D+M",
                0b010011 => "D-M",
                0b000111 => "M-D",
                0b000000 => "D&M",
                0b010101 => "D|M",
                _ => "???",
            }
        };

        let dest_str = match dest {
            0b000 => "",
            0b001 => "M=",
            0b010 => "D=",
            0b011 => "MD=",
            0b100 => "A=",
            0b101 => "AM=",
            0b110 => "AD=",
            0b111 => "AMD=",
            _ => unreachable!(),
        };

        let jump_str = match jump {
            0b000 => "",
            0b001 => ";JGT",
            0b010 => ";JEQ",
            0b011 => ";JGE",
            0b100 => ";JLT",
            0b101 => ";JNE",
            0b110 => ";JLE",
            0b111 => ";JMP",
            _ => unreachable!(),
        };

        format!("{}{}{}", dest_str, comp_str, jump_str)
    }

    /// Disassemble `count` instructions starting at `start` address.
    /// Returns (address, raw_word, mnemonic) for each instruction.
    pub fn disassemble_range(&self, start: u16, count: u16) -> Vec<(u16, u16, String)> {
        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count {
            let addr = start.wrapping_add(i);
            if addr as usize >= self.rom.len() {
                break;
            }
            let word = self.rom[addr as usize];
            result.push((addr, word, Self::disassemble_one(word)));
        }
        result
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
                break;
            }
        }
        assert!(cpu.ram[17] == 2);
        assert!(cpu.ram[16] == 3);
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
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
            if cpu.execute_instructions(Duration::ZERO).unwrap() == StopReason::HardLoop {
                break;
            }
        }
        assert!(cpu.ram[1] == 0);
    }

    // Run a real C program compiled by hack_cc:
    //   factorial(5) + fib(7) = 120 + 13 = 133
    // The return value of main ends up at RAM[256] (top of the call stack).
    #[test]
    fn test_c_program_factorial_fib() {
        let mut engine = HackEngine::new();
        let content = include_str!("../../tests/data/test2.hackem");
        engine.load_file(content).unwrap();

        let result = loop {
            let stop = engine
                .execute_instructions(Duration::from_secs(10))
                .unwrap();
            match stop {
                StopReason::SysHalt | StopReason::HardLoop => break stop,
                StopReason::RefreshUI => continue,
                other => panic!("unexpected stop: {:?}", other),
            }
        };
        assert_eq!(result, StopReason::SysHalt);
        assert_eq!(engine.ram[256] as i16, 133);
    }
}
