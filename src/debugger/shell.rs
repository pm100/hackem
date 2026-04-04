use std::path::Path;

use anyhow::Result;
use common::pdb::database::Pdb;
use web_time::Duration;

use crate::utils;

use super::{debug_em::HackSystem, syntax};

pub struct Shell {}

impl Shell {
    pub fn new() -> Self {
        utils::set_say_cb(Self::say);
        Self {}
    }

    fn say(s: &str, _v: bool) {
        println!("{}", s);
    }

    pub fn execute_message(&mut self, line: &str, hacksys: &mut HackSystem) -> Result<String> {
        match self.dispatch(line, hacksys) {
            Err(e) => {
                if let Some(original_error) = e.downcast_ref::<clap::error::Error>() {
                    Ok(format!("{}", original_error))
                } else if e.backtrace().status() == std::backtrace::BacktraceStatus::Captured {
                    Ok(format!("{} {}", e, e.backtrace()))
                } else {
                    Ok(format!("{}", e))
                }
            }
            Ok(string) => Ok(string),
        }
    }

    /// Resolve an address argument: `=expr` evaluates as expression, otherwise use convert_addr.
    fn resolve_addr(&self, arg: &str, hacksys: &HackSystem) -> Result<u16> {
        if let Some(expr) = arg.strip_prefix('=') {
            return hacksys.evaluate(expr);
        }
        Ok(hacksys.convert_addr(arg)?.0)
    }

    pub fn dispatch(&mut self, line: &str, hacksys: &mut HackSystem) -> Result<String> {
        let args = line.split_whitespace();
        let matches = syntax::syntax().try_get_matches_from(args)?;

        match matches.subcommand() {
            // load
            Some(("load_code", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                let bin = std::fs::read_to_string(Path::new(file))?;
                hacksys.engine.load_file(&bin)?;
                Ok(format!("Loaded {}", file))
            }
            Some(("load_pdb", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                let pdb_json = std::fs::read_to_string(Path::new(file))?;
                hacksys.pdb = Pdb::load_json(&pdb_json)?;
                hacksys.load_waw()?;
                Ok(format!("Loaded PDB {}", file))
            }

            // navigation
            Some(("next_instruction", _)) | Some(("step_instruction", _)) => {
                hacksys.engine.execute_instructions(Duration::ZERO)?;
                let (pc, a, d) = hacksys.engine.get_registers();
                Ok(format!("PC={:04X}  A={:04X}  D={:04X}", pc, a, d))
            }
            Some(("go", _)) => Ok("__go__".to_string()),

            // breakpoints
            Some(("break", args)) => {
                let raw = args.get_one::<String>("address").unwrap();
                let addr = self.resolve_addr(raw, hacksys)?;
                hacksys.engine.add_breakpoint(addr);
                Ok(format!("Breakpoint set at 0x{:04X}", addr))
            }
            Some(("list_breakpoints", _)) => {
                if hacksys.engine.break_points.is_empty() {
                    return Ok("No breakpoints".to_string());
                }
                let mut out = String::new();
                for (addr, bp) in &hacksys.engine.break_points {
                    out.push_str(&format!(
                        "0x{:04X}  {}\n",
                        addr,
                        if bp.enabled { "enabled" } else { "disabled" }
                    ));
                }
                Ok(out.trim_end().to_string())
            }
            Some(("delete_breakpoint", args)) => {
                if let Some(raw) = args.get_one::<String>("address") {
                    let addr = self.resolve_addr(raw, hacksys)?;
                    hacksys.engine.remove_breakpoint(addr);
                    Ok(format!("Deleted breakpoint at 0x{:04X}", addr))
                } else {
                    hacksys.engine.remove_all_breakpoints();
                    Ok("All breakpoints deleted".to_string())
                }
            }

            // watchpoints
            Some(("watch", args)) => {
                let raw = args.get_one::<String>("address").unwrap();
                let addr = self.resolve_addr(raw, hacksys)?;
                let read = args.get_flag("read");
                let write = args.get_flag("write");
                let (r, w) = if !read && !write { (true, true) } else { (read, write) };
                hacksys.engine.add_watchpoint(addr, r, w);
                Ok(format!(
                    "Watchpoint set at 0x{:04X} ({}{})",
                    addr,
                    if r { "r" } else { "" },
                    if w { "w" } else { "" }
                ))
            }
            Some(("list_watchpoints", _)) => {
                if hacksys.engine.watch_points.is_empty() {
                    return Ok("No watchpoints".to_string());
                }
                let mut out = String::new();
                for (addr, wp) in &hacksys.engine.watch_points {
                    out.push_str(&format!(
                        "0x{:04X}  {}{}\n",
                        addr,
                        if wp.read { "r" } else { "" },
                        if wp.write { "w" } else { "" }
                    ));
                }
                Ok(out.trim_end().to_string())
            }
            Some(("delete_watchpoint", args)) => {
                if let Some(raw) = args.get_one::<String>("address") {
                    let addr = self.resolve_addr(raw, hacksys)?;
                    hacksys.engine.remove_watchpoint(addr);
                    Ok(format!("Deleted watchpoint at 0x{:04X}", addr))
                } else {
                    hacksys.engine.remove_all_watchpoints();
                    Ok("All watchpoints deleted".to_string())
                }
            }

            // disassembly
            Some(("dis", args)) => {
                let start = if let Some(raw) = args.get_one::<String>("address") {
                    self.resolve_addr(raw, hacksys)?
                } else {
                    hacksys.engine.pc
                };
                let count = *args.get_one::<u16>("count").unwrap_or(&16);
                let lines = hacksys.engine.disassemble_range(start, count);
                let mut out = String::new();
                for (addr, raw, mnemonic) in lines {
                    let bp = if hacksys.engine.break_points.contains_key(&addr) { "*" } else { " " };
                    let pc = if addr == hacksys.engine.pc { ">" } else { " " };
                    out.push_str(&format!(
                        "{}{} {:04X}  {:04X}  {}\n",
                        pc, bp, addr, raw, mnemonic
                    ));
                }
                Ok(out.trim_end().to_string())
            }

            // memory
            Some(("display_memory", args)) => {
                let raw = args.get_one::<String>("address").unwrap();
                let start = self.resolve_addr(raw, hacksys)?;
                let count = *args.get_one::<u16>("count").unwrap_or(&16);
                let mut out = String::new();
                for i in 0..count {
                    let addr = start.wrapping_add(i);
                    if addr as usize >= hacksys.engine.ram.len() {
                        break;
                    }
                    let val = hacksys.engine.ram[addr as usize];
                    if i % 8 == 0 {
                        if i > 0 {
                            out.push('\n');
                        }
                        out.push_str(&format!("{:04X}:", addr));
                    }
                    out.push_str(&format!(" {:04X}", val));
                }
                Ok(out)
            }
            Some(("print", args)) => {
                let raw = args.get_one::<String>("address").unwrap();
                let addr = self.resolve_addr(raw, hacksys)?;
                let val = hacksys.engine.ram[addr as usize];
                if args.get_flag("asstring") {
                    let mut s = String::new();
                    let mut a = addr;
                    loop {
                        let c = hacksys.engine.ram[a as usize];
                        if c == 0 || a as usize >= hacksys.engine.ram.len() - 1 {
                            break;
                        }
                        s.push(char::from_u32(c as u32).unwrap_or('?'));
                        a = a.wrapping_add(1);
                    }
                    Ok(format!("0x{:04X} = \"{}\"", addr, s))
                } else {
                    Ok(format!("0x{:04X} = 0x{:04X}  ({})", addr, val, val as i16))
                }
            }

            // registers
            Some(("reg", _)) => {
                let (pc, a, d) = hacksys.engine.get_registers();
                Ok(format!("PC=0x{:04X}  A=0x{:04X}  D=0x{:04X}", pc, a, d))
            }

            // write memory
            Some(("write_memory", args)) => {
                let addr_raw = args.get_one::<String>("address").unwrap();
                let val_raw = args.get_one::<String>("value").unwrap();
                let addr = self.resolve_addr(addr_raw, hacksys)?;
                let val = self.resolve_addr(val_raw, hacksys)?;
                hacksys.engine.ram[addr as usize] = val;
                Ok(format!("0x{:04X} <- 0x{:04X}", addr, val))
            }

            // symbols
            Some(("list_symbols", args)) => {
                let filter = args.get_one::<String>("match");
                let mut out = String::new();
                hacksys
                    .pdb
                    .symbols
                    .iter()
                    .filter(|s| filter.is_none() || s.name.contains(filter.unwrap()))
                    .for_each(|s| {
                        let t = match s.symbol_type {
                            common::pdb::database::SymbolType::Func => "F",
                            common::pdb::database::SymbolType::Label => "L",
                            common::pdb::database::SymbolType::Var => "V",
                            common::pdb::database::SymbolType::Unknown => "?",
                        };
                        out.push_str(&format!("0x{:04X} {} {}\n", s.address, t, s.name));
                    });
                if out.is_empty() {
                    Ok("No symbols".to_string())
                } else {
                    Ok(out.trim_end().to_string())
                }
            }

            // expression
            Some(("expr", args)) => {
                let expr = args.get_one::<String>("expression").unwrap();
                let val = self.resolve_addr(expr, hacksys)?;
                Ok(format!("0x{:04X}  ({})", val, val as i16))
            }

            // misc
            Some(("cd", args)) => {
                let dir = args.get_one::<String>("directory").unwrap();
                std::env::set_current_dir(dir)?;
                Ok(format!("cwd: {}", std::env::current_dir()?.display()))
            }
            Some(("quit", _)) => Ok("__quit__".to_string()),
            Some(("about", _)) => Ok(syntax::syntax().render_long_help().to_string()),

            _ => Ok(String::new()),
        }
    }
}
