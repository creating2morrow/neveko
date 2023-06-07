use neveko_core::*;
use std::sync::mpsc::{
    Receiver,
    Sender,
};

pub struct MarketApp {
    is_product_image_set: bool,
    is_showing_products: bool,
    is_showing_product_image: bool,
    is_showing_product_update: bool,
    is_showing_orders: bool,
    is_vendor_enabled: bool,
    orders: Vec<models::Order>,
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
}

impl Default for MarketApp {
    fn default() -> Self {
        let (_refresh_on_delete_product_tx, _refresh_on_delete_product_rx) =
            std::sync::mpsc::channel();
        let read_product_image = std::fs::read("./assets/qr.png").unwrap_or(Vec::new());
        MarketApp {
            is_product_image_set: false,
            is_showing_orders: false,
            is_showing_products: false,
            is_showing_product_image: false,
            is_showing_product_update: false,
            is_vendor_enabled: false,
            orders: Vec::new(),
            product_image: egui_extras::RetainedImage::from_image_bytes("qr.png", &read_product_image).unwrap(),
            products: Vec::new(),
            product_update_pid: utils::empty_string(),
            new_product_image: utils::empty_string(),
            new_product_name: utils::empty_string(),
            new_product_desc: utils::empty_string(),
            new_product_price: utils::empty_string(),
            new_product_qty: utils::empty_string(),
            _refresh_on_delete_product_tx,
            _refresh_on_delete_product_rx,
        }
    }
}

impl eframe::App for MarketApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hook into async channel threads
        //-----------------------------------------------------------------------------------

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
                    self.product_image = egui_extras::RetainedImage::from_image_bytes("qr.png", &read_product_image).unwrap();
                }
            });
        
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

        // Products window
        //-----------------------------------------------------------------------------------
        let mut is_showing_products = self.is_showing_products;
        let mut is_showing_orders = self.is_showing_orders;
        egui::Window::new("Products")
            .open(&mut is_showing_products)
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
                                            self.is_showing_product_image = true;
                                            let file_path = format!(
                                                "/home/{}/.neveko/{}.jpeg",
                                                std::env::var("USER").unwrap_or(String::from("user")),
                                                p.pid
                                            );
                                            // For the sake of brevity product list doesn't have image bytes, get them
                                            let i_product = product::find(&p.pid);
                                            match std::fs::write(&file_path, &i_product.image) {
                                                Ok(w) => w,
                                                Err(_) => log::error!("failed to write product image")
                                            };
                                            self.is_product_image_set = true;
                                            let contents = std::fs::read(&file_path).unwrap_or(Vec::new());
                                            if !i_product.image.is_empty() {
                                                self.product_image =
                                                egui_extras::RetainedImage::from_image_bytes(file_path, &contents).unwrap();
                                            }
                                            self.is_product_image_set = true;
                                            ctx.request_repaint();
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.horizontal(|ui| {
                                        if ui.button("Update").clicked() {
                                          self.product_update_pid = p.pid.clone();
                                          self.new_product_desc = p.description.clone();
                                          self.new_product_name = p.name.clone();
                                          self.new_product_price = format!("{}", p.price);
                                          self.new_product_qty = format!("{}", p.qty);
                                          self.is_showing_product_update = true;
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

        // TODO(c2m): Orders window
        //-----------------------------------------------------------------------------------
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
                    
            }
            ui.label("\n");
            if ui.button("View Orders").clicked() {
                    
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
                }
            }
        });
    }
}

fn _refresh_on_delete_product_req(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::error!("refreshing products....");
        let _ = tx.send(true);
        ctx.request_repaint();
    });
}
