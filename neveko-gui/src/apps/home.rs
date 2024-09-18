#![deny(clippy::all)]
#![forbid(unsafe_code)]

use eframe::egui;
use egui::{
    Color32,
    RichText,
};
use image::Luma;
use neveko_core::*;
use qrcode::QrCode;
use std::{
    sync::mpsc::{
        Receiver,
        Sender,
    },
    time::Duration,
};

pub struct HomeApp {
    /// blocks fetched during last wallet refresh
    blocks_fetched: u64,
    wallet_height: u64,
    connections: utils::Connections,
    core_timeout_tx: Sender<bool>,
    core_timeout_rx: Receiver<bool>,
    is_core_running: bool,
    is_editing_connections: bool,
    is_init: bool,
    is_installing: bool,
    is_qr_set: bool,
    is_loading: bool,
    is_timeout: bool,
    is_showing_qr: bool,
    is_updated: bool,
    // Sender/Receiver for async notifications.
    i2p_status_tx: Sender<i2p::ProxyStatus>,
    i2p_status_rx: Receiver<i2p::ProxyStatus>,
    xmrd_get_info_tx: Sender<reqres::XmrDaemonGetInfoResponse>,
    xmrd_get_info_rx: Receiver<reqres::XmrDaemonGetInfoResponse>,
    xmr_address_tx: Sender<reqres::XmrRpcAddressResponse>,
    xmr_address_rx: Receiver<reqres::XmrRpcAddressResponse>,
    xmr_balance_tx: Sender<reqres::XmrRpcBalanceResponse>,
    xmr_balance_rx: Receiver<reqres::XmrRpcBalanceResponse>,
    xmr_rpc_ver_tx: Sender<reqres::XmrRpcVersionResponse>,
    xmr_rpc_ver_rx: Receiver<reqres::XmrRpcVersionResponse>,
    can_refresh_tx: Sender<bool>,
    can_refresh_rx: Receiver<bool>,
    wallet_height_tx: Sender<reqres::XmrRpcGetHeightResponse>,
    wallet_height_rx: Receiver<reqres::XmrRpcGetHeightResponse>,
    wallet_refresh_tx: Sender<reqres::XmrRpcRefreshResponse>,
    wallet_refresh_rx: Receiver<reqres::XmrRpcRefreshResponse>,
    pub qr: egui_extras::RetainedImage,
    // application state set
    s_xmr_address: reqres::XmrRpcAddressResponse,
    s_xmr_balance: reqres::XmrRpcBalanceResponse,
    s_xmr_rpc_ver: reqres::XmrRpcVersionResponse,
    s_xmrd_get_info: reqres::XmrDaemonGetInfoResponse,
    s_i2p_status: i2p::ProxyStatus,
    s_can_refresh: bool,
    // logos
    logo_i2p: egui_extras::RetainedImage,
    logo_xmr: egui_extras::RetainedImage,
}

impl Default for HomeApp {
    fn default() -> Self {
        let blocks_fetched = 0;
        let connections = Default::default();
        let is_core_running = false;
        let is_editing_connections = false;
        let is_init = true;
        let is_installing = false;
        let is_loading = false;
        let is_qr_set = false;
        let is_showing_qr = false;
        let is_timeout = false;
        let is_updated = false;
        let (core_timeout_tx, core_timeout_rx) = std::sync::mpsc::channel();
        let (xmrd_get_info_tx, xmrd_get_info_rx) = std::sync::mpsc::channel();
        let (xmr_rpc_ver_tx, xmr_rpc_ver_rx) = std::sync::mpsc::channel();
        let (xmr_address_tx, xmr_address_rx) = std::sync::mpsc::channel();
        let (xmr_balance_tx, xmr_balance_rx) = std::sync::mpsc::channel();
        let (wallet_refresh_tx, wallet_refresh_rx) = std::sync::mpsc::channel();
        let (wallet_height_tx, wallet_height_rx) = std::sync::mpsc::channel();
        let (can_refresh_tx, can_refresh_rx) = std::sync::mpsc::channel();
        let (i2p_status_tx, i2p_status_rx) = std::sync::mpsc::channel();
        let contents = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
        let s_xmr_rpc_ver = Default::default();
        let s_xmr_address = Default::default();
        let s_xmr_balance = Default::default();
        let s_xmrd_get_info = Default::default();
        let s_i2p_status = i2p::ProxyStatus::Opening;
        let s_can_refresh = false;
        let c_xmr_logo = std::fs::read("./assets/xmr.png").unwrap_or(Vec::new());
        let logo_xmr =
            egui_extras::RetainedImage::from_image_bytes("./assets/xmr.png", &c_xmr_logo).unwrap();
        let c_i2p_logo = std::fs::read("./assets/i2p.png").unwrap_or(Vec::new());
        let logo_i2p =
            egui_extras::RetainedImage::from_image_bytes("./assets/i2p.png", &c_i2p_logo).unwrap();
        let wallet_height = 0;
        Self {
            blocks_fetched,
            connections,
            core_timeout_rx,
            core_timeout_tx,
            is_core_running,
            is_editing_connections,
            is_init,
            is_installing,
            is_loading,
            is_qr_set,
            is_showing_qr,
            is_timeout,
            is_updated,
            xmrd_get_info_tx,
            xmrd_get_info_rx,
            xmr_rpc_ver_tx,
            xmr_rpc_ver_rx,
            xmr_address_tx,
            xmr_address_rx,
            xmr_balance_tx,
            xmr_balance_rx,
            i2p_status_tx,
            i2p_status_rx,
            can_refresh_rx,
            can_refresh_tx,
            qr: egui_extras::RetainedImage::from_image_bytes("qr.png", &contents).unwrap(),
            wallet_height,
            wallet_height_rx,
            wallet_height_tx,
            wallet_refresh_rx,
            wallet_refresh_tx,
            // state of self defaults
            s_xmr_address,
            s_xmr_balance,
            s_xmr_rpc_ver,
            s_xmrd_get_info,
            s_i2p_status,
            // misc state
            s_can_refresh,
            // logo
            logo_xmr,
            logo_i2p,
        }
    }
}

impl eframe::App for HomeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(ver) = self.xmr_rpc_ver_rx.try_recv() {
            self.s_xmr_rpc_ver = ver;
        }
        if let Ok(address) = self.xmr_address_rx.try_recv() {
            self.s_xmr_address = address;
        }
        if let Ok(wallet_height) = self.wallet_height_rx.try_recv() {
            self.wallet_height = wallet_height.result.height;
        }
        if let Ok(wallet_refresh) = self.wallet_refresh_rx.try_recv() {
            self.blocks_fetched = wallet_refresh.result.blocks_fetched;
        }
        if let Ok(balance) = self.xmr_balance_rx.try_recv() {
            self.s_xmr_balance = balance;
        }
        if let Ok(can_refresh) = self.can_refresh_rx.try_recv() {
            self.s_can_refresh = can_refresh;
        }
        if let Ok(i2p_status) = self.i2p_status_rx.try_recv() {
            self.s_i2p_status = i2p_status;
        }
        if let Ok(info) = self.xmrd_get_info_rx.try_recv() {
            self.s_xmrd_get_info = info;
        }
        if let Ok(timeout) = self.core_timeout_rx.try_recv() {
            self.is_timeout = true;
            if timeout {
                self.is_loading = false;
                self.is_core_running = false;
                self.is_installing = false;
            }
        }

        // I2P Address QR
        //-----------------------------------------------------------------------------------
        let mut is_showing_qr = self.is_showing_qr;
        egui::Window::new("i2p qr")
            .open(&mut is_showing_qr)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                let mut i2p_address =
                    i2p::get_destination(i2p::ServerTunnelType::App).unwrap_or_default();
                if !self.is_qr_set && !i2p_address.is_empty() {
                    let code = QrCode::new(&i2p_address).unwrap();
                    let image = code.render::<Luma<u8>>().build();
                    let file_path = format!(
                        "/home/{}/.neveko/i2p-qr.png",
                        std::env::var("USER").unwrap_or(String::from("user"))
                    );
                    image.save(&file_path).unwrap();
                    self.is_qr_set = true;
                    let contents = std::fs::read(&file_path).unwrap_or(Vec::new());
                    self.qr = egui_extras::RetainedImage::from_image_bytes("i2p-qr.png", &contents)
                        .unwrap();
                    ctx.request_repaint();
                }
                self.qr.show(ui);
                let address_label = ui.label("copy: \t");
                ui.text_edit_singleline(&mut i2p_address)
                    .labelled_by(address_label.id);
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.is_showing_qr = false;
                }
            });

        // Connection Manager window
        //-----------------------------------------------------------------------------------
        let mut is_editing_connections = self.is_editing_connections;
        egui::Window::new("connection")
            .open(&mut is_editing_connections)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Connection Manager");
                ui.horizontal(|ui| {
                    let cm_daemon_label = ui.label("daemon host:\t");
                    ui.text_edit_singleline(&mut self.connections.daemon_host)
                        .labelled_by(cm_daemon_label.id);
                });
                ui.horizontal(|ui| {
                    let cm_rpc_label = ui.label("rpc host:  \t\t\t");
                    ui.text_edit_singleline(&mut self.connections.rpc_host)
                        .labelled_by(cm_rpc_label.id);
                });
                ui.horizontal(|ui| {
                    let cm_user_label = ui.label("rpc user:  \t\t\t");
                    ui.text_edit_singleline(&mut self.connections.rpc_username)
                        .labelled_by(cm_user_label.id);
                });
                ui.horizontal(|ui| {
                    let cm_cred_label = ui.label("rpc cred:  \t\t\t");
                    ui.text_edit_singleline(&mut self.connections.rpc_credential)
                        .labelled_by(cm_cred_label.id);
                });
                ui.horizontal(|ui| {
                    let cm_db_dir_label = ui.label("db path:   \t\t\t");
                    ui.text_edit_singleline(&mut self.connections.blockchain_dir)
                        .labelled_by(cm_db_dir_label.id);
                });
                ui.horizontal(|ui| {
                    let cm_xmr_dir_label = ui.label("xmr path:\t\t\t");
                    ui.text_edit_singleline(&mut self.connections.monero_location)
                        .labelled_by(cm_xmr_dir_label.id);
                });
                if self.connections.is_i2p_advanced {
                    ui.horizontal(|ui| {
                        let cm_i2p_proxy_label = ui.label("i2p proxy host: \t");
                        ui.text_edit_singleline(&mut self.connections.i2p_proxy_host)
                            .labelled_by(cm_i2p_proxy_label.id);
                    });
                    ui.horizontal(|ui| {
                        let cm_i2p_socks_label = ui.label("i2p socks host: \t");
                        ui.text_edit_singleline(&mut self.connections.i2p_socks_host)
                            .labelled_by(cm_i2p_socks_label.id);
                    });
                }
                let mut is_remote_node = self.connections.is_remote_node;
                if ui.checkbox(&mut is_remote_node, "remote node").changed() {
                    self.connections.is_remote_node = !self.connections.is_remote_node;
                    log::debug!("is remote node: {}", self.connections.is_remote_node);
                }
                let mut is_i2p_advanced = self.connections.is_i2p_advanced;
                if ui
                    .checkbox(&mut is_i2p_advanced, "i2p advanced mode")
                    .changed()
                {
                    self.connections.is_i2p_advanced = !self.connections.is_i2p_advanced;
                    log::debug!("is i2p advanced mode: {}", self.connections.is_i2p_advanced);
                }
                // TODO(c2m): uncomment this to add mainnet flag to gui
                // let mut is_mainnet = self.connections.mainnet;
                // if ui.checkbox(&mut is_mainnet, "mainnet").changed() {
                //     self.connections.mainnet = !self.connections.mainnet;
                //     log::debug!("is mainnet: {}", self.connections.mainnet);
                // }
                if ui.button("Start/Restart").clicked() {
                    self.is_editing_connections = false;
                    utils::kill_child_processes(true);
                    utils::start_core(&self.connections);
                    self.is_loading = true;
                    start_core_timeout(self.core_timeout_tx.clone(), ctx.clone());
                }
                if ui.button("Exit").clicked() {
                    self.is_editing_connections = false;
                    self.is_loading = false;
                }
            });

        //----------------------------------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.is_updated {
                if !self.is_init {
                    send_ver_req(self.xmr_rpc_ver_tx.clone(), ctx.clone());
                    send_wallet_req(
                        self.xmr_address_tx.clone(),
                        self.xmr_balance_tx.clone(),
                        self.wallet_refresh_tx.clone(),
                        self.wallet_height_tx.clone(),
                        ctx.clone()
                    );
                    send_i2p_status_req(self.i2p_status_tx.clone(), ctx.clone());
                    send_xmrd_get_info_req(self.xmrd_get_info_tx.clone(), ctx.clone());
                }
                self.is_updated = true;
                let is_initializing = self.is_init;
                send_reset_refresh(self.can_refresh_tx.clone(), ctx.clone(), is_initializing);
                self.is_init = false;
            }
            if self.s_can_refresh {
                self.is_updated = false;
                self.s_can_refresh = false;
                if self.s_xmr_rpc_ver.result.version != 0 {
                        self.is_loading = false;
                }
            }
            let mut str_i2p_status = String::from("offline");
            let mut hover_txt = String::from("http proxy tunnel is closed");
            let mut color = Color32::from_rgb(255, 0, 0);
            if self.s_i2p_status == i2p::ProxyStatus::Open {
                str_i2p_status = String::from("online");
                color = Color32::from_rgb(0, 255, 0);
                hover_txt = String::from("http proxy tunnel is open but connectivity is not guaranteed");
            }
            if self.connections.is_i2p_advanced {
                str_i2p_status = String::from("remote proxy");
            }
            ui.horizontal(|ui| {
                self.logo_i2p.show(ui);
                ui.horizontal(|ui| {
                    let i2p_address = i2p::get_destination(i2p::ServerTunnelType::App)
                        .unwrap_or_default();
                    ui.label(
                        RichText::new(format!("- status: {}\n- address: {}", str_i2p_status, i2p_address))
                            .size(16.0)
                            .color(color),
                    ).on_hover_text(hover_txt);
                });
            });
            ui.horizontal(|ui| {
                if ui.button("Show QR").clicked() {
                    self.is_showing_qr = true;
                }
            });
            ui.label("____________________________________________________________________\n");
            ui.label("\n\n");
            ui.horizontal(|ui| {
                self.logo_xmr.show(ui);
                let address = &self.s_xmr_address.result.address;
                let blocks_fetched = self.blocks_fetched;
                let height = self.wallet_height;
                let unlocked_balance = self.s_xmr_balance.result.unlocked_balance;
                let locked_balance = self.s_xmr_balance.result.balance - unlocked_balance;
                let unlock_time = self.s_xmr_balance.result.blocks_to_unlock * crate::BLOCK_TIME_IN_SECS_EST;
                let xmrd_info: &reqres::XmrDaemonGetInfoResult = &self.s_xmrd_get_info.result;
                let sync = if xmrd_info.height != 0 { (height / xmrd_info.height) * 100 } else { 0 };
                let free_space = xmrd_info.free_space / crate::BYTES_IN_GB;
                let db_size = xmrd_info.database_size / crate::BYTES_IN_GB;
                let ver = self.s_xmr_rpc_ver.result.version;
                ui.label(format!("- rpc version: {}\n- wallet sync: {}%\n- blocks fetched: {}\n- address: {}\n- balance: {} piconero(s)\n- locked balance: {} piconero(s)\n- unlock time (secs): {}\n- daemon info\n\t- net type: {}\n\t- current hash: {}\n\t- height: {}\n\t- synced: {}\n\t- blockchain size : ~{} GB\n\t- free space : ~{} GB\n\t- version: {}\n", 
                    ver, sync, blocks_fetched, address, unlocked_balance, locked_balance,
                    unlock_time, xmrd_info.nettype, xmrd_info.top_block_hash, xmrd_info.height, xmrd_info.synchronized,
                    db_size, free_space, xmrd_info.version));
            });
            ui.label("____________________________________________________________________\n");
            ui.label("\n");
            if self.is_loading {
                let label = if self.is_installing { "installing software" } else { "starting neveko-core..." };
                ui.add(egui::Spinner::new());
                ui.label(label);
            }
            if !self.is_core_running && self.s_xmr_rpc_ver.result.version == 0 {
                if !self.is_loading {
                    if ui.button("Edit Connections").clicked() {
                        self.is_editing_connections = true;
                    }
                }
            }
        });
    }
}

// Async requests to neveko_core module
//-------------------------------------------------------------------------------------------------
fn send_xmrd_get_info_req(tx: Sender<reqres::XmrDaemonGetInfoResponse>, ctx: egui::Context) {
    tokio::spawn(async move {
        let remote_var = std::env::var(neveko_core::GUI_REMOTE_NODE).unwrap_or(String::new());
        if remote_var == String::from(neveko_core::GUI_SET_REMOTE_NODE) {
            let p_info = monero::p_get_info().await;
            let info = p_info.unwrap_or(Default::default());
            let _ = tx.send(info);
        } else {
            let info = monero::get_info().await;
            let _ = tx.send(info);
        }
        ctx.request_repaint();
    });
}

fn send_ver_req(tx: Sender<reqres::XmrRpcVersionResponse>, ctx: egui::Context) {
    tokio::spawn(async move {
        let ver: reqres::XmrRpcVersionResponse = monero::get_version().await;
        let _ = tx.send(ver);
        ctx.request_repaint();
    });
}

fn send_wallet_req(
    address_tx: Sender<reqres::XmrRpcAddressResponse>,
    balance_tx: Sender<reqres::XmrRpcBalanceResponse>,
    wallet_refresh_tx: Sender<reqres::XmrRpcRefreshResponse>,
    wallet_height_tx: Sender<reqres::XmrRpcGetHeightResponse>,
    ctx: egui::Context,
) {
    tokio::spawn(async move {
        let wallet_name = String::from(neveko_core::APP_NAME);
        let wallet_password =
            std::env::var(neveko_core::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
        monero::open_wallet(&wallet_name, &wallet_password).await;
        let address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        let refresh: reqres::XmrRpcRefreshResponse = monero::refresh().await;
        let balance: reqres::XmrRpcBalanceResponse = monero::get_balance().await;
        let wallet_height: reqres::XmrRpcGetHeightResponse = monero::get_wallet_height().await;
        monero::close_wallet(&wallet_name, &wallet_password).await;
        let _ = address_tx.send(address);
        let _ = balance_tx.send(balance);
        let _ = wallet_refresh_tx.send(refresh);
        let _ = wallet_height_tx.send(wallet_height);
        ctx.request_repaint();
    });
}

fn send_i2p_status_req(tx: Sender<i2p::ProxyStatus>, ctx: egui::Context) {
    tokio::spawn(async move {
        let status = i2p::check_connection().await;
        let _ = tx.send(status.unwrap_or(i2p::ProxyStatus::Opening));
        ctx.request_repaint();
    });
}

// refresh rate for the home screen
fn send_reset_refresh(tx: Sender<bool>, ctx: egui::Context, init: bool) {
    tokio::spawn(async move {
        log::debug!("refreshing home screen");
        if !init {
            tokio::time::sleep(Duration::from_secs(120)).await;
        }
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn start_core_timeout(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(
            crate::START_CORE_TIMEOUT_SECS,
        ))
        .await;
        log::error!("start neveko-core timeout");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}
