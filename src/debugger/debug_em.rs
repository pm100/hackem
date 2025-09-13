use std::{cell::RefCell, collections::BTreeMap};

use common::pdb::database::Pdb;

use super::{pdbio::CodeLocation, shell::Shell};
use crate::emulator::engine::HackEngine;
use anyhow::{bail, Result};

pub struct HackSystem {
    pub engine: HackEngine,
    // pub shell: Shell,
    pub pdb: Pdb,
    pub waw: BTreeMap<u16, CodeLocation>,
    pub(crate) expr_value: RefCell<evalexpr::Value>,
}
impl Default for HackSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HackSystem {
    pub fn new() -> HackSystem {
        HackSystem {
            engine: HackEngine::new(),
            //  shell: Shell::new(),
            pdb: Pdb::new(),
            expr_value: RefCell::new(evalexpr::Value::Int(0)),
            waw: BTreeMap::new(),
        }
    }
    // converts a string representing an address into an address
    // if string contains ':' then its a source line
    // if string starts with '$' or 0x it is a hex number
    // if digits it is a decimal number
    // else its a symbol

    pub fn convert_addr(&self, addr_str: &str) -> Result<(u16, String)> {
        // source line?

        // if addr_str.contains(':') {
        //     let mut parts = addr_str.split(':');
        //     let file = parts.next().unwrap();
        //     let line = parts.next().unwrap();
        //     let file_info = self
        //         .lookup_file_by_name(file)
        //         .ok_or_else(|| anyhow::anyhow!("File '{}' not found", file))?;
        //     let line_no = line.parse::<i64>()?;
        //     if let Some(addr) = self
        //         .dbgdb
        //         .find_source_line_by_line_no(file_info.file_id, line_no)?
        //     {
        //         return Ok((addr.absaddr, addr_str.to_string()));
        //     }
        //     bail!("Source line not found");
        // }

        // is this a hex number?
        if let Some(hex) = addr_str.strip_prefix('$').or_else(|| {
            addr_str
                .strip_prefix("0x")
                .or_else(|| addr_str.strip_prefix("0X"))
        }) {
            return Ok((u16::from_str_radix(hex, 16)?, String::new()));
        }

        // a decimal number?
        if addr_str.chars().next().unwrap().is_ascii_digit() {
            return Ok((addr_str.parse::<u16>()?, String::new()));
        }

        // a c symbol? Only meaningful when running
        // because the c symbols we support depend on the
        // stack frame we are in

        // if self.run_done {
        //     if let Some(caddr) = self.find_csym_address(addr_str)? {
        //         return Ok((caddr, addr_str.to_string()));
        //     }
        // }

        // a regular symbol?
        let syms = self.get_symbols(addr_str)?;
        match syms.len() {
            0 => bail!("Symbol '{}' not found", addr_str),
            1 => Ok((syms[0].address, addr_str.to_string())),
            _ => bail!("Symbol '{}' is ambiguous", addr_str),
        }
    }
}
