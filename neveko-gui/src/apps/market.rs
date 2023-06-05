use neveko_core::*;
use std::sync::mpsc::{
    Receiver,
    Sender,
};

pub struct MarketApp {
    is_showing_products: bool,
    is_showing_orders: bool,
    orders: Vec<models::Order>,
    products: Vec<models::Product>,
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
        MarketApp {
            is_showing_orders: false,
            is_showing_products: false,
            orders: Vec::new(),
            products: Vec::new(),
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
                    .column(Column::initial(100.0).at_least(40.0).clip(true))
                    .column(Column::initial(100.0).at_least(40.0).clip(true))
                    .column(Column::initial(100.0).at_least(40.0).clip(true))
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
                            ui.strong("");
                        });
                    })
                    .body(|mut body| {
                        for p in &self.products {
                            let row_height = 200.0;
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
                                    ui.style_mut().wrap = Some(false);
                                    ui.horizontal(|_ui| {
                                        // update button
                                    });
                                });
                            });
                        }
                    });
            });

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
                            ui.strong("");
                        });
                    })
                    .body(|mut body| {
                        for o in &self.orders {
                            let row_height = 200.0;
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
            }
            ui.horizontal(|ui| {
                ui.label("vendor mode: \t");
                if ui.button("toggle").clicked() {
                    utils::toggle_vendor_enabled();
                }
            });
            ui.label("\n");
            ui.heading("Add Product");
            ui.label(
                "____________________________________________________________________________\n",
            );
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
            });
            ui.horizontal(|ui| {
                let product_qty = ui.label("quantity:  \t\t");
                ui.text_edit_singleline(&mut self.new_product_qty)
                    .labelled_by(product_qty.id);
            });
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
