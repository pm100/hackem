use crate::HackEngine;
use std::io::{self, Write};
use tempfile::tempfile;
enum LoadTarget {
    RAM,
    ROM,
    None,
}

impl<'a> HackEngine<'a> {
    pub fn load_file(&mut self, path: &str) {
        let file = std::fs::read_to_string(path).unwrap();
        let mut address = 0;

        let mut target = LoadTarget::None;
        for line in file.lines() {

            let line = line.trim();
            if line.is_empty() {
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
    }
}
mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    #[test]
    fn test_load_file() {
        let binfile = r#"
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
        assert_eq!(hack.ram[0], 0x1234);
        assert_eq!(hack.ram[0x3334], 0xffff);
    }
}
