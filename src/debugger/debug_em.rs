use std::{cell::RefCell, collections::BTreeMap};

use common::pdb::database::Pdb;

use super::pdbio::CodeLocation;
use crate::emulator::engine::HackEngine;
use anyhow::{bail, Result};

pub struct HackSystem {
    pub engine: HackEngine,
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
            pdb: Pdb::new(),
            expr_value: RefCell::new(evalexpr::Value::Int(0)),
            waw: BTreeMap::new(),
        }
    }

    /// Converts an address string to a numeric address.
    /// - `$1a2b` or `0x1a2b` → hex
    /// - plain digits → decimal
    /// - anything else → symbol lookup from PDB
    pub fn convert_addr(&self, addr_str: &str) -> Result<(u16, String)> {
        if let Some(hex) = addr_str.strip_prefix('$').or_else(|| {
            addr_str
                .strip_prefix("0x")
                .or_else(|| addr_str.strip_prefix("0X"))
        }) {
            return Ok((u16::from_str_radix(hex, 16)?, String::new()));
        }

        if addr_str.chars().next().unwrap().is_ascii_digit() {
            return Ok((addr_str.parse::<u16>()?, String::new()));
        }

        let syms = self.get_symbols(addr_str)?;
        match syms.len() {
            0 => bail!("Symbol '{}' not found", addr_str),
            1 => Ok((syms[0].address, addr_str.to_string())),
            _ => bail!("Symbol '{}' is ambiguous", addr_str),
        }
    }
}
