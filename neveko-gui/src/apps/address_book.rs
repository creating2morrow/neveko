use neveko_core::*;
use std::sync::mpsc::{
    Receiver,
    Sender,
};

use crate::ADD_CONTACT_TIMEOUT_SECS;

// TODO(c2m): better error handling with and error_tx/error_rx channel
//       hook into the error thread and show toast messages as required

/// Maintain the message for sending in this struct
struct Compose {
    message: String,
    to: String,
}

impl Default for Compose {
    fn default() -> Self {
        Compose {
            message: utils::empty_string(),
            to: utils::empty_string(),
        }
    }
}

/// The AddressBookApp unfornuately does more than that.
///
/// Herein lies the logic for filtering contacts, generating JWPs,
///
/// transaction proofs, etc. Once a contact has a valid JWP that has
///
/// not yet expired the `Compose` button will appear by their i2p address.
///
/// NOTE: the `Sign Key` must be pressed for trusted contacts before a
///
/// message can be composed.
pub struct AddressBookApp {
    add_nick: String,
    approve_contact: bool,
    approve_payment: bool,
    added: bool,
    can_transfer: bool,
    can_transfer_tx: Sender<bool>,
    can_transfer_rx: Receiver<bool>,
    compose: Compose,
    contact: String,
    find_contact: String,
    contacts: Vec<models::Contact>,
    contacts_init: bool,
    contact_add_tx: Sender<models::Contact>,
    contact_add_rx: Receiver<models::Contact>,
    contact_info_tx: Sender<models::Contact>,
    contact_info_rx: Receiver<models::Contact>,
    contact_timeout_tx: Sender<bool>,
    contact_timeout_rx: Receiver<bool>,
    invoice_tx: Sender<reqres::Invoice>,
    invoice_rx: Receiver<reqres::Invoice>,
    is_adding: bool,
    is_composing: bool,
    is_approving_jwp: bool,
    is_estimating_fee: bool,
    is_pinging: bool,
    is_loading: bool,
    is_message_sent: bool,
    is_payment_processed: bool,
    is_timeout: bool,
    payment_tx: Sender<bool>,
    payment_rx: Receiver<bool>,
    showing_status: bool,
    status: utils::ContactStatus,
    send_message_tx: Sender<bool>,
    send_message_rx: Receiver<bool>,
    s_contact: models::Contact,
    s_invoice: reqres::Invoice,
    s_added_contact: models::Contact,
}

impl Default for AddressBookApp {
    fn default() -> Self {
        let (can_transfer_tx, can_transfer_rx) = std::sync::mpsc::channel();
        let (contact_add_tx, contact_add_rx) = std::sync::mpsc::channel();
        let (contact_info_tx, contact_info_rx) = std::sync::mpsc::channel();
        let (contact_timeout_tx, contact_timeout_rx) = std::sync::mpsc::channel();
        let (invoice_tx, invoice_rx) = std::sync::mpsc::channel();
        let (payment_tx, payment_rx) = std::sync::mpsc::channel();
        let (send_message_tx, send_message_rx) = std::sync::mpsc::channel();
        AddressBookApp {
            add_nick: utils::empty_string(),
            approve_contact: false,
            approve_payment: false,
            added: false,
            can_transfer: false,
            can_transfer_rx,
            can_transfer_tx,
            compose: Default::default(),
            contact: utils::empty_string(),
            contacts: Vec::new(),
            contacts_init: false,
            contact_add_tx,
            contact_add_rx,
            contact_info_tx,
            contact_info_rx,
            contact_timeout_tx,
            contact_timeout_rx,
            find_contact: utils::empty_string(),
            invoice_tx,
            invoice_rx,
            is_adding: false,
            is_composing: false,
            is_approving_jwp: false,
            is_estimating_fee: false,
            is_loading: false,
            is_message_sent: false,
            is_pinging: false,
            is_payment_processed: false,
            is_timeout: false,
            payment_rx,
            payment_tx,
            send_message_tx,
            send_message_rx,
            status: Default::default(),
            showing_status: false,
            s_contact: Default::default(),
            s_added_contact: Default::default(),
            s_invoice: Default::default(),
        }
    }
}

impl eframe::App for AddressBookApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hook into async channel threads
        //-----------------------------------------------------------------------------------
        if let Ok(contact_info) = self.contact_info_rx.try_recv() {
            self.s_contact = contact_info;
            if self.s_contact.xmr_address != utils::empty_string() && !self.showing_status {
                self.approve_contact = true;
            }
            if self.showing_status {
                self.is_pinging = false;
            }
        }

        if let Ok(added_contact) = self.contact_add_rx.try_recv() {
            self.s_added_contact = added_contact;
            if self.s_added_contact.cid != utils::empty_string() {
                self.added = true;
                self.is_loading = false;
            }
        }

        if let Ok(timeout) = self.contact_timeout_rx.try_recv() {
            self.is_timeout = true;
            if timeout {
                self.is_loading = false;
                self.is_adding = false;
                self.approve_contact = false;
                self.contact = utils::empty_string();
            }
        }

        if let Ok(invoice) = self.invoice_rx.try_recv() {
            self.s_invoice = invoice;
            if self.s_invoice.pay_threshold > 0 {
                send_can_transfer_req(
                    self.can_transfer_tx.clone(),
                    ctx.clone(),
                    self.s_invoice.pay_threshold,
                );
                self.is_estimating_fee = true;
            }
        }

        if let Ok(payment) = self.payment_rx.try_recv() {
            self.is_payment_processed = payment;
            if self.is_payment_processed {
                self.is_loading = false;
                self.approve_payment = false;
                self.showing_status = false;
            }
        }

        if let Ok(message) = self.send_message_rx.try_recv() {
            self.is_message_sent = message;
            if self.is_message_sent {
                self.is_loading = false;
                self.is_composing = false;
                self.compose.message = utils::empty_string();
            }
        }

        if let Ok(can_transfer) = self.can_transfer_rx.try_recv() {
            self.can_transfer = can_transfer;
            self.is_estimating_fee = false;
        }

        // initial contact load
        if !self.contacts_init {
            self.contacts = contact::find_all();
            self.contacts_init = true;
        }

        // Compose window
        //-----------------------------------------------------------------------------------
        let mut is_composing = self.is_composing;
        egui::Window::new("compose")
            .open(&mut is_composing)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Compose Message");
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("sending message...");
                }
                ui.horizontal(|ui| ui.label(format!("to: {}", self.status.i2p)));
                ui.horizontal(|ui| {
                    let message_label = ui.label("msg: ");
                    ui.text_edit_multiline(&mut self.compose.message)
                        .labelled_by(message_label.id);
                });
                if !self.is_loading {
                    self.compose.to = self.status.i2p.clone();
                    if self.status.jwp != utils::empty_string() {
                        if ui.button("Send").clicked() {
                            self.is_loading = true;
                            send_message_req(
                                self.send_message_tx.clone(),
                                ctx.clone(),
                                self.compose.message.clone(),
                                self.compose.to.clone(),
                                self.status.jwp.clone(),
                            );
                        }
                    }
                    if ui.button("Exit").clicked() {
                        self.is_composing = false;
                    }
                }
            });

        // Payment approval window
        //-----------------------------------------------------------------------------------
        let mut is_approving_payment =
            self.approve_payment && self.s_invoice.address != utils::empty_string();
        let address = self.s_invoice.address.clone();
        let amount = self.s_invoice.pay_threshold;
        let expire = self.s_invoice.conf_threshold;
        egui::Window::new("approve payment")
            .open(&mut is_approving_payment)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Approve Payment for JWP");
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("creating jwp. please wait...");
                }
                if self.is_estimating_fee {
                    ui.add(egui::Spinner::new());
                    ui.label("running neveko jwp fee estimator...");
                }
                ui.heading(self.status.i2p.clone());
                ui.label(format!("pay to: {}", address));
                ui.label(format!("amount: {} piconero(s)", amount));
                ui.label(format!("expiration: {} blocks", expire));
                let show_approve = self.s_invoice.address != utils::empty_string()
                    && self.can_transfer && !self.is_estimating_fee;
                if !self.is_loading {
                    if show_approve {
                        if ui.button("Approve").clicked() {
                            // activate xmr "transfer", check the hash, update db and refresh
                            // Note it is simply disabled on insufficient funds as calcd by fee
                            // estimator
                            let d: reqres::Destination = reqres::Destination { address, amount };
                            send_payment_req(
                                self.payment_tx.clone(),
                                ctx.clone(),
                                d,
                                self.status.i2p.clone(),
                                expire,
                                false,
                            );
                            self.is_loading = true;
                            self.is_approving_jwp = false;
                        }
                    }
                }
                // TODO(c2m): add payment timeout error handling to prevent infinite loading window
                if ui.button("Exit").clicked() {
                    self.approve_payment = false;
                    self.is_loading = false;
                    self.is_approving_jwp = false;
                }
            });

        // Contact status window
        //-----------------------------------------------------------------------------------
        let mut is_showing_status = self.showing_status;
        egui::Window::new("contact status")
            .open(&mut is_showing_status)
            .title_bar(false)
            .vscroll(true)
            .title_bar(false)
            .id(egui::Id::new(self.status.i2p.clone()))
            .show(&ctx, |ui| {
                ui.heading(&self.status.i2p);
                if self.is_pinging || self.is_loading {
                    let spinner_text = if self.is_loading {
                        "retrying payment proof... "
                    } else {
                        "pinging..."
                    };
                    ui.add(egui::Spinner::new());
                    ui.label(spinner_text);
                }
                let status = if self.s_contact.xmr_address != utils::empty_string() {
                    "online"
                } else {
                    "offline"
                };
                ui.label(format!("status: {}", status));
                ui.label(format!("nick: {}", self.status.nick));
                ui.label(format!("tx proof: {}", self.status.txp));
                ui.label(format!("jwp: {}", self.status.jwp));
                ui.label(format!("expiration: {}", self.status.h_exp));
                ui.label(format!("signed key: {}", self.status.signed_key));
                if self.status.jwp == utils::empty_string()
                    && !self.is_pinging
                    && status == "online"
                    && self.status.txp == utils::empty_string()
                {
                    if ui.button("Create JWP").clicked() {
                        self.s_invoice = Default::default();
                        send_invoice_req(
                            self.invoice_tx.clone(),
                            ctx.clone(),
                            self.status.i2p.clone(),
                        );
                        self.approve_payment = true;
                        self.showing_status = false;
                        self.is_approving_jwp = true;
                    }
                }
                if !self.status.signed_key {
                    if ui.button("Sign Key").clicked() {
                        contact::trust_gpg(self.status.i2p.clone());
                        utils::write_gui_db(
                            String::from(crate::GUI_SIGNED_GPG_DB_KEY),
                            self.status.i2p.clone(),
                            String::from(crate::SIGNED_GPG_KEY),
                        );
                        self.showing_status = false;
                    }
                }
                let failed_to_prove = self.status.txp != utils::empty_string()
                    && self.status.jwp == utils::empty_string();
                if self.status.jwp != utils::empty_string() || failed_to_prove {
                    if ui.button("Clear stale JWP").clicked() {
                        utils::clear_gui_db(String::from("gui-txp"), self.status.i2p.clone());
                        utils::clear_gui_db(String::from("gui-jwp"), self.status.i2p.clone());
                        utils::clear_gui_db(String::from("gui-exp"), self.status.i2p.clone());
                        self.showing_status = false;
                    }
                }
                if self.status.txp != utils::empty_string()
                    && self.status.jwp == utils::empty_string()
                    && status == "online"
                {
                    if ui.button("Prove Retry").clicked() {
                        send_payment_req(
                            self.payment_tx.clone(),
                            ctx.clone(),
                            Default::default(),
                            self.status.i2p.clone(),
                            expire as u64,
                            true,
                        );
                        self.is_loading = true;
                    }
                }
                ui.horizontal(|ui| {
                    let nick_label = ui.label("nick: ");
                    ui.text_edit_singleline(&mut self.add_nick)
                        .labelled_by(nick_label.id);
                });
                if ui.button("Change nick").clicked() {
                    change_nick_req(self.status.i2p.clone(), self.add_nick.clone());
                    self.add_nick = utils::empty_string();
                }
                if ui.button("Exit").clicked() {
                    self.showing_status = false;
                    self.is_loading = false;
                    self.is_approving_jwp = false;
                }
            });

        // Main panel for adding contacts
        //-----------------------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_approving_jwp {
                ui.add(egui::Spinner::new());
            }
            ui.heading("Add Contact");
            ui.label(
                "____________________________________________________________________________\n",
            );
            ui.horizontal(|ui| {
                let contact_label = ui.label("contact: ");
                ui.text_edit_singleline(&mut self.contact)
                    .labelled_by(contact_label.id);
            });
            let mut is_approved = self.approve_contact;
            let mut is_added = self.added;
            let is_loading = self.is_loading;
            let i2p_address = self.s_contact.i2p_address.clone();
            let is_vendor = self.s_contact.is_vendor;
            let xmr_address = self.s_contact.xmr_address.clone();
            let gpg_key = self.s_contact.gpg_key.iter().cloned().collect();

            // Contact added confirmation screen
            //-----------------------------------------------------------------------------------
            egui::Window::new("added contact")
                .open(&mut is_added)
                .title_bar(false)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.heading("Added contact");
                    ui.label(format!("i2p address: {}", self.s_added_contact.i2p_address));
                    if ui.button("Exit").clicked() {
                        self.added = false;
                        self.contact = utils::empty_string();
                        self.is_adding = false;
                        self.approve_contact = false;
                        self.contacts = contact::find_all();
                        for c in &self.contacts {
                            ui.label(format!("{}", c.i2p_address));
                        }
                    }
                });

            // Contact approval screen
            //-----------------------------------------------------------------------------------
            egui::Window::new("approve contact")
                .open(&mut is_approved)
                .title_bar(false)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.heading("Approve Contact");
                    if is_loading {
                        ui.add(egui::Spinner::new());
                        ui.label("adding contact...");
                    }
                    ui.label(format!("i2p: {}", i2p_address));
                    ui.label(format!("xmr: {}", xmr_address));
                    ui.label(format!(
                        "gpg: {}",
                        String::from_utf8(gpg_key).unwrap_or(utils::empty_string())
                    ));
                    ui.horizontal(|ui| {
                        if !is_loading {
                            if ui.button("Approve").clicked() {
                                self.is_loading = true;
                                self.approve_contact = false;
                                self.is_adding = false;
                                let c_contact: models::Contact = models::Contact {
                                    cid: self.s_contact.cid.clone(),
                                    i2p_address,
                                    is_vendor,
                                    xmr_address,
                                    gpg_key: self.s_contact.gpg_key.iter().cloned().collect(),
                                };
                                send_create_contact_req(
                                    self.contact_add_tx.clone(),
                                    ctx.clone(),
                                    c_contact,
                                );
                            }
                            if ui.button("Exit").clicked() {
                                self.approve_contact = false;
                            }
                        }
                    });
                });
            if self.is_adding {
                ui.add(egui::Spinner::new());
            }
            if !self.is_adding && self.contact.contains(".b32.i2p") {
                if ui.button("Add").clicked() {
                    // Get the contacts information from the /share API
                    let contact = self.contact.clone();
                    let prune = contact::Prune::Full.value();
                    send_contact_info_req(
                        self.contact_info_tx.clone(),
                        ctx.clone(),
                        contact,
                        prune,
                    );
                    add_contact_timeout(self.contact_timeout_tx.clone(), ctx.clone());
                    self.is_adding = true;
                }
            }

            // Contact filter
            //-----------------------------------------------------------------------------------
            ui.heading("\nFind Contact");
            ui.label(
                "____________________________________________________________________________\n",
            );
            ui.horizontal(|ui| {
                let find_contact_label = ui.label("filter contacts: ");
                ui.text_edit_singleline(&mut self.find_contact)
                    .labelled_by(find_contact_label.id);
            });
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
                        ui.strong("Nickname");
                    });
                    header.col(|ui| {
                        ui.strong(".b32.i2p");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                })
                .body(|mut body| {
                    for c in &self.contacts {
                        if c.i2p_address.contains(&self.find_contact) {
                            let row_height = 20.0;
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.label("anon");
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", c.i2p_address));
                                });
                                row.col(|ui| {
                                    if ui.button("Check Status").clicked() {
                                        let nick_db = utils::search_gui_db(
                                            String::from(crate::GUI_NICK_DB_KEY),
                                            String::from(&c.i2p_address),
                                        );
                                        let nick = if nick_db == utils::empty_string() {
                                            String::from("anon")
                                        } else {
                                            nick_db
                                        };
                                        self.status.nick = nick;
                                        self.status.i2p = String::from(&c.i2p_address);
                                        // get the txp
                                        self.status.txp = utils::search_gui_db(
                                            String::from(crate::GUI_TX_PROOF_DB_KEY),
                                            String::from(&c.i2p_address),
                                        );
                                        // get the jwp
                                        self.status.jwp = utils::search_gui_db(
                                            String::from(crate::GUI_JWP_DB_KEY),
                                            String::from(&c.i2p_address),
                                        );
                                        let r_exp = utils::search_gui_db(
                                            String::from(crate::GUI_EXP_DB_KEY),
                                            String::from(&c.i2p_address),
                                        );
                                        self.status.exp = r_exp;
                                        let expire = match self.status.exp.parse::<i64>() {
                                            Ok(n) => n,
                                            Err(_e) => 0,
                                        };
                                        self.status.h_exp =
                                            chrono::NaiveDateTime::from_timestamp_opt(expire, 0)
                                                .unwrap()
                                                .to_string();
                                        // MESSAGES WON'T BE SENT UNTIL KEY IS SIGNED AND TRUSTED!
                                        self.status.signed_key =
                                            check_signed_key(self.status.i2p.clone());
                                        let prune = contact::Prune::Pruned.value();
                                        send_contact_info_req(
                                            self.contact_info_tx.clone(),
                                            ctx.clone(),
                                            self.status.i2p.clone(),
                                            prune,
                                        );
                                        self.showing_status = true;
                                        self.is_pinging = true;
                                    }
                                });
                                row.col(|ui| {
                                    let now = chrono::offset::Utc::now().timestamp();
                                    let expire = match self.status.exp.parse::<i64>() {
                                        Ok(n) => n,
                                        Err(_e) => 0,
                                    };
                                    if now < expire
                                        && self.status.signed_key
                                        && self.status.jwp != utils::empty_string()
                                        && c.i2p_address == self.status.i2p
                                    {
                                        if ui.button("Compose").clicked() {
                                            self.is_composing = true;
                                        }
                                    }
                                });
                            });
                        }
                    }
                });
        });
    }
}

// Send asyc requests to neveko-core
//------------------------------------------------------------------------------
fn send_contact_info_req(
    tx: Sender<models::Contact>,
    ctx: egui::Context,
    contact: String,
    prune: u32,
) {
    log::debug!("async send_contact_info_req");
    tokio::spawn(async move {
        match contact::add_contact_request(contact, prune).await {
            Ok(contact) => {
                let _ = tx.send(contact);
                ctx.request_repaint();
            }
            _ => log::debug!("failed to request invoice"),
        }
    });
}

fn send_create_contact_req(tx: Sender<models::Contact>, ctx: egui::Context, c: models::Contact) {
    log::debug!("async send_create_contact_req");
    tokio::spawn(async move {
        let j_contact = utils::contact_to_json(&c);
        let a_contact: models::Contact = contact::create(&j_contact).await;
        let _ = tx.send(a_contact);
        ctx.request_repaint();
    });
}

fn add_contact_timeout(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(ADD_CONTACT_TIMEOUT_SECS)).await;
        log::error!("add contact timeout");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn send_invoice_req(tx: Sender<reqres::Invoice>, ctx: egui::Context, contact: String) {
    log::debug!("async send_invoice_req");
    tokio::spawn(async move {
        match contact::request_invoice(contact).await {
            Ok(contact) => {
                let _ = tx.send(contact);
                ctx.request_repaint();
            }
            _ => log::debug!("failed to request invoice"),
        }
    });
}

fn send_payment_req(
    tx: Sender<bool>,
    ctx: egui::Context,
    d: reqres::Destination,
    contact: String,
    expire: u64,
    retry: bool,
) {
    log::debug!("async send_payment_req");
    log::debug!("cleaning stale jwp values");
    tokio::spawn(async move {
        if !retry {
            utils::clear_gui_db(String::from("gui-txp"), String::from(&contact));
            utils::clear_gui_db(String::from("gui-jwp"), String::from(&contact));
            utils::clear_gui_db(String::from("gui-exp"), String::from(&contact));
            let ptxp_address = String::from(&d.address);
            let ftxp_address = String::from(&d.address);
            log::debug!("sending {} piconero(s) to: {}", &d.amount, &d.address);
            let wallet_name = String::from(neveko_core::APP_NAME);
            let wallet_password = std::env::var(neveko_core::MONERO_WALLET_PASSWORD)
                .unwrap_or(String::from("password"));
            monero::open_wallet(&wallet_name, &wallet_password).await;
            let transfer: reqres::XmrRpcTransferResponse = monero::transfer(d).await;
            // in order to keep the jwp creation process transparent to the user
            // we will process all logic in one shot here.

            // use the hash to create a PENDING transaction proof
            let ptxp_hash = String::from(&transfer.result.tx_hash);
            let ftxp_hash = String::from(&transfer.result.tx_hash);
            let ptxp: proof::TxProof = proof::TxProof {
                subaddress: ptxp_address,
                confirmations: 0,
                hash: ptxp_hash,
                message: utils::empty_string(),
                signature: utils::empty_string(),
            };
            log::debug!("creating transaction proof for: {}", &ptxp.hash);
            // if we made it this far we can now request a JWP from our friend
            // wait a bit for the tx to propogate, i2p takes longer
            let wait = if std::env::var(neveko_core::GUI_REMOTE_NODE)
                .unwrap_or(utils::empty_string())
                == String::from(neveko_core::GUI_SET_REMOTE_NODE)
            {
                crate::I2P_PROPAGATION_TIME_IN_SECS_EST
            } else {
                crate::PROPAGATION_TIME_IN_SECS_EST
            };
            tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
            let get_txp: reqres::XmrRpcGetTxProofResponse = monero::get_tx_proof(ptxp).await;
            // TODO(c2m): error handling on failed tx proof generation
            // use the signature to create the FINALIZED transaction proof
            let ftxp: proof::TxProof = proof::TxProof {
                subaddress: ftxp_address,
                confirmations: 0,
                hash: ftxp_hash,
                message: utils::empty_string(),
                signature: get_txp.result.signature,
            };
            utils::write_gui_db(
                String::from(crate::GUI_TX_PROOF_DB_KEY),
                String::from(&contact),
                String::from(&ftxp.signature),
            );
            utils::write_gui_db(
                String::from(crate::GUI_TX_HASH_DB_KEY),
                String::from(&contact),
                String::from(&ftxp.hash),
            );
            utils::write_gui_db(
                String::from(crate::GUI_TX_SIGNATURE_DB_KEY),
                String::from(&contact),
                String::from(&ftxp.signature),
            );
            utils::write_gui_db(
                String::from(crate::GUI_TX_SUBADDRESS_DB_KEY),
                String::from(&contact),
                String::from(&ftxp.subaddress),
            );
            log::debug!(
                "proving payment to {} for: {}",
                String::from(&contact),
                &ftxp.hash
            );
            match proof::prove_payment(String::from(&contact), &ftxp).await {
                Ok(result) => {
                    utils::write_gui_db(
                        String::from(crate::GUI_JWP_DB_KEY),
                        String::from(&contact),
                        String::from(&result.jwp),
                    );
                    // this is just an estimate expiration but should suffice
                    let seconds: i64 = expire as i64 * 2 * 60;
                    let unix: i64 = chrono::offset::Utc::now().timestamp() + seconds;
                    utils::write_gui_db(
                        String::from(crate::GUI_EXP_DB_KEY),
                        String::from(&contact),
                        format!("{}", unix),
                    );
                    ctx.request_repaint();
                }
                _ => log::error!("failed to obtain jwp"),
            }
            monero::close_wallet(&wallet_name, &wallet_password).await;
        }
        if retry {
            let k_hash = String::from(crate::GUI_TX_HASH_DB_KEY);
            let k_sig = String::from(crate::GUI_TX_SIGNATURE_DB_KEY);
            let k_subaddress = String::from(crate::GUI_TX_SUBADDRESS_DB_KEY);
            let hash = utils::search_gui_db(k_hash, String::from(&contact));
            let signature = utils::search_gui_db(k_sig, String::from(&contact));
            let subaddress = utils::search_gui_db(k_subaddress, String::from(&contact));
            let ftxp: proof::TxProof = proof::TxProof {
                subaddress,
                confirmations: 0,
                hash: String::from(&hash),
                message: utils::empty_string(),
                signature,
            };
            log::debug!(
                "proving payment to {} for: {}",
                String::from(&contact),
                &ftxp.hash
            );
            match proof::prove_payment(String::from(&contact), &ftxp).await {
                Ok(result) => {
                    utils::write_gui_db(
                        String::from(crate::GUI_JWP_DB_KEY),
                        String::from(&contact),
                        String::from(&result.jwp),
                    );
                    ctx.request_repaint();
                }
                _ => log::error!("failed to obtain jwp"),
            }
        }
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn send_message_req(tx: Sender<bool>, ctx: egui::Context, body: String, to: String, jwp: String) {
    log::debug!("constructing message");
    let m: models::Message = models::Message {
        body: body.into_bytes(),
        to,
        mid: utils::empty_string(),
        uid: utils::empty_string(),
        created: 0,
        from: i2p::get_destination(None),
    };
    let j_message = utils::message_to_json(&m);
    tokio::spawn(async move {
        let m_type = message::MessageType::Normal;
        let result = message::create(j_message, jwp, m_type).await;
        if result.mid != utils::empty_string() {
            log::info!("sent message: {}", result.mid);
            let _ = tx.send(true);
            ctx.request_repaint();
        }
    });
}

fn check_signed_key(contact: String) -> bool {
    let v = utils::search_gui_db(String::from(crate::GUI_SIGNED_GPG_DB_KEY), contact);
    v != utils::empty_string()
}

fn change_nick_req(contact: String, nick: String) {
    log::debug!("change nick");
    utils::clear_gui_db(String::from(crate::GUI_NICK_DB_KEY), String::from(&contact));
    utils::write_gui_db(
        String::from(crate::GUI_NICK_DB_KEY),
        String::from(&contact),
        nick,
    );
}

fn send_can_transfer_req(tx: Sender<bool>, ctx: egui::Context, invoice: u128) {
    log::debug!("async send_can_transfer_req");
    tokio::spawn(async move {
        let can_transfer = utils::can_transfer(invoice).await;
        log::debug!("can transfer: {}", can_transfer);
        let _ = tx.send(can_transfer);
        ctx.request_repaint();
    });
}
