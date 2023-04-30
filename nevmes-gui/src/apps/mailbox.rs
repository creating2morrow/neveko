use nevmes_core::*;
use std::sync::mpsc::{Sender, Receiver};

pub struct MailBoxApp {
    decrypted_message: String,
    is_showing_decryption: bool,
    messages: Vec<models::Message>,
    message_init: bool,
    refresh_on_delete_tx: Sender<bool>,
    refresh_on_delete_rx: Receiver<bool>,
}

impl Default for MailBoxApp {
    fn default() -> Self {
        let (refresh_on_delete_tx, refresh_on_delete_rx) = std::sync::mpsc::channel();
        MailBoxApp {
            decrypted_message: utils::empty_string(),
            is_showing_decryption: false,
            messages: Vec::new(),
            message_init: false,
            refresh_on_delete_rx,
            refresh_on_delete_tx,
        }
    }
}

impl eframe::App for MailBoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Hook into async channel threads
        //-----------------------------------------------------------------------------------
        if let Ok(refresh) = self.refresh_on_delete_rx.try_recv() {
            if refresh { self.message_init = false; }
        }

        // initial message load
        if !self.message_init {
            self.messages = message::find_all();
            self.message_init = true;
        }

        // Compose window
        //-----------------------------------------------------------------------------------
        let mut is_showing_decryption = self.is_showing_decryption;
        egui::Window::new("Decrypted Message")
            .open(&mut is_showing_decryption)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.label(format!("{}", self.decrypted_message));
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.decrypted_message = utils::empty_string();
                    self.is_showing_decryption = false;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                self.messages = message::find_all();
            }
            ui.label("\n");
            use egui_extras::{Column, TableBuilder};

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::initial(100.0).at_least(40.0).clip(true))
                    .column(Column::initial(100.0).at_least(40.0).clip(true))
                    .column(Column::initial(100.0).at_least(40.0).clip(true))
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Date");
                        });
                        header.col(|ui| {
                            ui.strong("From");
                        });
                        header.col(|ui| {
                            ui.strong("To");
                        });
                        header.col(|ui| {
                            ui.strong("Message");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                    })
                    .body(|mut body|
                            for m in &self.messages {
                                let row_height =  200.0;
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        let h = chrono::NaiveDateTime::from_timestamp_opt(m.created, 0).unwrap().to_string();
                                        ui.label(format!("{}", h));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", m.from));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", m.to));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", 
                                            String::from_utf8(m.body.iter().cloned().collect()).unwrap()));
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap = Some(false);
                                        ui.horizontal(|ui| {
                                            if m.from != i2p::get_destination() {
                                                if ui.button("Decrypt").clicked() {
                                                    let mut d = message::decrypt_body(m.mid.clone());
                                                    let mut bytes = hex::decode(d.body.into_bytes())
                                                    .unwrap_or(Vec::new());
                                                    self.decrypted_message = String::from_utf8(bytes)
                                                        .unwrap_or(utils::empty_string());
                                                    self.is_showing_decryption = true;
                                                    d = Default::default();
                                                    bytes = Vec::new();
                                                    log::debug!("cleared decryption bytes: {:?} string: {}", bytes, d.body);
                                                }
                                            }
                                            if ui.button("Delete").clicked() {
                                                message::delete(&m.mid);
                                                refresh_on_delete_req(self.refresh_on_delete_tx.clone(), ctx.clone())
                                            }
                                        });
                                    });
                                });
                    });
        });
    }
}

fn refresh_on_delete_req
(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::error!("refreshing messages....");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}
