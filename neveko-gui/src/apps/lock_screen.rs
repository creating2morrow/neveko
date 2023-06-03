use crate::CREDENTIAL_KEY;
use neveko_core::*;
use sha2::{
    Digest,
    Sha512,
};

/// TODO(c2m): Create a more secure locking mechanism
///
/// is there a way to trigger system screen lock on the machine???
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct LockScreen {
    credential: String,
}

impl Default for LockScreen {
    fn default() -> Self {
        LockScreen {
            credential: utils::empty_string(),
        }
    }
}

#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct LockScreenApp {
    is_locked: bool,
    lock_screen: LockScreen,
}

impl Default for LockScreenApp {
    fn default() -> Self {
        Self {
            is_locked: true,
            lock_screen: Default::default(),
        }
    }
}

impl LockScreenApp {
    pub fn get_lock_status(&mut self) -> bool {
        self.is_locked
    }
    pub fn set_lock(&mut self) {
        self.is_locked = true
    }
}

impl eframe::App for LockScreenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Screen Locked");
            ui.horizontal(|ui| {
                ui.label("credential: ");
                ui.add(egui::TextEdit::singleline(&mut self.lock_screen.credential).password(true));
            });
            if ui.button("Login").clicked() {
                // Get the credential hash from lmdb
                let s = db::Interface::open();
                let r = db::Interface::read(&s.env, &s.handle, CREDENTIAL_KEY);
                // hash the text entered and compare
                let mut hasher = Sha512::new();
                hasher.update(self.lock_screen.credential.clone());
                let result = hasher.finalize();
                let hex = hex::encode(&result[..]);
                if hex == r {
                    self.is_locked = false;
                }
                self.lock_screen = Default::default();
            }
        });
    }
}
