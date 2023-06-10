use image::Luma;
use neveko_core::*;
use qrcode::QrCode;
use std::sync::mpsc::{
    Receiver,
    Sender,
};

pub struct WalletApp {
    pub init: bool,
    pub is_loading: bool,
    pub is_qr_set: bool,
    pub is_showing_qr: bool,
    pub is_showing_sweep_result: bool,
    pub qr: egui_extras::RetainedImage,
    pub sweep_address: String,
    pub xmr_address_tx: Sender<reqres::XmrRpcAddressResponse>,
    pub xmr_address_rx: Receiver<reqres::XmrRpcAddressResponse>,
    pub xmr_sweep_all_tx: Sender<reqres::XmrRpcSweepAllResponse>,
    pub xmr_sweep_all_rx: Receiver<reqres::XmrRpcSweepAllResponse>,
    pub s_xmr_address: String,
    pub x_xmr_sweep_res: reqres::XmrRpcSweepAllResponse,
}

impl Default for WalletApp {
    fn default() -> Self {
        let (xmr_address_tx, xmr_address_rx) = std::sync::mpsc::channel();
        let (xmr_sweep_all_tx, xmr_sweep_all_rx) = std::sync::mpsc::channel();
        let contents = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
        WalletApp {
            init: false,
            is_loading: false,
            is_qr_set: false,
            is_showing_qr: false,
            is_showing_sweep_result: false,
            qr: egui_extras::RetainedImage::from_image_bytes("qr.png", &contents).unwrap(),
            sweep_address: utils::empty_string(),
            xmr_address_rx,
            xmr_address_tx,
            xmr_sweep_all_rx,
            xmr_sweep_all_tx,
            s_xmr_address: utils::empty_string(),
            x_xmr_sweep_res: Default::default(),
        }
    }
}

impl eframe::App for WalletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(a) = self.xmr_address_rx.try_recv() {
            self.s_xmr_address = a.result.address;
        }
        if let Ok(sweep) = self.xmr_sweep_all_rx.try_recv() {
            self.x_xmr_sweep_res = sweep;
            self.is_loading = false;
        }
        if !self.init {
            send_address_req(self.xmr_address_tx.clone(), ctx.clone());
            self.init = true;
        }
        // Sweep Result
        //-----------------------------------------------------------------------------------
        let mut is_showing_sweep_result = self.is_showing_sweep_result;
        egui::Window::new("Sweep Result")
            .open(&mut is_showing_sweep_result)
            .vscroll(true)
            .show(ctx, |ui| {
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("sweeping...");
                }
                ui.label(format!("{:?}", self.x_xmr_sweep_res));
                if ui.button("Exit").clicked() {
                    self.is_showing_sweep_result = false;
                }
            });

        // QR
        //-----------------------------------------------------------------------------------
        let mut is_showing_qr = self.is_showing_qr;
        egui::Window::new("")
            .open(&mut is_showing_qr)
            .vscroll(true)
            .show(ctx, |ui| {
                if !self.is_qr_set && self.s_xmr_address != utils::empty_string() {
                    let code = QrCode::new(&self.s_xmr_address.clone()).unwrap();
                    let image = code.render::<Luma<u8>>().build();
                    let file_path = format!(
                        "/home/{}/.neveko/qr.png",
                        std::env::var("USER").unwrap_or(String::from("user"))
                    );
                    image.save(&file_path).unwrap();
                    self.init = true;
                    self.is_qr_set = true;
                    let contents = std::fs::read(&file_path).unwrap_or(Vec::new());
                    self.qr =
                        egui_extras::RetainedImage::from_image_bytes("qr.png", &contents).unwrap();
                    ctx.request_repaint();
                }
                self.qr.show(ui);
                let address_label = ui.label("copy: \t");
                ui.text_edit_singleline(&mut self.s_xmr_address)
                    .labelled_by(address_label.id);
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.is_showing_qr = false;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Address");
            ui.label(
                "____________________________________________________________________________\n",
            );
            ui.horizontal(|ui| {
                if ui.button("Show QR").clicked() {
                    self.is_showing_qr = true;
                }
            });
            ui.label("\n\n");
            ui.heading("Sweep Wallet");
            ui.label(
                "____________________________________________________________________________\n",
            );
            ui.horizontal(|ui| {
                let sweep_label = ui.label("send to: \t");
                ui.text_edit_singleline(&mut self.sweep_address)
                    .labelled_by(sweep_label.id);
                if ui.button("Sweep").clicked() {
                    send_sweep_all_req(
                        self.xmr_sweep_all_tx.clone(),
                        ctx.clone(),
                        self.sweep_address.clone(),
                    );
                    self.sweep_address = utils::empty_string();
                    self.is_showing_sweep_result = true;
                    self.is_loading = true;
                }
            });
        });
    }
}

fn send_address_req(tx: Sender<reqres::XmrRpcAddressResponse>, ctx: egui::Context) {
    tokio::spawn(async move {
        let wallet_name = String::from(neveko_core::APP_NAME);
        let wallet_password =
            std::env::var(neveko_core::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
        monero::open_wallet(&wallet_name, &wallet_password).await;
        let address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        monero::close_wallet(&wallet_name, &wallet_password).await;
        let _ = tx.send(address);
        ctx.request_repaint();
    });
}

fn send_sweep_all_req(
    tx: Sender<reqres::XmrRpcSweepAllResponse>,
    ctx: egui::Context,
    address: String,
) {
    tokio::spawn(async move {
        let wallet_name = String::from(neveko_core::APP_NAME);
        let wallet_password =
            std::env::var(neveko_core::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
        monero::open_wallet(&wallet_name, &wallet_password).await;
        let result: reqres::XmrRpcSweepAllResponse = monero::sweep_all(address).await;
        monero::close_wallet(&wallet_name, &wallet_password).await;
        let _ = tx.send(result);
        ctx.request_repaint();
    });
}
