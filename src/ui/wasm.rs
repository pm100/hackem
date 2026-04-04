use crate::HackEgui;
use rfd::AsyncFileDialog;

static mut FILE_OPEN_STATE: RequestState = RequestState::Idle;
static mut FILE_OPEN_DATA: String = String::new();

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RequestState {
    Idle,
    InFlight,
    HasData,
}

impl HackEgui {
    pub(crate) fn wasm_update_and_menu(&mut self, ui: &mut egui::Ui) {
        unsafe {
            if FILE_OPEN_STATE == RequestState::HasData {
                FILE_OPEN_STATE = RequestState::InFlight;
                let bin = FILE_OPEN_DATA.clone();
                let _ = self.hacksys.engine.load_file(&bin);
                log::debug!("Loaded binary");
            }
        }

        ui.menu_button("File", |ui| {
            if ui.button("Load Binary").clicked() {
                unsafe { FILE_OPEN_STATE = RequestState::InFlight };
                wasm_bindgen_futures::spawn_local(async move {
                    unsafe {
                        let file = AsyncFileDialog::new()
                            .add_filter("x", &["hx"])
                            .pick_file()
                            .await;
                        let data = file.unwrap().read().await;
                        let bin = String::from_utf8(data).unwrap();
                        FILE_OPEN_DATA = bin;
                        FILE_OPEN_STATE = RequestState::HasData;
                    }
                });
                ui.close_menu();
            }
        });
    }
}
