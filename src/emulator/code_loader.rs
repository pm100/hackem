use crate::{say, utils};

use super::engine::HackEngine;
use anyhow::Result;

enum LoadTarget {
    Ram,
    Rom,
    None,
}

impl HackEngine {
    pub fn load_file(&mut self, bin: &str) -> Result<()> {
        // peek at first line
        let mut address = 0;
        if bin.starts_with("hackem") {
            let mut target = LoadTarget::None;
            for line in bin.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line.starts_with("hackem") {
                    let parts = line.split_whitespace().collect::<Vec<&str>>();
                    if parts.len() != 3 {
                        panic!("Invalid version line");
                    }
                    let version = parts[1];
                    if version != "v1.0" {
                        panic!("Invalid version");
                    }
                    let halt_str = parts[2];
                    self.halt_addr = u16::from_str_radix(&halt_str[2..], 16).unwrap();
                    continue;
                }
                if line.starts_with("//") {
                    continue;
                }
                if let Some(stripped) = line.strip_prefix("RAM@") {
                    address = u16::from_str_radix(stripped, 16).unwrap();
                    target = LoadTarget::Ram;
                } else if let Some(stripped) = line.strip_prefix("ROM@") {
                    address = u16::from_str_radix(stripped, 16).unwrap();
                    target = LoadTarget::Rom;
                } else {
                    let value = u16::from_str_radix(line, 16).unwrap();
                    match target {
                        LoadTarget::Ram => self.ram[address as usize] = value,
                        LoadTarget::Rom => self.rom[address as usize] = value,
                        LoadTarget::None => panic!("No target specified"),
                    }
                    address += 1;
                }
            }
        } else {
            for line in bin.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line.starts_with("//") {
                    continue;
                }
                let value = u16::from_str_radix(line, 2).unwrap();
                self.rom[address as usize] = value;
                address += 1;
            }
        }
        say!("Loaded file");
        Ok(())
    }
}
#[cfg(test)]
mod tests {

    use crate::emulator::engine::HackEngine;

    #[test]
    fn test_load_file() {
        let binfile = r#"hackem v1.0 0x0000
ROM@0000
0002
8c10
0011
8308
000a
8304
0003
8c10
0010
8308
000a
8307
RAM@0000
1234
2345
RAM@3333
abcd
ffff"#;

        let mut hack = HackEngine::new();
        hack.load_file(binfile);

        assert_eq!(hack.rom[0], 0x0002);
        assert_eq!(hack.rom[1], 0x8c10);
        //  assert_eq!(hack.ram[0], 0x1234);
        //  assert_eq!(hack.ram[0x3334], 0xffff);
    }
}
