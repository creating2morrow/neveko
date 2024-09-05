use neveko_core::*;
use sha2::{
    Digest,
    Sha512,
};
use std::sync::mpsc::{
    Receiver,
    Sender,
};

use crate::CREDENTIAL_KEY;

pub struct SettingsApp {
    credential: String,
    change_wallet_password_tx: Sender<bool>,
    change_wallet_password_rx: Receiver<bool>,
    is_loading: bool,
    is_not_showing_password: bool,
}

impl Default for SettingsApp {
    fn default() -> Self {
        let (change_wallet_password_tx, change_wallet_password_rx) = std::sync::mpsc::channel();
        SettingsApp {
            credential: String::new(),
            change_wallet_password_rx,
            change_wallet_password_tx,
            is_loading: false,
            is_not_showing_password: true,
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //--- async hooks
        if let Ok(update_password) = self.change_wallet_password_rx.try_recv() {
            if !update_password {
                log::error!("failed to update wallet password");
            }
            self.is_loading = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                ui.add(egui::Spinner::new());
            }
            ctx.settings_ui(ui);
            ui.label("\n\n");
            ui.heading("Reset Credential");
            ui.label(
                "____________________________________________________________________________\n",
            );
            ui.horizontal(|ui| {
                ui.label("new credential: \t");
                ui.add(
                    egui::TextEdit::singleline(&mut self.credential)
                        .password(self.is_not_showing_password),
                );
                let mut show_password = self.is_not_showing_password;
                if ui.checkbox(&mut show_password, "show password").changed() {
                    self.is_not_showing_password = !self.is_not_showing_password;
                }
                if ui.button("Change").clicked() {
                    self.is_loading = true;

                    // TODO: don't open the database in the GUI
                    
                    let s = db::DatabaseEnvironment::open(&utils::get_release_env().value()).unwrap();
                    let k = CREDENTIAL_KEY;
                    db::DatabaseEnvironment::delete(&s.env, &s.handle, &k);
                    let mut hasher = Sha512::new();
                    hasher.update(self.credential.clone());
                    let result = hasher.finalize();
                    db::write_chunks(&s.env, &s.handle, &k, &hex::encode(&result[..]));
                    // update wallet rpc
                    change_wallet_password(
                        self.change_wallet_password_tx.clone(),
                        &self.credential,
                        ctx.clone(),
                    );
                    self.credential = String::new();
                }
            });
        });
    }
}

fn change_wallet_password(
    change_wallet_password_tx: Sender<bool>,
    new_password: &String,
    ctx: egui::Context,
) {
    let update_password = String::from(new_password);
    tokio::spawn(async move {
        let wallet_name = String::from(neveko_core::APP_NAME);
        let wallet_password =
            std::env::var(neveko_core::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
        monero::open_wallet(&wallet_name, &wallet_password).await;
        let is_changed: bool = monero::change_wallet_password(&update_password).await;
        if is_changed {
            std::env::set_var(neveko_core::MONERO_WALLET_PASSWORD, update_password);
        }
        monero::close_wallet(&wallet_name, &wallet_password).await;
        let _ = change_wallet_password_tx.send(is_changed);
        ctx.request_repaint();
    });
}
