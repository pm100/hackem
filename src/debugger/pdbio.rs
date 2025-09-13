use std::collections::BTreeMap;

use super::debug_em::HackSystem;
use anyhow::Result;
use common::pdb::database::Symbol;
#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub module: Option<i32>,
    pub cfile: Option<usize>,
    pub cline: usize,
    pub ctext: Option<String>,

    pub absaddr: u16,
}

impl HackSystem {
    pub fn get_symbols(&self, name: &str) -> Result<Vec<Symbol>> {
        Ok(self
            .pdb
            .symbols
            .iter()
            .filter(|s| s.name == name)
            .cloned()
            .collect())
    }

    pub fn where_are_we(&self, addr: u16) -> CodeLocation {
        if let Some(cl) = self.waw.range(..=addr).next_back() {
            cl.1.clone()
        } else {
            CodeLocation {
                module: None,
                cfile: None,
                cline: 0,
                ctext: None,
                absaddr: addr,
            }
        }
    }

    pub fn load_waw(&mut self) -> Result<()> {
        let mut waw = BTreeMap::new();
        for s in self.pdb.source_map.iter() {
            let cl = CodeLocation {
                module: None,
                cfile: Some(s.file),
                cline: s.line_no,
                ctext: None,
                absaddr: s.addr,
            };
            waw.insert(s.addr, cl);
        }
        self.waw = waw;
        Ok(())
    }
}
