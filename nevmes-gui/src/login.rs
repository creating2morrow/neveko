use nevmes_core::*;
use sha2::{Sha512, Digest};
use crate::CREDENTIAL_KEY;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LoginApp {
    pub credential: String,
    pub is_cred_generated: bool,
}

impl Default for LoginApp {
    fn default() -> Self {
        let credential = utils::empty_string();
        let is_cred_generated = false;
        LoginApp { credential, is_cred_generated }
    }
}

impl eframe::App for LoginApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.is_cred_generated {
                self.credential = utils::generate_rnd();
                self.is_cred_generated = true;
            }
            ui.label("this is your randomly generated credential");
            ui.label("it will not be displayed again after logging in");
            ui.label("use this or set your own secure password.");
            ui.horizontal(|ui| {
                let cred_label = ui.label("credential: \t");
                ui.text_edit_singleline(&mut self.credential)
                    .labelled_by(cred_label.id);
            });
            if ui.button("Login").clicked() {
                // TODO(c2m): security / encryption, for now only the hash of auth put in lmdb
                let k = CREDENTIAL_KEY;
                let mut hasher = Sha512::new();
                hasher.update(self.credential.clone());
                let result = hasher.finalize();
                let s = db::Interface::open();
                db::Interface::write(&s.env, &s.handle, k, &hex::encode(&result[..]));
                self.credential = utils::empty_string();
            }
        });
    }
}

