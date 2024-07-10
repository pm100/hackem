use super::debug_em::HackSystem;
use anyhow::Result;
use common::pdb::database::Symbol;

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
}
