use neveko_core::{
    models::Message,
    *,
};
use std::sync::mpsc::{
    Receiver,
    Sender,
};

pub struct MailBoxApp {
    deciphered: String,
    is_showing_decipher: bool,
    messages: Vec<models::Message>,
    message_init: bool,
    refresh_on_delete_tx: Sender<bool>,
    refresh_on_delete_rx: Receiver<bool>,
    deciphered_tx: Sender<String>,
    deciphered_rx: Receiver<String>,
}

impl Default for MailBoxApp {
    fn default() -> Self {
        let (refresh_on_delete_tx, refresh_on_delete_rx) = std::sync::mpsc::channel();
        let (deciphered_tx, deciphered_rx) = std::sync::mpsc::channel();
        MailBoxApp {
            deciphered: String::new(),
            is_showing_decipher: false,
            messages: Vec::new(),
            message_init: false,
            refresh_on_delete_tx,
            refresh_on_delete_rx,
            deciphered_rx,
            deciphered_tx,
        }
    }
}

impl eframe::App for MailBoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hook into async channel threads
        //-----------------------------------------------------------------------------------
        if let Ok(refresh) = self.refresh_on_delete_rx.try_recv() {
            if refresh {
                self.message_init = false;
            }
        }

        if let Ok(decipher) = self.deciphered_rx.try_recv() {
            self.deciphered = decipher;
        }

        // initial message load
        if !self.message_init {
            self.messages = message::find_all();
            self.message_init = true;
        }

        // Compose window
        //-----------------------------------------------------------------------------------
        let mut is_showing_decipher = self.is_showing_decipher;
        egui::Window::new("decipher message")
            .open(&mut is_showing_decipher)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Deciphered Message");
                ui.label(format!("{}", self.deciphered));
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.deciphered = String::new();
                    self.is_showing_decipher = false;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                self.messages = message::find_all();
            }
            ui.label("\n");
            use egui_extras::{
                Column,
                TableBuilder,
            };

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
                .body(|mut body| {
                    for m in &self.messages {
                        let row_height = 200.0;
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                let h = chrono::NaiveDateTime::from_timestamp_opt(m.created, 0)
                                    .unwrap()
                                    .to_string();
                                ui.label(format!("{}", h));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", m.from));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", m.to));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", m.body));
                            });
                            row.col(|ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.horizontal(|ui| {
                                    if m.uid == String::from("rx") {
                                        if ui.button("Decipher").clicked() {
                                            decipher_req(
                                                &m,
                                                self.deciphered_tx.clone(),
                                                ctx.clone(),
                                            );
                                            self.is_showing_decipher = true;
                                        }
                                    }
                                    if ui.button("Delete").clicked() {
                                        message::delete(&m.mid);
                                        refresh_on_delete_req(
                                            self.refresh_on_delete_tx.clone(),
                                            ctx.clone(),
                                        )
                                    }
                                });
                            });
                        });
                    }
                });
        });
    }
}

fn refresh_on_delete_req(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::info!("refreshing messages....");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn decipher_req(m: &Message, tx: Sender<String>, ctx: egui::Context) {
    let from: String = String::from(&m.from);
    let body: String = String::from(&m.body);
    tokio::spawn(async move {
        log::info!("async decipher_req");
        let contact = contact::find_by_i2p_address(&from);
        let deciphered = neveko25519::cipher(&contact.nmpk, body, None).await;
        let _ = tx.send(deciphered);
        ctx.request_repaint();
    });
}
