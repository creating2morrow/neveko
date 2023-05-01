use nevmes_core::*;
use std::sync::mpsc::{Receiver, Sender};

use crate::{
    ADD_CONTACT_TIMEOUT_SECS,
    BLOCK_TIME_IN_SECS_EST_U64,
    BLOCK_TIME_IN_SECS_EST_I64
};

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
            to: utils::empty_string()
        }
    }
}
/// Struct for the contact status window
struct Status {
    /// UNIX timestamp of expiration as string
    exp: String,
    /// human readable date of expiration as string
    h_exp: String,
    /// i2p address of current status check
    i2p: String,
    /// JSON Web Proof of current status check
    jwp: String,
    /// Alias for contact
    nick: String,
    signed_key: bool, 
    /// transaction proof signature of current status check
    txp: String,
}

impl Default for Status {
    fn default() -> Self {
        Status {
            exp: utils::empty_string(),
            h_exp: utils::empty_string(),
            i2p: utils::empty_string(),
            jwp: utils::empty_string(),
            nick: String::from("anon"),
            signed_key: false,
            txp: utils::empty_string(),
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
    is_pinging: bool,
    is_loading: bool,
    is_message_sent: bool,
    is_payment_processed: bool,
    is_timeout: bool,
    payment_tx: Sender<bool>,
    payment_rx: Receiver<bool>,
    showing_status: bool,
    status: Status,
    send_message_tx: Sender<bool>,
    send_message_rx: Receiver<bool>,
    s_contact: models::Contact,
    s_invoice: reqres::Invoice,
    s_added_contact: models::Contact,
}

impl Default for AddressBookApp {
    fn default() -> Self {
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
        
        // initial contact load
        if !self.contacts_init {
            self.contacts = contact::find_all();
            self.contacts_init = true;
        }

        // Compose window
        //-----------------------------------------------------------------------------------
        let mut is_composing = self.is_composing;
        egui::Window::new("Compose Message")
            .open(&mut is_composing)
            .vscroll(true)
            .show(&ctx, |ui| {
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("sending nevmes...");
                }
                ui.horizontal(|ui| {
                    ui.label(format!("to: {}", self.status.i2p))
                });
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
                            send_message_req(self.send_message_tx.clone(), ctx.clone(),
                                self.compose.message.clone(), self.compose.to.clone(), self.status.jwp.clone());
                        }
                    }
                    if ui.button("Exit").clicked() {
                        self.is_composing = false;
                    }
                }
            });

        // Payment approval window
        //-----------------------------------------------------------------------------------
        let mut is_approving_payment = self.approve_payment && self.s_invoice.address != utils::empty_string();
        let address = self.s_invoice.address.clone();
        let amount = self.s_invoice.pay_threshold;
        let expire = self.s_invoice.conf_threshold;
        egui::Window::new("Approve Payment for JWP")
            .open(&mut is_approving_payment)
            .vscroll(true)
            .show(&ctx, |ui| {
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("creating jwp may take a few minutes...");
                }
                ui.heading(self.status.i2p.clone());
                ui.label(format!("pay to: {}", address));
                ui.label(format!("amount: {} piconero(s)", amount));
                ui.label(format!("expiration: {} blocks", expire));
                if !self.is_loading {
                    if self.s_invoice.address != utils::empty_string() {
                        if ui.button("Approve").clicked() {
                            // activate xmr "transfer", check the hash, update db and refresh
                            let d: reqres::Destination = reqres::Destination { address, amount };
                            send_payment_req(self.payment_tx.clone(), ctx.clone(), d, self.status.i2p.clone(), expire);
                            self.is_loading = true;
                        }
                    }
                    if ui.button("Exit").clicked() {
                        self.approve_payment = false;
                        self.is_loading = false;
                    }
                }
            });

        // Contact status window
        //-----------------------------------------------------------------------------------
        let mut is_showing_status = self.showing_status;
        egui::Window::new(&self.status.i2p)
            .open(&mut is_showing_status)
            .vscroll(true)
            .title_bar(false)
            .id(egui::Id::new(self.status.i2p.clone()))
            .show(&ctx, |ui| {
                if self.is_pinging {
                    ui.add(egui::Spinner::new());
                    ui.label("pinging...");
                }
                let status = if self.s_contact.xmr_address != utils::empty_string() { "online" } else { "offline" };
                ui.label(format!("status: {}", status));
                ui.label(format!("nick: {}", self.status.nick));
                ui.label(format!("tx proof: {}", self.status.txp));
                ui.label(format!("jwp: {}", self.status.jwp));
                ui.label(format!("expiration: {}", self.status.h_exp));
                ui.label(format!("signed key: {}", self.status.signed_key));
                if self.status.jwp == utils::empty_string()
                    && !self.is_pinging && status == "online"
                    && self.status.txp == utils::empty_string() {
                    if ui.button("Create JWP").clicked() {
                        send_invoice_req(self.invoice_tx.clone(), ctx.clone(), self.status.i2p.clone());
                        self.approve_payment = true;
                        self.showing_status = false;
                    }
                }
                if !self.status.signed_key {
                    if ui.button("Sign Key").clicked() {
                        contact::trust_gpg(self.status.i2p.clone());
                        write_gui_db(String::from("gui-signed-key"), self.status.i2p.clone(), String::from("1"));
                        self.showing_status = false;
                    }
                }
                let failed_to_prove = self.status.txp != utils::empty_string()
                    && self.status.jwp == utils::empty_string();
                if self.status.jwp != utils::empty_string() || failed_to_prove {
                    if ui.button("Clear stale JWP").clicked() {
                        clear_gui_db(String::from("gui-txp"), self.status.i2p.clone());
                        clear_gui_db(String::from("gui-jwp"), self.status.i2p.clone());
                        clear_gui_db(String::from("gui-exp"), self.status.i2p.clone());
                        self.showing_status = false;
                    }
                }
                ui.horizontal(|ui| {
                    let nick_label = ui.label("nick: ");
                    ui.text_edit_singleline(&mut self.add_nick).labelled_by(nick_label.id);
                });
                if ui.button("Change nick").clicked() {
                    change_nick_req(self.status.i2p.clone(), self.add_nick.clone());
                    self.add_nick = utils::empty_string();
                }
                if ui.button("Exit").clicked() {
                    self.showing_status = false;
                }
            });

        // Main panel for adding contacts
        //-----------------------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Add Contact");
            ui.label(
                "____________________________________________________________________________\n",
            );
            ui.horizontal(|ui| {
                let contact_label = ui.label("contact: ");
                ui.text_edit_singleline(&mut self.contact).labelled_by(contact_label.id);
            });
            let mut is_approved = self.approve_contact;
            let mut is_added = self.added;
            let is_loading = self.is_loading;
            let i2p_address = self.s_contact.i2p_address.clone();
            let xmr_address = self.s_contact.xmr_address.clone();
            let gpg_key = self.s_contact.gpg_key.iter().cloned().collect();
            
            // Contact added confirmation screen
            //-----------------------------------------------------------------------------------
            egui::Window::new("Added contact")
                .open(&mut is_added)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label(format!("i2p address: {}", self.s_added_contact.i2p_address));
                    if ui.button("Exit").clicked() {
                        self.added = false;
                        self.contact = utils::empty_string();
                        self.is_adding = false;
                        self.approve_contact = false;
                        self.contacts = contact::find_all();
                        for c in &self.contacts { ui.label(format!("{}", c.i2p_address)); }
                    }
                });
            
            // Contact approval screen
            //-----------------------------------------------------------------------------------
            egui::Window::new("Approve Contact")
                .open(&mut is_approved)
                .vscroll(true)
                .show(ctx, |ui| {
                    if is_loading {
                        ui.add(egui::Spinner::new());
                        ui.label("adding contact...");
                    }
                    ui.label(format!("i2p: {}", i2p_address));
                    ui.label(format!("xmr: {}", xmr_address));
                    ui.label(format!("gpg: {}", String::from_utf8(gpg_key).unwrap_or(utils::empty_string())));
                    ui.horizontal(|ui| {
                        if !is_loading {
                            if ui.button("Approve").clicked() {
                                self.is_loading = true;
                                self.approve_contact = false;
                                let c_contact: models::Contact = models::Contact {
                                    cid: self.s_contact.cid.clone(),
                                    i2p_address, xmr_address, gpg_key: self.s_contact.gpg_key.iter().cloned().collect()
                                };
                                send_create_contact_req(self.contact_add_tx.clone(), ctx.clone(), c_contact);  
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
                    send_contact_info_req(self.contact_info_tx.clone(), ctx.clone(), contact);
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
                ui.text_edit_singleline(&mut self.find_contact).labelled_by(find_contact_label.id);
            });
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
                    .body(|mut body|
                        for c in &self.contacts {
                            if c.i2p_address.contains(&self.find_contact) {
                                let row_height =  20.0;
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.label("anon");
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", c.i2p_address));
                                    });
                                    row.col(|ui| {
                                        if ui.button("Check Status").clicked() {
                                            let nick_db = search_gui_db(String::from("gui-nick"), String::from(&c.i2p_address));
                                            let nick = if nick_db == utils::empty_string() { String::from("anon") } else { nick_db };
                                            self.status.nick = nick;
                                            self.status.i2p = String::from(&c.i2p_address);
                                            // get the txp
                                            self.status.txp = search_gui_db(String::from("gui-txp"), String::from(&c.i2p_address));
                                            // get the jwp
                                            self.status.jwp = search_gui_db(String::from("gui-jwp"), String::from(&c.i2p_address));
                                            let r_exp = search_gui_db(String::from("gui-exp"), String::from(&c.i2p_address));
                                            self.status.exp = r_exp;
                                            let expire = match self.status.exp.parse::<i64>() {
                                                Ok(n) => n,
                                                Err(_e) => 0,
                                            };
                                            self.status.h_exp = chrono::NaiveDateTime::from_timestamp_opt(expire, 0)
                                                .unwrap().to_string();
                                            // MESSAGES WON'T BE SENT UNTIL KEY IS SIGNED AND TRUSTED!
                                            self.status.signed_key = check_signed_key(self.status.i2p.clone());
                                            send_contact_info_req(self.contact_info_tx.clone(), ctx.clone(), self.status.i2p.clone());
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
                                        if now < expire && self.status.signed_key
                                            && self.status.jwp != utils::empty_string() 
                                            && c.i2p_address == self.status.i2p {
                                                if ui.button("Compose").clicked() {
                                                    self.is_composing = true;
                                                }
                                        }
                                    });
                                });
                            }
                    });
        });
    }
}

// Send asyc requests to nevmes-core
//------------------------------------------------------------------------------
fn send_contact_info_req
(tx: Sender<models::Contact>, ctx: egui::Context, contact: String) {
    log::debug!("async send_contact_info_req");
    tokio::spawn(async move {
        match contact::add_contact_request(contact).await {
            Ok(contact) => {
                let _ = tx.send(contact);
                ctx.request_repaint();
            }
            _ => log::debug!("failed to request invoice"),
        }
    });
}

fn send_create_contact_req
(tx: Sender<models::Contact>, ctx: egui::Context, c: models::Contact) {
    log::debug!("async send_create_contact_req");
    tokio::spawn(async move {
        let j_contact = utils::contact_to_json(&c);
        let a_contact: models::Contact = contact::create(&j_contact).await;
        let _ = tx.send(a_contact);
        ctx.request_repaint();
    });
}

fn add_contact_timeout
(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(ADD_CONTACT_TIMEOUT_SECS)).await;
        log::error!("add contact timeout");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn search_gui_db
(f: String, data: String) -> String {
    let s = db::Interface::open();
    let k = format!("{}-{}", f, data);
    db::Interface::read(&s.env, &s.handle, &k)
}

fn write_gui_db
(f: String, key: String, data: String) {
    let s = db::Interface::open();
    let k = format!("{}-{}", f, key);
    db::Interface::write(&s.env, &s.handle, &k, &data);
}

fn clear_gui_db
(f: String, key: String) {
    let s = db::Interface::open();
    let k = format!("{}-{}", f, key);
    db::Interface::delete(&s.env, &s.handle, &k);
}

fn send_invoice_req
(tx: Sender<reqres::Invoice>, ctx: egui::Context, contact: String) {
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

fn send_payment_req
(tx: Sender<bool>, ctx: egui::Context, d: reqres::Destination, contact: String, expire: u64) {
    log::debug!("async send_payment_req");
    log::debug!("cleaning stale jwp values");
    clear_gui_db(String::from("gui-txp"), String::from(&contact));
    clear_gui_db(String::from("gui-jwp"), String::from(&contact));
    clear_gui_db(String::from("gui-exp"), String::from(&contact));
    let mut retry_count = 1;
    tokio::spawn(async move {
        let ptxp_address = String::from(&d.address);
        let ftxp_address = String::from(&d.address);
        log::debug!("sending {} piconero(s) to: {}", &d.amount, &d.address);
        let transfer: reqres::XmrRpcTransferResponse = monero::transfer(d).await;
        // in order to keep the jwp creation process transparent to the user
        // we will process all logic in one shot here.

        // use the hash to create a PENDING transaction proof
        let ptxp_hash = String::from(&transfer.result.tx_hash);
        let ftxp_hash = String::from(&transfer.result.tx_hash);
        let ptxp: proof::TxProof = proof::TxProof {
            address: ptxp_address,
            confirmations: 0,
            hash: ptxp_hash,
            message: utils::empty_string(),
            signature: utils::empty_string(),
        };
        log::debug!("creating transaction proof for: {}", &ptxp.hash);
        let get_txp: reqres::XmrRpcGetTxProofResponse = monero::get_tx_proof(ptxp).await;
        // use the signature to create the FINALIZED transaction proof
        let ftxp: proof::TxProof = proof::TxProof {
            address: ftxp_address,
            confirmations: 0,
            hash:ftxp_hash ,
            message: utils::empty_string(),
            signature: get_txp.result.signature,
        };
        // we will poll for 6 minutes MAX because jwp cannot be created without at least ONE conf
        loop {
            if retry_count > 3 {
                break;
            }
            let check_txp: reqres::XmrRpcCheckTxProofResponse = monero::check_tx_proof(&ftxp).await;
            if check_txp.result.good && check_txp.result.confirmations > 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(BLOCK_TIME_IN_SECS_EST_U64)).await;
            retry_count += 1;
        }
        write_gui_db(String::from("gui-txp"), String::from(&contact), String::from(&ftxp.signature));
        log::debug!("proving payment to {} for: {}", String::from(&contact), &ftxp.hash);
        // if we made it this far we can now request a JWP from our friend
        match proof::prove_payment(String::from(&contact), &ftxp).await {
            Ok(result) => {
                write_gui_db(String::from("gui-jwp"), String::from(&contact), String::from(&result.jwp));
                // this is just an estimate expiration but should suffice
                let seconds: i64 = expire as i64*2*60;
                // subtract 120 seconds since we had to wait for one confirmation
                let grace: i64 = seconds-BLOCK_TIME_IN_SECS_EST_I64;
                let unix: i64 = chrono::offset::Utc::now().timestamp()+grace;
                write_gui_db(String::from("gui-exp"), String::from(&contact), format!("{}", unix));
                // TODO(c2m): edge case when proving payment fails to complete
                //            case the payment proof data and set retry logic
                ctx.request_repaint();
            }
            _ => log::error!("failed to obtain jwp"),
        }
        let _= tx.send(true);
        ctx.request_repaint();
    });
}

fn send_message_req
(tx: Sender<bool>, ctx: egui::Context, body: String, to: String, jwp: String) {
    log::debug!("constructing message");
    let m: models::Message = models::Message {
        body: body.into_bytes(),
        to,
        mid: utils::empty_string(),
        uid: utils::empty_string(),
        created: 0,
        from: i2p::get_destination(),
    };
    let j_message = utils::message_to_json(&m);
    tokio::spawn(async move {
        let result = message::create(j_message, jwp).await;
        if result.mid != utils::empty_string() {
            log::info!("sent message: {}", result.mid);
            let _= tx.send(true);
            ctx.request_repaint();
        }
    });
}

fn check_signed_key(contact: String) -> bool {
    let v = search_gui_db(String::from("gui-signed-key"), contact);
    v != utils::empty_string()
}

fn change_nick_req(contact: String, nick: String) {
    log::debug!("change nick");
    clear_gui_db(String::from("gui-nick"), String::from(&contact));
    write_gui_db(String::from("gui-nick"), String::from(&contact), nick);
}
