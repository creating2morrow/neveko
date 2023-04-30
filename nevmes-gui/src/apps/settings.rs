use nevmes_core::*;
use sha2::{Digest, Sha512};

use crate::CREDENTIAL_KEY;


#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SettingsApp {
    credential: String,
}

impl Default for SettingsApp {
    fn default() -> Self {
        SettingsApp {
            credential: utils::empty_string(),
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.settings_ui(ui);
            ui.label("\n\n");
                ui.heading("Reset Credential");
                ui.label(
                    "____________________________________________________________________________\n",
                );
            ui.horizontal(|ui| {
                let sweep_label = ui.label("new credential: \t");
                ui.text_edit_singleline(&mut self.credential)
                    .labelled_by(sweep_label.id);
                if ui.button("Change").clicked() {
                    let s = db::Interface::open();
                    let k = CREDENTIAL_KEY;
                    db::Interface::delete(&s.env, &s.handle, &k);
                    let mut hasher = Sha512::new();
                    hasher.update(self.credential.clone());
                    let result = hasher.finalize();
                    db::Interface::write(&s.env, &s.handle, &k, &hex::encode(&result[..]));
                    self.credential = utils::empty_string();
                }
            });
        });
    }
}
