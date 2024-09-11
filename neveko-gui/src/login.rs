use crate::CREDENTIAL_KEY;
use db::DATABASE_LOCK;
use neveko_core::*;
use sha2::{
    Digest,
    Sha512,
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LoginApp {
    pub credential: String,
    pub is_cred_generated: bool,
    pub is_not_showing_password: bool,
}

impl Default for LoginApp {
    fn default() -> Self {
        let credential = String::new();
        let is_cred_generated = false;
        let is_not_showing_password = true;
        LoginApp {
            credential,
            is_cred_generated,
            is_not_showing_password,
        }
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
                ui.label("credential: \t");
                let mut show_password = self.is_not_showing_password;
                ui.add(
                    egui::TextEdit::singleline(&mut self.credential)
                        .password(self.is_not_showing_password),
                );
                if ui.checkbox(&mut show_password, "show password").changed() {
                    self.is_not_showing_password = !self.is_not_showing_password;
                }
            });
            if ui.button("Login").clicked() {
                // temporarily set the password to user environment and clear with screenlock
                // we set it here for the initial launch of neveko
                std::env::set_var(neveko_core::MONERO_WALLET_PASSWORD, self.credential.clone());
                let k = CREDENTIAL_KEY;
                let mut hasher = Sha512::new();
                hasher.update(self.credential.clone());
                let result = hasher.finalize();
                let db = &DATABASE_LOCK;
                db::write_chunks(&db.env, &db.handle, k.as_bytes(), hex::encode(&result[..]).as_bytes())
                    .unwrap_or_else(|_| log::error!("failed to set credential"));
                self.credential = String::new();
            }
        });
    }
}
