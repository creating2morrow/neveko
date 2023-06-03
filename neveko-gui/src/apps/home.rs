#![deny(clippy::all)]
#![forbid(unsafe_code)]

use eframe::egui;
use neveko_core::*;
use std::{
    sync::mpsc::{
        Receiver,
        Sender,
    },
    time::Duration,
};

pub struct HomeApp {
    connections: utils::Connections,
    core_timeout_tx: Sender<bool>,
    core_timeout_rx: Receiver<bool>,
    has_install_failed: bool,
    installations: utils::Installations,
    installation_tx: Sender<bool>,
    installation_rx: Receiver<bool>,
    is_core_running: bool,
    is_editing_connections: bool,
    is_init: bool,
    is_installing: bool,
    is_loading: bool,
    is_timeout: bool,
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
        let connections = Default::default();
        let has_install_failed = false;
        let installations = Default::default();
        let is_core_running = false;
        let is_editing_connections = false;
        let is_init = true;
        let is_installing = false;
        let is_loading = false;
        let is_timeout = false;
        let is_updated = false;
        let (core_timeout_tx, core_timeout_rx) = std::sync::mpsc::channel();
        let (xmrd_get_info_tx, xmrd_get_info_rx) = std::sync::mpsc::channel();
        let (xmr_rpc_ver_tx, xmr_rpc_ver_rx) = std::sync::mpsc::channel();
        let (xmr_address_tx, xmr_address_rx) = std::sync::mpsc::channel();
        let (xmr_balance_tx, xmr_balance_rx) = std::sync::mpsc::channel();
        let (can_refresh_tx, can_refresh_rx) = std::sync::mpsc::channel();
        let (i2p_status_tx, i2p_status_rx) = std::sync::mpsc::channel();
        let (installation_tx, installation_rx) = std::sync::mpsc::channel();
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
        Self {
            connections,
            core_timeout_rx,
            core_timeout_tx,
            has_install_failed,
            installations,
            installation_rx,
            installation_tx,
            is_core_running,
            is_editing_connections,
            is_init,
            is_installing,
            is_loading,
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
        if let Ok(install) = self.installation_rx.try_recv() {
            self.is_installing = !install;
            if !install && self.is_loading {
                self.has_install_failed = true
            }
            self.is_loading = false;
        }
        if let Ok(timeout) = self.core_timeout_rx.try_recv() {
            self.is_timeout = true;
            if timeout {
                self.is_loading = false;
                self.is_core_running = false;
                self.is_installing = false;
            }
        }

        // Installation Error window
        //-----------------------------------------------------------------------------------
        let mut has_install_failed = self.has_install_failed;
        egui::Window::new("Error")
            .open(&mut has_install_failed)
            .vscroll(false)
            .show(&ctx, |ui| {
                ui.heading("Installation Failure");
                if ui.button("Exit").clicked() {
                    self.has_install_failed = false;
                    self.is_installing = false;
                    self.is_loading = false;
                }
            });

        // Connection Manager window
        //-----------------------------------------------------------------------------------
        let mut is_editing_connections = self.is_editing_connections;
        egui::Window::new("Connection Manager")
            .open(&mut is_editing_connections)
            .vscroll(true)
            .show(&ctx, |ui| {
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
                ui.horizontal(|ui| {
                    let cm_i2p_dir_label = ui.label("i2p-zero path: \t");
                    ui.text_edit_singleline(&mut self.connections.i2p_zero_dir)
                        .labelled_by(cm_i2p_dir_label.id);
                });
                let mut is_mainnet = self.connections.mainnet;
                if ui.checkbox(&mut is_mainnet, "mainnet").changed() {
                    self.connections.mainnet = !self.connections.mainnet;
                    log::debug!("is mainnet: {}", self.connections.mainnet);
                }
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

        // Installation Manager window
        //-----------------------------------------------------------------------------------
        let mut is_installing = self.is_installing;
        egui::Window::new("Installation Manager")
            .open(&mut is_installing)
            .vscroll(true)
            .show(&ctx, |ui| {
                let mut wants_i2p_zero = self.installations.i2p_zero;
                let mut wants_xmr = self.installations.xmr;
                if ui.checkbox(&mut wants_i2p_zero, "i2p-zero").changed() {
                    self.installations.i2p_zero = !self.installations.i2p_zero;
                }
                if ui.checkbox(&mut wants_xmr, "xmr").changed() {
                    self.installations.xmr = !self.installations.xmr;
                }
                let install = &self.installations;
                if install.i2p_zero || install.xmr {
                    if !self.is_loading {
                        if ui.button("Install").clicked() {
                            self.is_loading = true;
                            install_software_req(
                                self.installation_tx.clone(),
                                ctx.clone(),
                                &self.installations,
                            );
                        }
                    }
                }
                if ui.button("Exit").clicked() {
                    self.is_installing = false;
                    self.is_loading = false;
                }
            });

        //----------------------------------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.is_updated {
                send_ver_req(self.xmr_rpc_ver_tx.clone(), ctx.clone());
                send_address_req(self.xmr_address_tx.clone(), ctx.clone());
                send_balance_req(self.xmr_balance_tx.clone(), ctx.clone());
                send_i2p_status_req(self.i2p_status_tx.clone(), ctx.clone());
                send_xmrd_get_info_req(self.xmrd_get_info_tx.clone(), ctx.clone());
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
            if self.s_i2p_status == i2p::ProxyStatus::Open {
                str_i2p_status = String::from("online");
            }
            ui.horizontal(|ui| {
                self.logo_i2p.show(ui);
                ui.horizontal(|ui| {
                    let i2p_address = i2p::get_destination();
                    ui.label(format!("- status: {}\n- address: {}", str_i2p_status, i2p_address));
                });
            });
            ui.label("____________________________________________________________________\n");
            ui.label("\n\n");
            ui.horizontal(|ui| {
                self.logo_xmr.show(ui);
                let address = &self.s_xmr_address.result.address;
                let unlocked_balance = self.s_xmr_balance.result.unlocked_balance;
                let locked_balance = self.s_xmr_balance.result.balance - unlocked_balance;
                let unlock_time = self.s_xmr_balance.result.blocks_to_unlock * crate::BLOCK_TIME_IN_SECS_EST;
                let xmrd_info: &reqres::XmrDaemonGetInfoResult = &self.s_xmrd_get_info.result;
                let free_space = xmrd_info.free_space / crate::BYTES_IN_GB;
                let db_size = xmrd_info.database_size / crate::BYTES_IN_GB;
                ui.label(format!("- rpc version: {}\n- address: {}\n- balance: {} piconero(s)\n- locked balance: {} piconero(s)\n- unlock time (secs): {}\n- daemon info\n\t- net type: {}\n\t- current hash: {}\n\t- height: {}\n\t- synced: {}\n\t- blockchain size : ~{} GB\n\t- free space : ~{} GB\n\t- version: {}\n", 
                    self.s_xmr_rpc_ver.result.version, address, unlocked_balance, locked_balance,
                    unlock_time, xmrd_info.nettype, xmrd_info.top_block_hash, xmrd_info.height, xmrd_info.synchronized,
                    db_size, free_space, xmrd_info.version));
                    // TODO(c2m): pull in more xmr blockchain information?
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
            if !self.is_core_running && !self.is_installing
            && (self.s_xmr_rpc_ver.result.version == 0 || self.s_i2p_status == i2p::ProxyStatus::Opening) {
                if !self.is_loading {
                    if ui.button("Install Software").clicked() {
                        self.is_installing = true;
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
        let info: reqres::XmrDaemonGetInfoResponse = monero::get_info().await;
        let _ = tx.send(info);
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

fn send_address_req(tx: Sender<reqres::XmrRpcAddressResponse>, ctx: egui::Context) {
    tokio::spawn(async move {
        let address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        let _ = tx.send(address);
        ctx.request_repaint();
    });
}

fn send_balance_req(tx: Sender<reqres::XmrRpcBalanceResponse>, ctx: egui::Context) {
    tokio::spawn(async move {
        let balance: reqres::XmrRpcBalanceResponse = monero::get_balance().await;
        let _ = tx.send(balance);
        ctx.request_repaint();
    });
}

fn send_i2p_status_req(tx: Sender<i2p::ProxyStatus>, ctx: egui::Context) {
    tokio::spawn(async move {
        let status = i2p::check_connection().await;
        let _ = tx.send(status);
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

fn install_software_req(
    tx: Sender<bool>,
    ctx: egui::Context,
    installations: &utils::Installations,
) {
    let req_install: utils::Installations = utils::Installations {
        i2p_zero: installations.i2p_zero,
        xmr: installations.xmr,
    };
    tokio::spawn(async move {
        let did_install = utils::install_software(req_install).await;
        let _ = tx.send(did_install);
        ctx.request_repaint();
    });
}
