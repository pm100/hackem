use crate::HackEmulator;
use rfd::AsyncFileDialog;
// stuff for async file open in wasm

#[cfg(target_arch = "wasm32")]
static mut FILE_OPEN_STATE: RequestState = RequestState::Idle;
#[cfg(target_arch = "wasm32")]
static mut FILE_OPEN_DATA: String = String::new();
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RequestState {
    Idle,
    InFlight,
    HasData,
}
impl HackEmulator {
    pub(crate) fn wasm_update_and_menu(&mut self, ui: &mut egui::Ui) {
        unsafe {
            if FILE_OPEN_STATE == RequestState::HasData {
                FILE_OPEN_STATE = RequestState::InFlight;
                self.hacksys.load_file(&FILE_OPEN_DATA);
                log::debug!("Loaded binary");
            }
        }

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load Binary").clicked() {
                    unsafe { FILE_OPEN_STATE = RequestState::InFlight };
                    wasm_bindgen_futures::spawn_local(async move {
                        unsafe {
                            let file = AsyncFileDialog::new().pick_file().await;
                            let data = file.unwrap().read().await;
                            let bin = String::from_utf8(data).unwrap();
                            FILE_OPEN_DATA = bin;
                            FILE_OPEN_STATE = RequestState::HasData;
                        }
                    });
                    ui.close_menu();
                }
            })
        });
    }
}
