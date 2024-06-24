use crate::HackEngine;
use std::io::{self, Write};
use tempfile::tempfile;
enum LoadTarget {
    RAM,
    ROM,
    None,
}

impl HackEngine {
    pub fn load_file(&mut self, file: &str) {
        //let file = std::fs::read_to_string(path).unwrap();
        let mut address = 0;
        // peek at first line
        if file.starts_with("hackem") {
            let mut target = LoadTarget::None;
            for line in file.lines() {
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
                if line.starts_with("RAM@") {
                    address = u16::from_str_radix(&line[4..], 16).unwrap();
                    target = LoadTarget::RAM;
                } else if line.starts_with("ROM@") {
                    address = u16::from_str_radix(&&line[4..], 16).unwrap();
                    target = LoadTarget::ROM;
                } else {
                    let value = u16::from_str_radix(line, 16).unwrap();
                    match target {
                        LoadTarget::RAM => self.ram[address as usize] = value,
                        LoadTarget::ROM => self.rom[address as usize] = value,
                        LoadTarget::None => panic!("No target specified"),
                    }
                    address += 1;
                }
            }
        } else {
            for line in file.lines() {
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
    }
}
mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    #[test]
    fn test_load_file() {
        let binfile = r#"hackem v1.0
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

        let mut file = NamedTempFile::new().unwrap();

        file.write_all(binfile.as_bytes()).unwrap();
        let mut hack = HackEngine::new();
        hack.load_file(file.path().to_str().unwrap());

        assert_eq!(hack.rom[0], 0x0002);
        assert_eq!(hack.rom[1], 0x8c10);
        //  assert_eq!(hack.ram[0], 0x1234);
        //  assert_eq!(hack.ram[0x3334], 0xffff);
    }
}
