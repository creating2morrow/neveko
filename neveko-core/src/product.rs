// Product repo/service layer
use crate::{
    db,
    models::*,
    utils,
};
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;
use std::error::Error;

/// Create a new product
pub fn create(d: Json<Product>) -> Product {
    let pid: String = format!("product{}", utils::generate_rnd());
    if !validate_product(&d) {
        error!("invalid product");
        return Default::default();
    }
    let new_product = Product {
        pid: String::from(&pid),
        description: String::from(&d.description),
        image: d.image.iter().cloned().collect(),
        in_stock: d.in_stock,
        name: String::from(&d.name),
        price: d.price,
        qty: d.qty,
    };
    debug!("insert product: {:?}", &new_product);
    let s = db::Interface::open();
    let k = &new_product.pid;
    db::Interface::write(&s.env, &s.handle, k, &Product::to_db(&new_product));
    // in order to retrieve all products, write keys to with pl
    let list_key = format!("pl");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    if r == utils::empty_string() {
        debug!("creating product index");
    }
    let product_list = [r, String::from(&pid)].join(",");
    debug!(
        "writing product index {} for id: {}",
        product_list, list_key
    );
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &product_list);
    new_product
}

/// Single Product lookup
pub fn find(pid: &String) -> Product {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(pid));
    if r == utils::empty_string() {
        error!("product not found");
        return Default::default();
    }
    Product::from_db(String::from(pid), r)
}

/// Product lookup for all
pub fn find_all() -> Vec<Product> {
    let i_s = db::Interface::open();
    let i_list_key = format!("pl");
    let i_r = db::Interface::read(&i_s.env, &i_s.handle, &String::from(i_list_key));
    if i_r == utils::empty_string() {
        error!("product index not found");
    }
    let i_v_pid = i_r.split(",");
    let i_v: Vec<String> = i_v_pid.map(|s| String::from(s)).collect();
    let mut products: Vec<Product> = Vec::new();
    for p in i_v {
        let mut product: Product = find(&p);
        if product.pid != utils::empty_string() {
            // don't return images
            product.image = Vec::new();
            products.push(product);
        }
    }
    products
}

/// Modify product
pub fn modify(p: Json<Product>) -> Product {
    info!("modify product: {}", &p.pid);
    let f_prod: Product = find(&p.pid);
    if f_prod.pid == utils::empty_string() {
        error!("product not found");
        return Default::default();
    }
    let u_prod = Product::update(f_prod, &p);
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, &u_prod.pid);
    db::Interface::write(&s.env, &s.handle, &u_prod.pid, &Product::to_db(&u_prod));
    return u_prod;
}

/// check product field lengths to prevent db spam
fn validate_product(p: &Json<Product>) -> bool {
    info!("validating product: {}", &p.pid);
    p.pid.len() < utils::string_limit()
        && p.description.len() < utils::string_limit()
        && p.name.len() < utils::string_limit()
        && p.image.len() < utils::image_limit()
}

/// Send the request to vendor for the products available
pub async fn get_vendor_products(
    contact: String,
    jwp: String,
) -> Result<Vec<Product>, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .get(format!("http://{}/market/products", contact))
        .header("proof", jwp)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<Vec<Product>>().await;
            debug!("get vendor products response: {:?}", res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to fetch products due to: {:?}", e);
            Ok(Default::default())
        }
    }
}

/// Send the request to vendor a single product
pub async fn get_vendor_product(
    contact: String,
    jwp: String,
    pid: String,
) -> Result<Product, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .get(format!("http://{}/market/product/{}", contact, pid))
        .header("proof", jwp)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<Product>().await;
            debug!("get vendor product response: {:?}", res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to fetch product due to: {:?}", e);
            Ok(Default::default())
        }
    }
}
