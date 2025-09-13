use std::path::Path;

use anyhow::Result;
use common::pdb::database::Pdb;

use crate::utils;

use super::{debug_em::HackSystem, syntax};
pub struct Shell {
    //  hacksys: &'static HackSystem,
}

// impl Default for Shell {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl Shell {
    pub fn new() -> Self {
        utils::set_say_cb(Self::say);
        Self {}
    }
    pub fn say(s: &str, _v: bool) {
        println!("{}", s);
    }
    pub fn execute_message(&mut self, line: &str, hacksys: &mut HackSystem) -> Result<String> {
        println!("Executing: {}", line);
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

            Ok(string) => Ok(string), // continue
        }
    }
    fn expand_expr(&mut self, exp: &str, hacksys: &HackSystem) -> Result<String> {
        if let Some(exp) = exp.strip_prefix('=') {
            let res = hacksys.evaluate(exp)?;
            Ok(format!("${:x}", res))
        } else {
            Ok(exp.to_string())
        }
    }
    pub fn dispatch(&mut self, line: &str, hacksys: &mut HackSystem) -> Result<String> {
        //let args = shlex::split(line).ok_or(anyhow!("error: Invalid quoting"))?;
        let args = line.split_whitespace();
        // parse with clap
        let matches = syntax::syntax().try_get_matches_from(args)?;
        // execute the command
        match matches.subcommand() {
            Some(("load_code", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                let bin = std::fs::read_to_string(Path::new(file))?;
                hacksys.engine.load_file(&bin)?;
                Ok(format!("Loaded file {}", file))
            }
            Some(("load_pdb", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                let pdb_json = std::fs::read_to_string(Path::new(file))?;
                hacksys.pdb = Pdb::load_json(&pdb_json)?;
                hacksys.load_waw()?;
                Ok(format!("Loaded pdb file {}", file))
            }
            Some(("cd", args)) => {
                let dir = args.get_one::<String>("directory").unwrap();
                std::env::set_current_dir(dir)?;
                let cwd = std::env::current_dir()?;
                Ok(format!("Current working directory: {}", cwd.display()))
            }
            Some(("list_symbols", args)) => {
                let mut out = String::new();
                let filter = args.get_one::<String>("match");
                hacksys
                    .pdb
                    .symbols
                    .iter()
                    .filter(|s| filter.is_none() || s.name.contains(filter.unwrap()))
                    .for_each(|s| {
                        let sym_str = match s.symbol_type {
                            common::pdb::database::SymbolType::Func => "F",
                            common::pdb::database::SymbolType::Label => "L",
                            common::pdb::database::SymbolType::Var => "V",
                            common::pdb::database::SymbolType::Unknown => "?",
                        };

                        out.push_str(&format!("0x{:04x} {} {}\n", s.address, sym_str, s.name))
                    });
                Ok(out)
            }
            Some(("expr", args)) => {
                let expr = args.get_one::<String>("expression").unwrap();
                let ans = self.expand_expr(expr, hacksys)?;
                Ok(format!("{:}", ans))
            }
            Some(("break", args)) => {
                let addr = args.get_one::<String>("address").unwrap();
                let addr = self.expand_expr(addr, hacksys)?;
                let addr = u16::from_str_radix(&addr[0..], 16)?;
                hacksys.engine.add_breakpoint(addr);
                Ok(format!("Added breakpoint at 0x{:04x}", addr))
            }
            Some(("display_memory", args)) => {
                // let addr = args.get_one::<String>("address").unwrap();
                // let addr = self.expand_expr(addr, hacksys)?;
                // let addr = u16::from_str_radix(&addr[1..], 16)?;
                // let value = hacksys.engine.read_memory(addr);
                // Ok(format!("Memory at 0x{:04x}: {}", addr, value))

                for i in 0..0x4000 {
                    let value = hacksys.engine.get_ram(i)?;
                    println!("0x{:04x}: {:04x}", i, value);
                }
                Ok(format!("Displayed memory from 0x0000 to 0x3FFF"))
            }
            _ => Ok("huh?".to_string()),
        }
    }
}
