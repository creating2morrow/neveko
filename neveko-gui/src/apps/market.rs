use neveko_core::*;
use std::sync::mpsc::{
    Receiver,
    Sender,
};

pub struct MarketApp {
    contact_info_tx: Sender<models::Contact>,
    contact_info_rx: Receiver<models::Contact>,
    contact_timeout_tx: Sender<bool>,
    contact_timeout_rx: Receiver<bool>,
    find_vendor: String,
    get_vendor_products_tx: Sender<Vec<models::Product>>,
    get_vendor_products_rx: Receiver<Vec<models::Product>>,
    get_vendor_product_tx: Sender<models::Product>,
    get_vendor_product_rx: Receiver<models::Product>,
    is_loading: bool,
    is_ordering: bool,
    is_pinging: bool,
    is_product_image_set: bool,
    is_showing_products: bool,
    is_showing_product_image: bool,
    is_showing_product_update: bool,
    is_showing_orders: bool,
    is_showing_vendor_status: bool,
    is_showing_vendors: bool,
    is_timeout: bool,
    is_vendor_enabled: bool,
    is_window_shopping: bool,
    orders: Vec<models::Order>,
    product_from_vendor: models::Product,
    product_image: egui_extras::RetainedImage,
    products: Vec<models::Product>,
    product_update_pid: String,
    new_product_image: String,
    new_product_name: String,
    new_product_desc: String,
    new_product_price: String,
    new_product_qty: String,
    _refresh_on_delete_product_tx: Sender<bool>,
    _refresh_on_delete_product_rx: Receiver<bool>,
    s_contact: models::Contact,
    vendor_status: utils::ContactStatus,
    vendors: Vec<models::Contact>,
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
        MarketApp {
            contact_info_rx,
            contact_info_tx,
            contact_timeout_rx,
            contact_timeout_tx,
            find_vendor: utils::empty_string(),
            get_vendor_products_rx,
            get_vendor_products_tx,
            get_vendor_product_rx,
            get_vendor_product_tx,
            is_loading: false,
            is_ordering: false,
            is_pinging: false,
            is_product_image_set: false,
            is_showing_orders: false,
            is_showing_products: false,
            is_showing_product_image: false,
            is_showing_product_update: false,
            is_showing_vendor_status: false,
            is_showing_vendors: false,
            is_timeout: false,
            is_vendor_enabled,
            is_window_shopping: false,
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
            vendor_status: Default::default(),
            vendors: Vec::new(),
        }
    }
}

impl eframe::App for MarketApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hook into async channel threads
        //-----------------------------------------------------------------------------------

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

        // TODO(c2m): create order form

        // View vendors
        //-----------------------------------------------------------------------------------
        let mut is_showing_vendors = self.is_showing_vendors;
        egui::Window::new("Vendors")
            .open(&mut is_showing_vendors)
            .vscroll(true)
            .show(&ctx, |ui| {
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
                                                String::from("gui-nick"),
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
                                                String::from("gui-txp"),
                                                String::from(&v.i2p_address),
                                            );
                                            // get the jwp
                                            self.vendor_status.jwp = utils::search_gui_db(
                                                String::from("gui-jwp"),
                                                String::from(&v.i2p_address),
                                            );
                                            log::debug!("jwp: {}", self.vendor_status.jwp);
                                            let r_exp = utils::search_gui_db(
                                                String::from("gui-exp"),
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
            .open(&mut is_showing_vendor_status)
            .vscroll(true)
            .title_bar(false)
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
        egui::Window::new("")
            .open(&mut is_showing_product_image)
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
        egui::Window::new("Products")
            .open(&mut is_showing_products)
            .vscroll(true)
            .show(&ctx, |ui| {
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
                                                self.product_update_pid = p.pid.clone();
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
        egui::Window::new(format!("Update Product - {}", self.new_product_name))
            .open(&mut is_showing_product_update)
            .vscroll(true)
            .show(ctx, |ui| {
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

        // TODO(c2m): Orders window
        //-----------------------------------------------------------------------------------
        let mut is_showing_orders = self.is_showing_orders;
        egui::Window::new("Orders")
            .open(&mut is_showing_orders)
            .vscroll(true)
            .show(&ctx, |ui| {
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
                // TODO(c2m):
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
            }
        });
    }
}

fn _refresh_on_delete_product_req(_tx: Sender<bool>, _ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::error!("refreshing products....");
        todo!();
        // let _ = tx.send(true);
        // ctx.request_repaint();
    });
}

fn send_contact_info_req(tx: Sender<models::Contact>, ctx: egui::Context, contact: String) {
    log::debug!("async send_contact_info_req");
    tokio::spawn(async move {
        match contact::add_contact_request(contact, 1).await {
            Ok(contact) => {
                let _ = tx.send(contact);
                ctx.request_repaint();
            }
            _ => log::debug!("failed to request invoice"),
        }
    });
}

fn check_signed_key(contact: String) -> bool {
    let v = utils::search_gui_db(String::from("gui-signed-key"), contact);
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
            log::info!("retreived product {}", &product.pid);
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
