use std::path::Path;

use anyhow::{Result};
use common::pdb::database::Pdb;

use crate::utils;

use super::{debug_em::HackSystem, syntax};
pub struct Shell {}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        utils::set_say_cb(Self::say);
        Self {}
    }
    pub fn say(s: &str, _v: bool) {
        println!("{}", s);
    }
    pub fn execute_message(line: &str, hacksys: &mut HackSystem) -> Result<String> {
        match Self::dispatch(line, hacksys) {
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
    pub fn dispatch(line: &str, hacksys: &mut HackSystem) -> Result<String> {
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
                Ok(format!("Loaded pdb file {}", file))
            }
            Some(("cd", args)) => {
                let dir = args.get_one::<String>("directory").unwrap();
                std::env::set_current_dir(dir)?;
                let cwd = std::env::current_dir()?;
                Ok(format!("Current working directory: {}", cwd.display()))
            }
            Some(("list_symbols", _args)) => {
                let mut out = String::new();

                for func in &hacksys.pdb.symbols {
                    out.push_str(&format!("{}\n", func.name));
                }
                Ok(out)
            }
            _ => Ok("huh?".to_string()),
        }
    }
}
