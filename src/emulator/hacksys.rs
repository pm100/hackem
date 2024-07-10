// use super::lib::{HackEngine, StopReason};
// use anyhow::Result;
// use web_time::Duration;

// pub struct HackSystem {
//     pub engine: HackEngine,
// }
// // fn set_ram(&mut self, address: u16, value: u16) {
// //     for io in &mut self.io_intercepts {
// //         if address >= io.start && address <= io.end {
// //             (io.write_cb)(address, value);
// //             return;
// //         }
// //     }
// //     self.ram[address as usize] = value;
// // }
// // fn get_ram(&mut self, address: u16) -> u16 {
// //     // for io in &mut self.io_intercepts {
// //     //     if address >= io.start && address <= io.end {
// //     //         return (io.read_cb)(address);
// //     //     }
// //     // }
// //     self.ram[address as usize]
// // }
// // impl RamIo for HackSystem {
// //     fn read(&self, address: u16) -> u16 {
// //         self.ram[address as usize]
// //     }

// //     fn write(&mut self, address: u16, value: u16) {
// //         self.ram[address as usize] = value;
// //     }
// // }
// impl HackSystem {
//     pub fn new() -> Self {
//         Self {
//             engine: HackEngine::new(),
//             // ram: [0; 0x8000],
//         }
//     }

//     pub fn get_screen_ram(&self) -> &[u16] {
//         &self.engine.ram[0x4000..0x6000]
//     }

//     pub fn get_registers(&self) -> (u16, u16, u16) {
//         (self.engine.pc, self.engine.a, self.engine.d)
//     }
//     pub fn get_speed(&self) -> f32 {
//         self.engine.speed
//     }
//     pub fn execute_instructions(&mut self, run_time: Duration) -> anyhow::Result<StopReason> {
//         self.engine.execute_instructions(run_time)
//     }
//     pub fn load_file(&mut self, path: &str) -> Result<()> {
//         self.engine.load_file(path)?;
//         Ok(())
//     }
// }
