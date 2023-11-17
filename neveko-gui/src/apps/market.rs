use egui::RichText;
use image::Luma;
use neveko_core::*;
use qrcode::QrCode;
use std::sync::mpsc::{
    Receiver,
    Sender,
};

pub struct MultisigManagement {
    pub completed_kex_init: bool,
    pub completed_kex_final: bool,
    pub completed_export: bool,
    pub completed_funding: bool,
    pub completed_prepare: bool,
    pub completed_make: bool,
    pub exchange_multisig_keys: String,
    pub export_info: String,
    pub has_mediator: bool,
    pub make_info: String,
    pub mediator: String,
    pub prepare_info: String,
    pub query_mediator: bool,
    pub signed_txset: String,
    pub vendor: String,
}

impl Default for MultisigManagement {
    fn default() -> Self {
        MultisigManagement {
            completed_kex_init: false,
            completed_kex_final: false,
            completed_export: false,
            completed_funding: false,
            completed_prepare: false,
            completed_make: false,
            exchange_multisig_keys: utils::empty_string(),
            export_info: utils::empty_string(),
            has_mediator: false,
            make_info: utils::empty_string(),
            mediator: utils::empty_string(),
            prepare_info: utils::empty_string(),
            query_mediator: false,
            signed_txset: utils::empty_string(),
            vendor: utils::empty_string(),
        }
    }
}

pub struct MarketApp {
    contact_info_tx: Sender<models::Contact>,
    contact_info_rx: Receiver<models::Contact>,
    contact_timeout_tx: Sender<bool>,
    contact_timeout_rx: Receiver<bool>,
    customer_orders: Vec<models::Order>,
    find_vendor: String,
    get_vendor_products_tx: Sender<Vec<models::Product>>,
    get_vendor_products_rx: Receiver<Vec<models::Product>>,
    get_vendor_product_tx: Sender<models::Product>,
    get_vendor_product_rx: Receiver<models::Product>,
    is_loading: bool,
    is_ordering: bool,
    order_funded_tx: Sender<bool>,
    order_funded_rx: Receiver<bool>,
    is_order_qr_set: bool,
    is_pinging: bool,
    is_customer_viewing_orders: bool,
    is_managing_multisig: bool,
    is_product_image_set: bool,
    is_showing_products: bool,
    is_showing_product_image: bool,
    is_showing_product_update: bool,
    is_showing_orders: bool,
    is_showing_order_qr: bool,
    is_showing_vendor_status: bool,
    is_showing_vendors: bool,
    is_timeout: bool,
    is_vendor_enabled: bool,
    is_window_shopping: bool,
    msig: MultisigManagement,
    /// order currently being acted on
    m_order: models::Order,
    orders: Vec<models::Order>,
    order_xmr_address: String,
    order_qr: egui_extras::RetainedImage,
    order_qr_init: bool,
    our_make_info_tx: Sender<String>,
    our_make_info_rx: Receiver<String>,
    our_prepare_info_tx: Sender<String>,
    our_prepare_info_rx: Receiver<String>,
    product_from_vendor: models::Product,
    product_image: egui_extras::RetainedImage,
    products: Vec<models::Product>,
    product_update_pid: String,
    new_product_image: String,
    new_product_name: String,
    new_product_desc: String,
    new_product_price: String,
    new_product_qty: String,
    new_order: models::Order,
    new_order_price: u128,
    new_order_quantity: String,
    new_order_shipping_address: String,
    _refresh_on_delete_product_tx: Sender<bool>,
    _refresh_on_delete_product_rx: Receiver<bool>,
    submit_order_tx: Sender<models::Order>,
    submit_order_rx: Receiver<models::Order>,
    s_contact: models::Contact,
    s_order: models::Order,
    vendor_status: utils::ContactStatus,
    vendors: Vec<models::Contact>,
    order_xmr_address_tx: Sender<reqres::XmrRpcAddressResponse>,
    order_xmr_address_rx: Receiver<reqres::XmrRpcAddressResponse>,
}

impl Default for MarketApp {
    fn default() -> Self {
        let (contact_timeout_tx, contact_timeout_rx) = std::sync::mpsc::channel();
        let (_refresh_on_delete_product_tx, _refresh_on_delete_product_rx) =
            std::sync::mpsc::channel();
        let read_product_image = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
        let s = db::Interface::open();
        let r = db::Interface::read(&s.env, &s.handle, contact::NEVEKO_VENDOR_ENABLED);
        let is_vendor_enabled = r == contact::NEVEKO_VENDOR_MODE_ON;
        let (contact_info_tx, contact_info_rx) = std::sync::mpsc::channel();
        let (get_vendor_products_tx, get_vendor_products_rx) = std::sync::mpsc::channel();
        let (get_vendor_product_tx, get_vendor_product_rx) = std::sync::mpsc::channel();
        let (submit_order_tx, submit_order_rx) = std::sync::mpsc::channel();
        let (our_prepare_info_tx, our_prepare_info_rx) = std::sync::mpsc::channel();
        let (our_make_info_tx, our_make_info_rx) = std::sync::mpsc::channel();
        let (order_xmr_address_tx, order_xmr_address_rx) = std::sync::mpsc::channel();
        let (order_funded_tx, order_funded_rx) = std::sync::mpsc::channel();
        let contents = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
        MarketApp {
            contact_info_rx,
            contact_info_tx,
            contact_timeout_rx,
            contact_timeout_tx,
            customer_orders: Vec::new(),
            find_vendor: utils::empty_string(),
            get_vendor_products_rx,
            get_vendor_products_tx,
            get_vendor_product_rx,
            get_vendor_product_tx,
            is_customer_viewing_orders: false,
            is_loading: false,
            is_managing_multisig: false,
            is_ordering: false,
            order_funded_rx,
            order_funded_tx,
            is_order_qr_set: false,
            is_pinging: false,
            is_product_image_set: false,
            is_showing_orders: false,
            is_showing_order_qr: false,
            is_showing_products: false,
            is_showing_product_image: false,
            is_showing_product_update: false,
            is_showing_vendor_status: false,
            is_showing_vendors: false,
            is_timeout: false,
            is_vendor_enabled,
            is_window_shopping: false,
            msig: Default::default(),
            m_order: Default::default(),
            order_xmr_address: utils::empty_string(),
            order_xmr_address_rx,
            order_xmr_address_tx,
            order_qr: egui_extras::RetainedImage::from_image_bytes("qr.png", &contents).unwrap(),
            order_qr_init: false,
            our_make_info_rx,
            our_make_info_tx,
            our_prepare_info_rx,
            our_prepare_info_tx,
            new_order: Default::default(),
            new_order_price: 0,
            new_order_shipping_address: utils::empty_string(),
            new_order_quantity: utils::empty_string(),
            orders: Vec::new(),
            product_from_vendor: Default::default(),
            product_image: egui_extras::RetainedImage::from_image_bytes(
                "qr.png",
                &read_product_image,
            )
            .unwrap(),
            products: Vec::new(),
            product_update_pid: utils::empty_string(),
            new_product_image: utils::empty_string(),
            new_product_name: utils::empty_string(),
            new_product_desc: utils::empty_string(),
            new_product_price: utils::empty_string(),
            new_product_qty: utils::empty_string(),
            _refresh_on_delete_product_tx,
            _refresh_on_delete_product_rx,
            s_contact: Default::default(),
            s_order: Default::default(),
            submit_order_rx,
            submit_order_tx,
            vendor_status: Default::default(),
            vendors: Vec::new(),
        }
    }
}

impl eframe::App for MarketApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hook into async channel threads
        //-----------------------------------------------------------------------------------

        if let Ok(submit_order) = self.submit_order_rx.try_recv() {
            self.s_order = submit_order;
            if self.s_order.orid != utils::empty_string() {
                self.is_ordering = false;
                self.is_loading = false;
            }
        }

        if let Ok(contact_info) = self.contact_info_rx.try_recv() {
            self.s_contact = contact_info;
            if self.s_contact.xmr_address != utils::empty_string() {
                self.is_pinging = false;
                self.vendor_status.is_vendor = self.s_contact.is_vendor;
            }
        }

        if let Ok(vendor_products) = self.get_vendor_products_rx.try_recv() {
            self.is_loading = false;
            self.products = vendor_products;
        }

        if let Ok(vendor_product) = self.get_vendor_product_rx.try_recv() {
            self.is_loading = false;
            if !vendor_product.image.is_empty() {
                // only pull image from vendor when we want to view
                let file_path = format!(
                    "/home/{}/.neveko/{}.jpeg",
                    std::env::var("USER").unwrap_or(String::from("user")),
                    vendor_product.pid
                );
                if self.is_window_shopping {
                    match std::fs::write(&file_path, &vendor_product.image) {
                        Ok(w) => w,
                        Err(_) => {
                            log::error!("failed to write product image")
                        }
                    };
                    self.is_loading = true;
                    let contents = std::fs::read(&file_path).unwrap_or(Vec::new());
                    // this image should uwrap if vendor image bytes are
                    // bad
                    let default_img = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
                    let default_r_img =
                        egui_extras::RetainedImage::from_image_bytes("qr.png", &default_img)
                            .unwrap();
                    self.product_image =
                        egui_extras::RetainedImage::from_image_bytes(file_path, &contents)
                            .unwrap_or(default_r_img);
                }
            }
            self.product_from_vendor = vendor_product;
            self.is_product_image_set = true;
            self.is_showing_product_image = true;
            self.is_loading = false;
        }

        if let Ok(timeout) = self.contact_timeout_rx.try_recv() {
            self.is_timeout = true;
            if timeout {
                self.is_pinging = false;
            }
        }

        if let Ok(our_prepare_info) = self.our_prepare_info_rx.try_recv() {
            self.msig.prepare_info = our_prepare_info;
            self.is_loading = false;
        }

        if let Ok(our_make_info) = self.our_make_info_rx.try_recv() {
            self.msig.make_info = our_make_info;
            self.is_loading = false;
        }

        if let Ok(a) = self.order_xmr_address_rx.try_recv() {
            self.order_xmr_address = a.result.address;
            if self.order_xmr_address != utils::empty_string() {
                self.is_showing_order_qr = true;
            }
        }

        if let Ok(funded) = self.order_funded_rx.try_recv() {
            self.msig.completed_funding = funded;
        }
        
        // Vendor status window
        //-----------------------------------------------------------------------------------
        let mut is_showing_vendor_status = self.is_showing_vendor_status;
        egui::Window::new("vendor status")
            .title_bar(false)
            .open(&mut is_showing_vendor_status)
            .vscroll(true)
            .id(egui::Id::new(self.vendor_status.i2p.clone()))
            .show(&ctx, |ui| {
                if self.is_pinging {
                    ui.add(egui::Spinner::new());
                    ui.label("pinging...");
                }
                let status = if self.s_contact.xmr_address != utils::empty_string() {
                    "online"
                } else {
                    "offline"
                };
                let mode = if self.vendor_status.is_vendor {
                    "enabled "
                } else {
                    "disabled"
                };
                ui.label(format!("status: {}", status));
                ui.label(format!("vendor mode: {}", mode));
                ui.label(format!("nick: {}", self.vendor_status.nick));
                ui.label(format!("tx proof: {}", self.vendor_status.txp));
                ui.label(format!("jwp: {}", self.vendor_status.jwp));
                ui.label(format!("expiration: {}", self.vendor_status.h_exp));
                ui.label(format!("signed key: {}", self.vendor_status.signed_key));
                if ui.button("Exit").clicked() {
                    self.is_showing_vendor_status = false;
                }
            });

        // Product image window
        //-----------------------------------------------------------------------------------
        let mut is_showing_product_image = self.is_showing_product_image;
        egui::Window::new("product image")
            .open(&mut is_showing_product_image)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                self.product_image.show(ui);
                if ui.button("Exit").clicked() {
                    self.is_showing_product_image = false;
                    self.is_product_image_set = false;
                    let read_product_image = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
                    self.product_image =
                        egui_extras::RetainedImage::from_image_bytes("qr.png", &read_product_image)
                            .unwrap();
                }
            });

        // Customer Multisig Management window
        //-----------------------------------------------------------------------------------
        let mut is_managing_multisig = self.is_managing_multisig;
        egui::Window::new("msig")
            .open(&mut is_managing_multisig)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading("Multisig Management");
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("msig request in progress...");
                }
                ui.horizontal(|ui| {
                    let mediator = ui.label("Mediator: ");
                    let prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                    if !self.msig.query_mediator {
                        let mediator_db =
                            utils::search_gui_db(String::from(&prefix), self.m_order.orid.clone());
                        log::debug!("mediator db: {}", mediator_db);
                        self.msig.has_mediator = mediator_db != utils::empty_string();
                        self.msig.mediator = mediator_db;
                        self.msig.query_mediator = true;
                    } else if self.msig.query_mediator && !self.msig.has_mediator {
                        ui.text_edit_singleline(&mut self.msig.mediator)
                            .labelled_by(mediator.id);
                        ui.label("\t");
                        if ui.button("Set Mediator").clicked() {
                            utils::write_gui_db(
                                prefix,
                                self.m_order.orid.clone(),
                                self.msig.mediator.clone(),
                            );
                            self.msig.has_mediator = true;
                        }
                    } else {
                        ui.label(self.msig.mediator.clone());
                        ui.label("\t");
                        if !self.msig.completed_prepare {
                            if ui.button("Clear Mediator").clicked() {
                                utils::clear_gui_db(prefix, self.m_order.orid.clone());
                                self.msig.mediator = utils::empty_string();
                                self.msig.has_mediator = false;
                                self.msig.query_mediator = false;
                            }
                        }
                    }
                });
                if !self.msig.completed_prepare {
                    ui.horizontal(|ui| {
                        ui.label("Prepare:  \t\t\t\t\t");
                        if ui.button("Prepare").clicked() {
                            self.is_loading = true;
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            // get prepare multisig info from vendor and mediator
                            // call prepare multisig and save to db
                            send_prepare_info_req(
                                self.our_prepare_info_tx.clone(),
                                ctx.clone(),
                                mediator,
                                &self.m_order.orid.clone(),
                                vendor,
                            )
                        }
                        if ui.button("Check").clicked() {
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            let sub_type = String::from(message::PREPARE_MSIG);
                            let is_prepared = validate_msig_step(
                                &mediator,
                                &self.m_order.orid,
                                &vendor,
                                &sub_type,
                            );
                            self.msig.completed_prepare = is_prepared;
                        }
                    });
                }
                if self.msig.completed_prepare && !self.msig.completed_make {
                    ui.horizontal(|ui| {
                        ui.label("Make:   \t\t\t\t\t\t");
                        if ui.button("Make").clicked() {
                            self.is_loading = true;
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            // get make multisig info from vendor and mediator
                            // call make multisig and save to db
                            send_make_info_req(
                                self.our_make_info_tx.clone(),
                                ctx.clone(),
                                mediator,
                                &self.m_order.orid.clone(),
                                vendor,
                            )
                        }
                        if ui.button("Check").clicked() {
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            let sub_type = String::from(message::MAKE_MSIG);
                            let is_made = validate_msig_step(
                                &mediator,
                                &self.m_order.orid,
                                &vendor,
                                &sub_type,
                            );
                            self.msig.completed_make = is_made;
                        }
                    });
                }
                if self.msig.completed_make && !self.msig.completed_kex_init {
                    ui.horizontal(|ui| {
                        ui.label("Key Exchange Initial:  \t\t\t");
                        if ui.button("KEX-INIT").clicked() {
                            self.is_loading = true;
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            // get kex round one info from vendor and mediator
                            // call make multisig and save to db
                            send_kex_initial_req(
                                self.our_make_info_tx.clone(),
                                ctx.clone(),
                                mediator,
                                &self.m_order.orid.clone(),
                                vendor,
                            )
                        }
                        if ui.button("Check").clicked() {
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            let sub_type = String::from(message::KEX_ONE_MSIG);
                            let is_made = validate_msig_step(
                                &mediator,
                                &self.m_order.orid,
                                &vendor,
                                &sub_type,
                            );
                            self.msig.completed_kex_init = is_made;
                        }
                    });
                }
                if self.msig.completed_kex_init && !self.msig.completed_kex_final {
                    ui.horizontal(|ui| {
                        ui.label("Key Exchange Final:  \t\t\t");
                        if ui.button("KEX-FINAL").clicked() {
                            self.is_loading = true;
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            // get kex round one info from vendor and mediator
                            // call make multisig and save to db
                            send_kex_final_req(
                                self.our_make_info_tx.clone(),
                                ctx.clone(),
                                mediator,
                                &self.m_order.orid.clone(),
                                vendor,
                            )
                        }
                        if ui.button("Check").clicked() {
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            let sub_type = String::from(message::KEX_TWO_MSIG);
                            let is_made = validate_msig_step(
                                &mediator,
                                &self.m_order.orid,
                                &vendor,
                                &sub_type,
                            );
                            self.msig.completed_kex_final = is_made;
                        }
                    });
                }
                if self.msig.completed_kex_final && !self.msig.completed_funding {
                    ui.horizontal(|ui| {
                        ui.label("Fund:\t\t\t\t\t\t\t");
                        if ui.button("Fund").clicked() {
                            set_order_address(
                                &self.m_order.orid,
                                self.order_xmr_address_tx.clone(),
                                ctx.clone(),
                            );
                        }
                        if ui.button("Check").clicked() {
                            // is the wallet multisig completed?
                            // the customer doesn't pay fees on orders
                            // ensure the balance of the order wallet matches the order total
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let contact =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            verify_order_wallet_funded(
                                &contact,
                                &self.m_order.orid,
                                self.order_funded_tx.clone(),
                                ctx.clone()
                            )
                        }
                    });
                }
                if self.msig.completed_funding && !self.msig.completed_export {
                    ui.horizontal(|ui| {
                        ui.label("Export Info: \t\t\t\t");
                        if ui.button("Export").clicked() {
                            self.is_loading = true;
                            let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                            let vendor_prefix = String::from(crate::GUI_OVL_DB_KEY);
                            let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                            let vendor =
                                utils::search_gui_db(vendor_prefix, self.m_order.orid.clone());
                            // not much orchestration here afaik, just send the output to the other participants
                            // TODO(c2m): 'idk remember why this tx.clone() is being reused' but not nothing breaks for now...
                            send_import_info_req(
                                self.our_make_info_tx.clone(),
                                ctx.clone(),
                                mediator,
                                &self.m_order.orid.clone(),
                                vendor,
                            )
                        }
                        if ui.button("Check").clicked() {}
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Import Info: \t");
                    if ui.button("Update").clicked() {}
                });
                ui.horizontal(|ui| {
                    ui.label("Release Payment: \t");
                    if ui.button("Sign Txset").clicked() {}
                });
                ui.horizontal(|ui| {
                    ui.label("Create Dispute: \t\t");
                    if ui.button("Dispute").clicked() {}
                });
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.is_managing_multisig = false;
                    self.is_loading = false;
                }
            });

        // Order Wallet QR
        //-----------------------------------------------------------------------------------
        let mut is_showing_order_qr = self.is_showing_order_qr;
        egui::Window::new("order wallet qr")
            .open(&mut is_showing_order_qr)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                if !self.is_order_qr_set && self.order_xmr_address != utils::empty_string() {
                    let code = QrCode::new(&self.order_xmr_address.clone()).unwrap();
                    let image = code.render::<Luma<u8>>().build();
                    let file_path = format!(
                        "/home/{}/.neveko/qr.png",
                        std::env::var("USER").unwrap_or(String::from("user"))
                    );
                    image.save(&file_path).unwrap();
                    self.order_qr_init = true;
                    self.is_order_qr_set = true;
                    let contents = std::fs::read(&file_path).unwrap_or(Vec::new());
                    self.order_qr =
                        egui_extras::RetainedImage::from_image_bytes("qr.png", &contents).unwrap();
                    ctx.request_repaint();
                }
                self.order_qr.show(ui);
                let address_label = ui.label("copy: \t");
                ui.text_edit_singleline(&mut self.order_xmr_address)
                    .labelled_by(address_label.id);
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.is_showing_order_qr = false;
                }
            });

        // View orders - Customer Order Flow Management
        //-----------------------------------------------------------------------------------
        let mut is_customer_viewing_orders = self.is_customer_viewing_orders;
        egui::Window::new("view orders")
            .open(&mut is_customer_viewing_orders)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("View Orders");
                use egui_extras::{
                    Column,
                    TableBuilder,
                };
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .min_scrolled_height(0.0);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("orid");
                        });
                        header.col(|ui| {
                            ui.strong("date");
                        });
                        header.col(|ui| {
                            ui.strong("status");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                    })
                    .body(|mut body| {
                        for o in &self.customer_orders {
                            let row_height = 20.0;
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{}", o.orid));
                                });
                                row.col(|ui| {
                                    let h_date =
                                        chrono::NaiveDateTime::from_timestamp_opt(o.date, 0)
                                            .unwrap()
                                            .to_string();
                                    ui.label(format!("{}", h_date));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", o.status));
                                });
                                row.col(|ui| {
                                    if ui.button("MSIG").clicked() {
                                        // dynamically generate buttons for multisig wallet ops
                                        self.is_managing_multisig = true;
                                        self.m_order.orid = String::from(&o.orid);
                                    }
                                });
                                row.col(|ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.horizontal(|ui| {
                                        if ui.button("Cancel").clicked() {
                                            // TODO(c2m): Cancel order logic
                                        }
                                    });
                                });
                            });
                        }
                    });
                if ui.button("Exit").clicked() {
                    self.is_customer_viewing_orders = false;
                    self.is_loading = false;
                }
            });

        // Customer Order Form
        //-----------------------------------------------------------------------------------
        let mut is_ordering = self.is_ordering;
        egui::Window::new("order form")
            .open(&mut is_ordering)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Order Form");
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("loading...");
                }
                let mediator_prefix = String::from(crate::GUI_MSIG_MEDIATOR_DB_KEY);
                let mediator =
                                utils::search_gui_db(mediator_prefix, self.m_order.orid.clone());
                ui.label(format!("customer id: {}", self.new_order.cid));
                ui.label(format!("mediator id: {}", mediator));
                ui.label(format!("product id: {}", self.new_order.pid));
                ui.horizontal(|ui| {
                    let shipping_name = ui.label("shipping address: ");
                    ui.text_edit_singleline(&mut self.new_order_shipping_address)
                        .labelled_by(shipping_name.id);
                });
                ui.horizontal(|ui| {
                    let qty_name = ui.label("quantity: \t\t\t\t");
                    ui.text_edit_singleline(&mut self.new_order_quantity)
                        .labelled_by(qty_name.id);
                });
                ui.label(format!("price: {}", self.new_order_price));
                let qty = match self.new_order_quantity.parse::<u128>() {
                    Ok(q) => q,
                    Err(_) => 0,
                };
                let mut p_qty: u128 = 0;
                for p in &self.products {
                    if p.pid == self.new_order.pid {
                        p_qty = p.qty;
                        break;
                    }
                }
                if qty <= p_qty && qty > 0 {
                    if ui.button("Submit Order").clicked() {
                        let address_bytes = self.new_order_shipping_address.clone().into_bytes();
                        let encrypted_shipping_address =
                            gpg::encrypt(self.vendor_status.i2p.clone(), &address_bytes);
                        let new_order = reqres::OrderRequest {
                            cid: String::from(&self.new_order.cid),
                            // TODO: inject mediator for vendor dispute handling
                            mediator: String::from(&mediator),
                            pid: String::from(&self.new_order.pid),
                            ship_address: encrypted_shipping_address.unwrap_or(Vec::new()),
                            quantity: qty,
                            ..Default::default()
                        };
                        log::debug!("new order: {:?}", &new_order);
                        self.is_loading = true;
                        submit_order_req(
                            self.submit_order_tx.clone(),
                            self.vendor_status.i2p.clone(),
                            ctx.clone(),
                            self.vendor_status.jwp.clone(),
                            new_order,
                        );
                        self.new_order = Default::default();
                        self.new_order_price = 0;
                        self.new_order_quantity = utils::empty_string();
                        self.new_order_shipping_address = utils::empty_string();
                        self.is_showing_products = false;
                    }
                }
                ui.label("\n");
                if ui.button("Exit").clicked() {
                    self.is_ordering = false;
                    self.is_loading = false;
                }
            });

        // View vendors
        //-----------------------------------------------------------------------------------
        let mut is_showing_vendors = self.is_showing_vendors;
        egui::Window::new("vendors")
            .open(&mut is_showing_vendors)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Vendors");
                // Vendor filter
                //-----------------------------------------------------------------------------------
                ui.heading("\nFind Vendor");
                ui.label("\n");
                ui.horizontal(|ui| {
                    let find_vendor_label = ui.label("filter vendors: ");
                    ui.text_edit_singleline(&mut self.find_vendor)
                        .labelled_by(find_vendor_label.id);
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
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
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
                        for v in &self.vendors {
                            if v.i2p_address.contains(&self.find_vendor) {
                                let row_height = 20.0;
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.label("vendor");
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{}", v.i2p_address));
                                    });
                                    row.col(|ui| {
                                        if ui.button("Check Status").clicked() {
                                            let nick_db = utils::search_gui_db(
                                                String::from(crate::GUI_NICK_DB_KEY),
                                                String::from(&v.i2p_address),
                                            );
                                            let nick = if nick_db == utils::empty_string() {
                                                String::from("anon")
                                            } else {
                                                nick_db
                                            };
                                            self.vendor_status.nick = nick;
                                            self.vendor_status.i2p = String::from(&v.i2p_address);
                                            // get the txp
                                            self.vendor_status.txp = utils::search_gui_db(
                                                String::from(crate::GUI_TX_PROOF_DB_KEY),
                                                String::from(&v.i2p_address),
                                            );
                                            // get the jwp
                                            self.vendor_status.jwp = utils::search_gui_db(
                                                String::from(crate::GUI_JWP_DB_KEY),
                                                String::from(&v.i2p_address),
                                            );
                                            let r_exp = utils::search_gui_db(
                                                String::from(crate::GUI_EXP_DB_KEY),
                                                String::from(&v.i2p_address),
                                            );
                                            self.vendor_status.exp = r_exp;
                                            let expire = match self.vendor_status.exp.parse::<i64>()
                                            {
                                                Ok(n) => n,
                                                Err(_e) => 0,
                                            };
                                            self.vendor_status.h_exp =
                                                chrono::NaiveDateTime::from_timestamp_opt(
                                                    expire, 0,
                                                )
                                                .unwrap()
                                                .to_string();
                                            // MESSAGES WON'T BE SENT UNTIL KEY IS SIGNED AND
                                            // TRUSTED!
                                            self.vendor_status.signed_key =
                                                check_signed_key(self.vendor_status.i2p.clone());
                                            send_contact_info_req(
                                                self.contact_info_tx.clone(),
                                                ctx.clone(),
                                                self.vendor_status.i2p.clone(),
                                                contact::Prune::Pruned.value(),
                                            );
                                            vendor_status_timeout(
                                                self.contact_timeout_tx.clone(),
                                                ctx.clone(),
                                            );
                                            self.is_showing_vendor_status = true;
                                            self.is_pinging = true;
                                        }
                                    });
                                    row.col(|ui| {
                                        let now = chrono::offset::Utc::now().timestamp();
                                        let expire = match self.vendor_status.exp.parse::<i64>() {
                                            Ok(n) => n,
                                            Err(_e) => 0,
                                        };
                                        if now < expire
                                            && self.vendor_status.signed_key
                                            && self.vendor_status.jwp != utils::empty_string()
                                            && v.i2p_address == self.vendor_status.i2p
                                            && self.vendor_status.is_vendor
                                        {
                                            if ui.button("View Products").clicked() {
                                                self.is_loading = true;
                                                send_products_from_vendor_req(
                                                    self.get_vendor_products_tx.clone(),
                                                    ctx.clone(),
                                                    self.vendor_status.i2p.clone(),
                                                    self.vendor_status.jwp.clone(),
                                                );
                                                self.is_window_shopping = true;
                                                self.is_showing_products = true;
                                                self.is_showing_vendors = false;
                                            }
                                        }
                                    });
                                });
                            }
                        }
                    });
                if ui.button("Exit").clicked() {
                    self.is_showing_vendors = false;
                }
            });

        // Vendor status window
        //-----------------------------------------------------------------------------------
        let mut is_showing_vendor_status = self.is_showing_vendor_status;
        egui::Window::new(&self.vendor_status.i2p)
            .title_bar(false)
            .open(&mut is_showing_vendor_status)
            .vscroll(true)
            .id(egui::Id::new(self.vendor_status.i2p.clone()))
            .show(&ctx, |ui| {
                if self.is_pinging {
                    ui.add(egui::Spinner::new());
                    ui.label("pinging...");
                }
                let status = if self.s_contact.xmr_address != utils::empty_string() {
                    "online"
                } else {
                    "offline"
                };
                let mode = if self.vendor_status.is_vendor {
                    "enabled "
                } else {
                    "disabled"
                };
                ui.label(format!("status: {}", status));
                ui.label(format!("vendor mode: {}", mode));
                ui.label(format!("nick: {}", self.vendor_status.nick));
                ui.label(format!("tx proof: {}", self.vendor_status.txp));
                ui.label(format!("jwp: {}", self.vendor_status.jwp));
                ui.label(format!("expiration: {}", self.vendor_status.h_exp));
                ui.label(format!("signed key: {}", self.vendor_status.signed_key));
                if ui.button("Exit").clicked() {
                    self.is_showing_vendor_status = false;
                }
            });

        // Product image window
        //-----------------------------------------------------------------------------------
        let mut is_showing_product_image = self.is_showing_product_image;
        egui::Window::new("product image")
            .open(&mut is_showing_product_image)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                self.product_image.show(ui);
                if ui.button("Exit").clicked() {
                    self.is_showing_product_image = false;
                    self.is_product_image_set = false;
                    let read_product_image = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
                    self.product_image =
                        egui_extras::RetainedImage::from_image_bytes("qr.png", &read_product_image)
                            .unwrap();
                }
            });

        // Products Management window
        //-----------------------------------------------------------------------------------
        let mut is_showing_products = self.is_showing_products;
        egui::Window::new("product management")
            .open(&mut is_showing_products)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Products");
                use egui_extras::{
                    Column,
                    TableBuilder,
                };
                if self.is_loading {
                    ui.add(egui::Spinner::new());
                    ui.label("loading...");
                }
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Description");
                        });
                        header.col(|ui| {
                            ui.strong("Price");
                        });
                        header.col(|ui| {
                            ui.strong("Quantity");
                        });
                        header.col(|ui| {
                            ui.strong("Image");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                    })
                    .body(|mut body| {
                        for p in &self.products {
                            let row_height = 20.0;
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{}", p.name));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", p.description));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", p.price));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", p.qty));
                                });
                                row.col(|ui| {
                                    if ui.button("View").clicked() {
                                        if !self.is_product_image_set {
                                            let file_path = format!(
                                                "/home/{}/.neveko/{}.jpeg",
                                                std::env::var("USER")
                                                    .unwrap_or(String::from("user")),
                                                p.pid
                                            );
                                            // For the sake of brevity product list doesn't have
                                            // image bytes, get them
                                            if self.is_window_shopping {
                                                self.is_loading = true;
                                                send_product_from_vendor_req(
                                                    self.get_vendor_product_tx.clone(),
                                                    ctx.clone(),
                                                    self.vendor_status.i2p.clone(),
                                                    self.vendor_status.jwp.clone(),
                                                    String::from(&p.pid),
                                                );
                                            } else {
                                                let i_product = product::find(&p.pid);
                                                match std::fs::write(&file_path, &i_product.image) {
                                                    Ok(w) => w,
                                                    Err(_) => {
                                                        log::error!("failed to write product image")
                                                    }
                                                };
                                                let contents =
                                                std::fs::read(&file_path).unwrap_or(Vec::new());
                                                if !i_product.image.is_empty() {
                                                    // this image should uwrap if vendor image bytes are
                                                    // bad
                                                    let default_img = std::fs::read("./assets/qr.png")
                                                        .unwrap_or(Vec::new());
                                                    let default_r_img =
                                                        egui_extras::RetainedImage::from_image_bytes(
                                                            "qr.png",
                                                            &default_img,
                                                        )
                                                        .unwrap();
                                                    self.product_image =
                                                        egui_extras::RetainedImage::from_image_bytes(
                                                            file_path, &contents,
                                                        )
                                                        .unwrap_or(default_r_img);
                                                }
                                            }
                                            if !self.is_window_shopping {
                                                self.is_product_image_set = true;
                                                self.is_showing_product_image = true;
                                            }
                                            ctx.request_repaint();
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.horizontal(|ui| {
                                        if !self.is_window_shopping {
                                            if ui.button("Update").clicked() {
                                                self.product_update_pid = p.pid.clone();
                                                self.new_product_desc = p.description.clone();
                                                self.new_product_name = p.name.clone();
                                                self.new_product_price = format!("{}", p.price);
                                                self.new_product_qty = format!("{}", p.qty);
                                                self.is_showing_product_update = true;
                                            }
                                        } else {
                                            if ui.button("Create Order").clicked() {
                                                self.new_order.pid = p.pid.clone();
                                                self.new_order.cid = i2p::get_destination(None);
                                                self.new_order_price = p.price;
                                                self.is_ordering = true;
                                            }
                                        }
                                    });
                                });
                            });
                        }
                    });
                if ui.button("Exit").clicked() {
                    self.is_showing_products = false;
                }
            });

        // Vendor specific

        // Update Product window
        //-----------------------------------------------------------------------------------
        let mut is_showing_product_update = self.is_showing_product_update;
        egui::Window::new("update product")
            .open(&mut is_showing_product_update)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading(format!("Update Product - {}", self.new_product_name));
                ui.label(
                    "____________________________________________________________________________\n",
                );
                // TODO(c2m): file picker for images
                ui.horizontal(|ui| {
                    let product_name = ui.label("image:   \t\t\t");
                    ui.text_edit_singleline(&mut self.new_product_image)
                        .labelled_by(product_name.id);
                    ui.label("\t/path/to/image.png");
                });
                ui.horizontal(|ui| {
                    let product_name = ui.label("name:    \t\t\t");
                    ui.text_edit_singleline(&mut self.new_product_name)
                        .labelled_by(product_name.id);
                });
                ui.horizontal(|ui| {
                    let product_desc = ui.label("description: \t");
                    ui.text_edit_singleline(&mut self.new_product_desc)
                        .labelled_by(product_desc.id);
                });
                ui.horizontal(|ui| {
                    let product_price = ui.label("price:     \t\t\t");
                    ui.text_edit_singleline(&mut self.new_product_price)
                        .labelled_by(product_price.id);
                    ui.label("\t (piconeros)")
                });
                ui.horizontal(|ui| {
                    let product_qty = ui.label("quantity:  \t\t");
                    ui.text_edit_singleline(&mut self.new_product_qty)
                        .labelled_by(product_qty.id);
                });
                ui.label("\n");
                if ui.button("Update Product").clicked() {
                    let image: Vec<u8> = std::fs::read(self.new_product_image.clone()).unwrap_or_default();
                    let price = match self.new_product_price.parse::<u128>() {
                        Ok(p) => p,
                        Err(_) => 0,
                    };
                    let qty = match self.new_product_qty.parse::<u128>() {
                        Ok(q) => q,
                        Err(_) => 0,
                    };
                    let product: models::Product = models::Product {
                        pid: self.product_update_pid.clone(),
                        description: self.new_product_desc.clone(),
                        image,
                        in_stock: qty > 0,
                        name: self.new_product_name.clone(),
                        price,
                        qty,
                    };
                    let j_product = utils::product_to_json(&product);
                    product::modify(j_product);
                    self.new_product_desc = utils::empty_string();
                    self.new_product_name = utils::empty_string();
                    self.new_product_price = utils::empty_string();
                    self.new_product_qty = utils::empty_string();
                    self.new_product_image = utils::empty_string();
                    self.is_showing_product_update = false;
                    self.products = product::find_all();
                }
                if ui.button("Exit").clicked() {
                    self.new_product_desc = utils::empty_string();
                    self.new_product_name = utils::empty_string();
                    self.new_product_price = utils::empty_string();
                    self.new_product_qty = utils::empty_string();
                    self.new_product_image = utils::empty_string();
                    self.is_showing_product_update = false;
                }
            });

        // Vendor Orders window
        //-----------------------------------------------------------------------------------
        let mut is_showing_orders = self.is_showing_orders;
        egui::Window::new("manage orders")
            .open(&mut is_showing_orders)
            .title_bar(false)
            .vscroll(true)
            .show(&ctx, |ui| {
                ui.heading("Manage Orders");
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
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                    })
                    .body(|mut body| {
                        for o in &self.orders {
                            let row_height = 20.0;
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{}", o.cid));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", o.status));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", o.date));
                                });
                                row.col(|ui| {
                                    ui.label(format!("{}", o.subaddress));
                                });
                                row.col(|ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.horizontal(|_ui| {
                                        // update button
                                    });
                                });
                            });
                        }
                    });
            });

        // End Vendor specific

        // Market Dashboard Main window
        //-----------------------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⚠ Experimental Multisig ⚠")
                        .small()
                        .color(ui.visuals().warn_fg_color),
                )
                .on_hover_text("monero multisig is experimental and usage of neveko may lead to loss of funds.");
            });
            if ui.button("Refresh").clicked() {
                self.products = product::find_all();
                self.orders = order::find_all();
            }
            ui.horizontal(|ui| {
                let vendor_mode: &str = if self.is_vendor_enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                ui.label(format!("vendor mode: {} \t", vendor_mode));
                if ui.button("toggle").clicked() {
                    self.is_vendor_enabled = utils::toggle_vendor_enabled();
                }
            });
            if ui.button("View Vendors").clicked() {
                // assume all contacts are vendors until updated status check
                self.vendors = contact::find_all();
                self.is_showing_vendors = true;
            }
            ui.label("\n");
            if ui.button("View Orders").clicked() {
                self.customer_orders = order::find_all_backup();
                self.is_customer_viewing_orders = true;
            }
            if self.is_vendor_enabled {
                ui.label("\n");
                ui.heading("Add Product");
                ui.label(
                    "____________________________________________________________________________\n",
                );
                // TODO(c2m): file picker for images
                ui.horizontal(|ui| {
                    let product_name = ui.label("image:   \t\t\t");
                    ui.text_edit_singleline(&mut self.new_product_image)
                        .labelled_by(product_name.id);
                    ui.label("\t/path/to/image.png");
                });
                ui.horizontal(|ui| {
                    let product_name = ui.label("name:    \t\t\t");
                    ui.text_edit_singleline(&mut self.new_product_name)
                        .labelled_by(product_name.id);
                });
                ui.horizontal(|ui| {
                    let product_desc = ui.label("description: \t");
                    ui.text_edit_singleline(&mut self.new_product_desc)
                        .labelled_by(product_desc.id);
                });
                ui.horizontal(|ui| {
                    let product_price = ui.label("price:     \t\t\t");
                    ui.text_edit_singleline(&mut self.new_product_price)
                        .labelled_by(product_price.id);
                    ui.label("\t (piconeros)")
                });
                ui.horizontal(|ui| {
                    let product_qty = ui.label("quantity:  \t\t");
                    ui.text_edit_singleline(&mut self.new_product_qty)
                        .labelled_by(product_qty.id);
                });
                if ui.button("Add Product").clicked() {
                    let image: Vec<u8> = std::fs::read(self.new_product_image.clone()).unwrap_or_default();
                    let price = match self.new_product_price.parse::<u128>() {
                        Ok(p) => p,
                        Err(_) => 0,
                    };
                    let qty = match self.new_product_qty.parse::<u128>() {
                        Ok(q) => q,
                        Err(_) => 0,
                    };
                    let product: models::Product = models::Product {
                        pid: utils::empty_string(),
                        description: self.new_product_desc.clone(),
                        image,
                        in_stock: qty > 0,
                        name: self.new_product_name.clone(),
                        price,
                        qty,
                    };
                    let j_product = utils::product_to_json(&product);
                    product::create(j_product);
                    self.new_product_desc = utils::empty_string();
                    self.new_product_name = utils::empty_string();
                    self.new_product_price = utils::empty_string();
                    self.new_product_qty = utils::empty_string();
                    self.new_product_image = utils::empty_string();
                }
                ui.label("\n");
                if ui.button("View Products").clicked() {
                    self.products = product::find_all();
                    self.is_showing_products = true;
                    self.is_showing_vendors = false;
                }
                ui.label("\n");
                if ui.button("Manage Orders").clicked() {
                    // TODO(c2m): vendor order management logic
                }
            }
        });
    }
}

// Async fn requests
fn _refresh_on_delete_product_req(_tx: Sender<bool>, _ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::error!("refreshing products....");
        todo!();
        // let _ = tx.send(true);
        // ctx.request_repaint();
    });
}

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

fn check_signed_key(contact: String) -> bool {
    let v = utils::search_gui_db(String::from(crate::GUI_SIGNED_GPG_DB_KEY), contact);
    v != utils::empty_string()
}

fn send_products_from_vendor_req(
    tx: Sender<Vec<models::Product>>,
    ctx: egui::Context,
    contact: String,
    jwp: String,
) {
    log::debug!("fetching products for vendor: {}", contact);
    tokio::spawn(async move {
        let result = product::get_vendor_products(contact, jwp).await;
        if result.is_ok() {
            let products: Vec<models::Product> = result.unwrap();
            log::info!("retreived {:?} products", products.len());
            let _ = tx.send(products);
            ctx.request_repaint();
        }
    });
}

fn send_product_from_vendor_req(
    tx: Sender<models::Product>,
    ctx: egui::Context,
    contact: String,
    jwp: String,
    pid: String,
) {
    log::debug!("fetching product {} from vendor: {}", &pid, contact);
    tokio::spawn(async move {
        let result = product::get_vendor_product(contact, jwp, pid).await;
        if result.is_ok() {
            let product: models::Product = result.unwrap();
            log::info!("retrieved product {}", &product.pid);
            let _ = tx.send(product);
            ctx.request_repaint();
        }
    });
}

fn vendor_status_timeout(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(
            crate::ADD_CONTACT_TIMEOUT_SECS,
        ))
        .await;
        log::error!("vendor status timeout");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn submit_order_req(
    tx: Sender<models::Order>,
    contact: String,
    ctx: egui::Context,
    jwp: String,
    request: reqres::OrderRequest,
) {
    tokio::spawn(async move {
        log::info!("submit order");
        let r_contact = String::from(&contact);
        let order = order::transmit_order_request(r_contact, jwp, request).await;
        let u_order = order.unwrap_or_else(|_| Default::default());
        // cache order request to db
        order::backup(&u_order);
        let prefix = String::from(crate::GUI_OVL_DB_KEY);
        let orid = String::from(&u_order.orid);
        let i2p = String::from(&contact);
        utils::write_gui_db(prefix, orid, i2p);
        let _ = tx.send(u_order);
        ctx.request_repaint();
    });
}

fn send_prepare_info_req(
    tx: Sender<String>,
    ctx: egui::Context,
    mediator: String,
    orid: &String,
    vendor: String,
) {
    let m_orid: String = String::from(orid);
    let v_orid: String = String::from(orid);
    let w_orid: String = String::from(orid);
    tokio::spawn(async move {
        let m_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&mediator));
        let v_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&vendor));
        let wallet_password = utils::empty_string();
        monero::create_wallet(&w_orid, &wallet_password).await;
        let m_wallet = monero::open_wallet(&w_orid, &wallet_password).await;
        if !m_wallet {
            log::error!("failed to open wallet");
            monero::close_wallet(&w_orid, &wallet_password).await;
            let _ = tx.send(utils::empty_string());
            return;
        }
        // enable multisig
        monero::close_wallet(&w_orid, &wallet_password).await;
        monero::enable_experimental_multisig(&w_orid);
        monero::open_wallet(&w_orid, &wallet_password).await;
        let prepare_info = monero::prepare_wallet().await;
        let ref_prepare_info: &String = &prepare_info.result.multisig_info;
        utils::write_gui_db(
            String::from(crate::GUI_MSIG_PREPARE_DB_KEY),
            String::from(&w_orid),
            String::from(ref_prepare_info),
        );
        // Request mediator and vendor while we're at it
        // Will coordinating send this on make requests next

        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::PREPARE_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::PREPARE_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_prepare = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_prepare = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        if v_prepare == utils::empty_string() {
            log::debug!(
                "constructing vendor {} msig messages",
                message::PREPARE_MSIG
            );
            let v_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: Vec::new(),
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::PREPARE_MSIG),
                orid: String::from(v_orid),
            };
            let _v_result = message::d_trigger_msig_info(&vendor, &v_jwp, &v_msig_request).await;
        }
        if m_prepare == utils::empty_string() {
            log::debug!(
                "constructing mediator {} msig messages",
                message::PREPARE_MSIG
            );
            let m_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: Vec::new(),
                init_mediator: true,
                kex_init: false,
                msig_type: String::from(message::PREPARE_MSIG),
                orid: String::from(m_orid),
            };
            let _m_result = message::d_trigger_msig_info(&mediator, &m_jwp, &m_msig_request).await;
        }
        let _ = tx.send(String::from(ref_prepare_info));
    });
    ctx.request_repaint();
}

fn send_make_info_req(
    tx: Sender<String>,
    ctx: egui::Context,
    mediator: String,
    orid: &String,
    vendor: String,
) {
    let m_orid: String = String::from(orid);
    let v_orid: String = String::from(orid);
    let w_orid: String = String::from(orid);
    tokio::spawn(async move {
        let m_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&mediator));
        let v_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&vendor));
        let wallet_password = utils::empty_string();
        let m_wallet = monero::open_wallet(&w_orid, &wallet_password).await;
        if !m_wallet {
            monero::close_wallet(&w_orid, &wallet_password).await;
            log::error!("failed to open wallet");
            let _ = tx.send(utils::empty_string());
            return;
        }
        let mut prepare_info_prep = Vec::new();
        let mut m_prepare_info_send = Vec::new();
        let mut v_prepare_info_send = Vec::new();
        // we need to send our info to mediator and vendor so they can perform
        // make_multisig and send the reponse (String) back
        let c_prepare = utils::search_gui_db(
            String::from(crate::GUI_MSIG_PREPARE_DB_KEY),
            String::from(&w_orid),
        );
        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::PREPARE_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::PREPARE_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_prepare = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_prepare = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        prepare_info_prep.push(String::from(&m_prepare));
        prepare_info_prep.push(String::from(&v_prepare));
        m_prepare_info_send.push(String::from(&c_prepare));
        m_prepare_info_send.push(String::from(&v_prepare));
        v_prepare_info_send.push(String::from(&m_prepare));
        v_prepare_info_send.push(String::from(&c_prepare));
        let local_make = utils::search_gui_db(
            String::from(crate::GUI_MSIG_MAKE_DB_KEY),
            String::from(&w_orid),
        );
        if local_make == utils::empty_string() {
            let make_info = monero::make_wallet(prepare_info_prep).await;
            monero::close_wallet(&w_orid, &wallet_password).await;
            let ref_make_info: &String = &make_info.result.multisig_info;
            if String::from(ref_make_info) != utils::empty_string() {
                utils::write_gui_db(
                    String::from(crate::GUI_MSIG_MAKE_DB_KEY),
                    String::from(&w_orid),
                    String::from(ref_make_info),
                );
            }
        }
        // Request mediator and vendor while we're at it
        // Will coordinating send this on make requests next

        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::MAKE_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::MAKE_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_make = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_make = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        if v_make == utils::empty_string() {
            log::debug!("constructing vendor {} msig messages", message::MAKE_MSIG);
            let v_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: v_prepare_info_send,
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::MAKE_MSIG),
                orid: String::from(v_orid),
            };
            let _v_result = message::d_trigger_msig_info(&vendor, &v_jwp, &v_msig_request).await;
        }
        if m_make == utils::empty_string() {
            log::debug!("constructing mediator {} msig messages", message::MAKE_MSIG);
            let m_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: m_prepare_info_send,
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::MAKE_MSIG),
                orid: String::from(m_orid),
            };
            let _m_result = message::d_trigger_msig_info(&mediator, &m_jwp, &m_msig_request).await;
        }
        let _ = tx.send(String::from(&local_make));
    });
    ctx.request_repaint();
}

fn send_kex_initial_req(
    tx: Sender<String>,
    ctx: egui::Context,
    mediator: String,
    orid: &String,
    vendor: String,
) {
    let m_orid: String = String::from(orid);
    let v_orid: String = String::from(orid);
    let w_orid: String = String::from(orid);
    tokio::spawn(async move {
        let m_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&mediator));
        let v_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&vendor));
        let wallet_password = utils::empty_string();
        let m_wallet = monero::open_wallet(&w_orid, &wallet_password).await;
        if !m_wallet {
            monero::close_wallet(&w_orid, &wallet_password).await;
            log::error!("failed to open wallet");
            let _ = tx.send(utils::empty_string());
            return;
        }
        let mut kex_init_prep = Vec::new();
        let mut m_kex_init_send = Vec::new();
        let mut v_kex_init_send = Vec::new();
        // we need to send our info to mediator and vendor so they can perform
        // kex final one and send the reponse (info) back
        let c_kex_init = utils::search_gui_db(
            String::from(crate::GUI_MSIG_MAKE_DB_KEY),
            String::from(&w_orid),
        );
        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::MAKE_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::MAKE_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_kex_init = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_kex_init = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        kex_init_prep.push(String::from(&m_kex_init));
        kex_init_prep.push(String::from(&v_kex_init));
        m_kex_init_send.push(String::from(&c_kex_init));
        m_kex_init_send.push(String::from(&v_kex_init));
        v_kex_init_send.push(String::from(&m_kex_init));
        v_kex_init_send.push(String::from(&c_kex_init));
        let local_kex_init = utils::search_gui_db(
            String::from(crate::GUI_MSIG_KEX_ONE_DB_KEY),
            String::from(&w_orid),
        );
        if local_kex_init == utils::empty_string() {
            let kex_out =
                monero::exchange_multisig_keys(false, kex_init_prep, &wallet_password).await;
            monero::close_wallet(&w_orid, &wallet_password).await;
            let ref_kex_info: &String = &kex_out.result.multisig_info;
            if String::from(ref_kex_info) != utils::empty_string() {
                utils::write_gui_db(
                    String::from(crate::GUI_MSIG_KEX_ONE_DB_KEY),
                    String::from(&w_orid),
                    String::from(ref_kex_info),
                );
            }
        }
        // Request mediator and vendor while we're at it
        // Will coordinating send this on kex round two next
        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::KEX_ONE_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::KEX_ONE_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_kex_init = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_kex_init = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        if v_kex_init == utils::empty_string() {
            log::debug!(
                "constructing vendor {} msig messages",
                message::KEX_ONE_MSIG
            );
            let v_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: v_kex_init_send,
                init_mediator: false,
                kex_init: true,
                msig_type: String::from(message::KEX_ONE_MSIG),
                orid: String::from(v_orid),
            };
            let _v_result = message::d_trigger_msig_info(&vendor, &v_jwp, &v_msig_request).await;
        }
        if m_kex_init == utils::empty_string() {
            log::debug!(
                "constructing mediator {} msig messages",
                message::KEX_ONE_MSIG
            );
            let m_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: m_kex_init_send,
                init_mediator: false,
                kex_init: true,
                msig_type: String::from(message::KEX_ONE_MSIG),
                orid: String::from(m_orid),
            };
            let _m_result = message::d_trigger_msig_info(&mediator, &m_jwp, &m_msig_request).await;
        }
        let _ = tx.send(String::from(&local_kex_init));
    });
    ctx.request_repaint();
}

fn send_kex_final_req(
    tx: Sender<String>,
    ctx: egui::Context,
    mediator: String,
    orid: &String,
    vendor: String,
) {
    let m_orid: String = String::from(orid);
    let v_orid: String = String::from(orid);
    let w_orid: String = String::from(orid);
    tokio::spawn(async move {
        let m_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&mediator));
        let v_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&vendor));
        let wallet_password = utils::empty_string();
        let m_wallet = monero::open_wallet(&w_orid, &wallet_password).await;
        if !m_wallet {
            monero::close_wallet(&w_orid, &wallet_password).await;
            log::error!("failed to open wallet");
            let _ = tx.send(utils::empty_string());
            return;
        }
        let mut kex_final_prep = Vec::new();
        let mut m_kex_final_send = Vec::new();
        let mut v_kex_final_send = Vec::new();
        let c_kex_final = utils::search_gui_db(
            String::from(crate::GUI_MSIG_KEX_ONE_DB_KEY),
            String::from(&w_orid),
        );
        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::KEX_ONE_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::KEX_ONE_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_kex_final = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_kex_final = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        kex_final_prep.push(String::from(&m_kex_final));
        kex_final_prep.push(String::from(&v_kex_final));
        m_kex_final_send.push(String::from(&c_kex_final));
        m_kex_final_send.push(String::from(&v_kex_final));
        v_kex_final_send.push(String::from(&m_kex_final));
        v_kex_final_send.push(String::from(&c_kex_final));
        let local_kex_final = utils::search_gui_db(
            String::from(crate::GUI_MSIG_KEX_TWO_DB_KEY),
            String::from(&w_orid),
        );
        if local_kex_final == utils::empty_string() {
            let kex_out =
                monero::exchange_multisig_keys(false, kex_final_prep, &wallet_password).await;
            monero::close_wallet(&w_orid, &wallet_password).await;
            let ref_kex_info: &String = &kex_out.result.address;
            if String::from(ref_kex_info) != utils::empty_string() {
                utils::write_gui_db(
                    String::from(crate::GUI_MSIG_KEX_TWO_DB_KEY),
                    String::from(&w_orid),
                    String::from(ref_kex_info),
                );
            }
        }
        // we can verify all good if the senders all send back the correct wallet address
        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::KEX_TWO_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::KEX_TWO_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_kex_final = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_kex_final = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        if v_kex_final == utils::empty_string() {
            log::debug!(
                "constructing vendor {} msig messages",
                message::KEX_TWO_MSIG
            );
            let v_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: v_kex_final_send,
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::KEX_TWO_MSIG),
                orid: String::from(v_orid),
            };
            let _v_result = message::d_trigger_msig_info(&vendor, &v_jwp, &v_msig_request).await;
        }
        if m_kex_final == utils::empty_string() {
            log::debug!(
                "constructing mediator {} msig messages",
                message::KEX_TWO_MSIG
            );
            let m_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: m_kex_final_send,
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::KEX_TWO_MSIG),
                orid: String::from(m_orid),
            };
            let _m_result = message::d_trigger_msig_info(&mediator, &m_jwp, &m_msig_request).await;
        }
        let _ = tx.send(String::from(&local_kex_final));
    });
    ctx.request_repaint();
}

fn set_order_address(orid: &String, tx: Sender<reqres::XmrRpcAddressResponse>, ctx: egui::Context) {
    let order_id = String::from(orid);
    tokio::spawn(async move {
        let wallet_password = utils::empty_string();
        monero::open_wallet(&order_id, &wallet_password).await;
        let address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        monero::close_wallet(&order_id, &wallet_password).await;
        let _ = tx.send(address);
        ctx.request_repaint();
    });
}

fn verify_order_wallet_funded(contact: &String, orid: &String, tx: Sender<bool>, ctx: egui::Context) {
    let order_id = String::from(orid);
    let l_contact = String::from(contact);
    tokio::spawn(async move {
        let wallet_password = utils::empty_string();
        monero::open_wallet(&order_id, &wallet_password).await;
        let _ = monero::refresh().await;
        let pre_bal = monero::get_balance().await;
        let is_msig_res = monero::is_multisig().await;
        if !is_msig_res.result.multisig || !is_msig_res.result.ready {
            let _ = tx.send(false);
            return;
        }
        monero::close_wallet(&order_id, &wallet_password).await;
        let order = order::find(&order_id);
        let vendor = String::from(&l_contact);
        let v_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&vendor));
        let opid = String::from(&order.pid);
        let result = product::get_vendor_product(vendor, v_jwp, opid).await;
        if !result.is_ok() {
            let _ = tx.send(false);
            return;
        }
        let product: models::Product = result.unwrap();
        log::info!("retrieved product {}", &product.pid);
        let total = &order.quantity & &product.price;
        if pre_bal.result.balance < total {
            let _ = tx.send(false);
            return;
        }
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}

fn send_import_info_req(
    tx: Sender<String>,
    ctx: egui::Context,
    mediator: String,
    orid: &String,
    vendor: String,
) {
    let m_orid: String = String::from(orid);
    let v_orid: String = String::from(orid);
    let w_orid: String = String::from(orid);
    tokio::spawn(async move {
        let m_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&mediator));
        let v_jwp: String =
            utils::search_gui_db(String::from(crate::GUI_JWP_DB_KEY), String::from(&vendor));
        let wallet_password = utils::empty_string();
        let m_wallet = monero::open_wallet(&w_orid, &wallet_password).await;
        if !m_wallet {
            log::error!("failed to open wallet");
            monero::close_wallet(&w_orid, &wallet_password).await;
            let _ = tx.send(utils::empty_string());
            return;
        }
        let export_info = monero::export_multisig_info().await;
        let ref_export_info: &String = &export_info.result.info;
        utils::write_gui_db(
            String::from(crate::GUI_MSIG_EXPORT_DB_KEY),
            String::from(&w_orid),
            String::from(ref_export_info),
        );
        // Request mediator and vendor while we're at it
        // Will coordinating send this on make requests next
        let s = db::Interface::async_open().await;
        let m_msig_key = format!(
            "{}-{}-{}",
            message::EXPORT_MSIG,
            String::from(&m_orid),
            mediator
        );
        let v_msig_key = format!(
            "{}-{}-{}",
            message::EXPORT_MSIG,
            String::from(&v_orid),
            vendor
        );
        let m_export = db::Interface::async_read(&s.env, &s.handle, &m_msig_key).await;
        let v_export = db::Interface::async_read(&s.env, &s.handle, &v_msig_key).await;
        if v_export == utils::empty_string() {
            log::debug!(
                "constructing vendor {} msig messages",
                message::EXPORT_MSIG
            );
            let v_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: Vec::new(),
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::IMPORT_MSIG),
                orid: String::from(v_orid),
            };
            let _v_result = message::d_trigger_msig_info(&vendor, &v_jwp, &v_msig_request).await;
        }
        if m_export == utils::empty_string() {
            log::debug!(
                "constructing mediator {} msig messages",
                message::EXPORT_MSIG
            );
            let m_msig_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
                contact: i2p::get_destination(None),
                info: Vec::new(),
                init_mediator: false,
                kex_init: false,
                msig_type: String::from(message::EXPORT_MSIG),
                orid: String::from(m_orid),
            };
            let _m_result = message::d_trigger_msig_info(&mediator, &m_jwp, &m_msig_request).await;
        }
        let _ = tx.send(String::from(ref_export_info));
    });
    ctx.request_repaint();
}
// End Async fn requests

fn validate_msig_step(
    mediator: &String,
    orid: &String,
    vendor: &String,
    sub_type: &String,
) -> bool {
    let s = db::Interface::open();
    let m_msig_key = format!("{}-{}-{}", sub_type, orid, mediator);
    let v_msig_key = format!("{}-{}-{}", sub_type, orid, vendor);
    let m_info = db::Interface::read(&s.env, &s.handle, &m_msig_key);
    let v_info = db::Interface::read(&s.env, &s.handle, &v_msig_key);
    log::debug!("mediator info: {}", &m_info);
    log::debug!("vendor info: {}", &v_info);
    m_info != utils::empty_string() && v_info != utils::empty_string()
}
