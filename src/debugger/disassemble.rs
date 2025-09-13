pub struct Disassembler {}

impl Disassembler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn disassemble(instruction: u16) -> String {
        let opcode = instruction >> 15;
        match opcode {
            0 => {
                // A instruction
                // trace!("0x{:04x}  {:04x}", self.pc - 1, instruction);
                format!("@{:04x}", instruction & 0x7fff)
            }
            1 => {
                // C instruction
                //let a = (instruction >> 12) & 0x1;
                let c = (instruction >> 6) & 0x7F;
                let d = (instruction >> 3) & 0x7;
                let j = instruction & 0x7;
                let mut ins_str = String::new();
                if d != 0 {
                    ins_str.push_str(format!("{}=", Self::generate_dest_str(d)).as_str());
                }
                ins_str.push_str(Self::generate_comp_str(c).as_str());
                if j != 0 {
                    ins_str.push_str(format!(";{}", Self::generate_jump_str(j)).as_str());
                }
                ins_str
            }

            _ => unreachable!(),
        }
    }
    fn generate_jump_str(jump: u16) -> String {
        match jump {
            0b000 => "".to_string(),
            0b001 => "JGT".to_string(),
            0b010 => "JEQ".to_string(),
            0b011 => "JGE".to_string(),
            0b100 => "JLT".to_string(),
            0b101 => "JNE".to_string(),
            0b110 => "JLE".to_string(),
            0b111 => "JMP".to_string(),
            _ => unreachable!(),
        }
    }
    fn generate_dest_str(dest: u16) -> String {
        let mut s = String::new();
        if dest & 0b100 != 0 {
            s.push('A');
        }
        if dest & 0b010 != 0 {
            s.push('D');
        }
        if dest & 0b001 != 0 {
            s.push('M');
        }
        s
    }
    fn generate_comp_str(comp: u16) -> String {
        match comp {
            0b0101010 => "0",
            0b0111111 => "1",
            0b0111010 => "-1",
            0b0001100 => "D",
            0b0110000 => "A",
            0b0001101 => "!D",
            0b0110001 => "!A",
            0b0001111 => "-D",
            0b0110011 => "-A",
            0b0011111 => "D+1",
            0b0110111 => "A+1",
            0b0001110 => "D-1",
            0b0110010 => "A-1",
            0b0000010 => "D+A",
            0b0010011 => "D-A",
            0b0000111 => "A-D",
            0b0000000 => "D&A",
            0b0010101 => "D|A",
            0b1110000 => "M",
            0b1110001 => "!M",
            0b1110011 => "-M",
            0b1110111 => "M+1",
            0b1110010 => "M-1",
            0b1000010 => "D+M",
            0b1010011 => "D-M",
            0b1000111 => "M-D",
            0b1000000 => "D&M",
            0b1010101 => "D|M",
            _ => unreachable!(),
        }
        .to_string()
    }
}
