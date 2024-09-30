//! Marketplace products upload, modification, etc module

use crate::{
    db::{
        self,
        DATABASE_LOCK,
    },
    error::NevekoError,
    models::*,
    utils,
};
use kn0sys_lmdb_rs::MdbError;
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;
use std::error::Error;

/// Create a new product
pub fn create(d: Json<Product>) -> Result<Product, NevekoError> {
    let pid: String = format!("{}{}", crate::PRODUCT_DB_KEY, utils::generate_rnd());
    if !validate_product(&d) {
        error!("invalid product");
        return Err(NevekoError::Database(MdbError::NotFound));
    }
    let new_product = Product {
        pid: String::from(&pid),
        description: String::from(&d.description),
        image: d.image.to_vec(),
        in_stock: d.in_stock,
        name: String::from(&d.name),
        price: d.price,
        qty: d.qty,
    };
    debug!("insert product: {:?}", &new_product);
    let db = &DATABASE_LOCK;
    let k = &new_product.pid;
    let product = bincode::serialize(&new_product).unwrap_or_default();
    db::write_chunks(&db.env, &db.handle, k.as_bytes(), &product)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    // in order to retrieve all products, write keys to with pl
    let list_key = crate::PRODUCT_LIST_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("creating product index");
    }
    let old: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let product_list = [old, String::from(&pid)].join(",");
    let s_product_list = bincode::serialize(&product_list).unwrap_or_default();
    debug!(
        "writing product index {} for id: {}",
        product_list, list_key
    );
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_product_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(new_product)
}

/// Single Product lookup
pub fn find(pid: &String) -> Result<Product, NevekoError> {
    let db = &DATABASE_LOCK;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &pid.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        error!("product not found");
        return Err(NevekoError::Database(MdbError::NotFound));
    }
    let result: Product = bincode::deserialize(&r[..]).unwrap_or_default();
    Ok(result)
}

/// Product lookup for all
pub fn find_all() -> Result<Vec<Product>, NevekoError> {
    let db = &DATABASE_LOCK;
    let i_list_key = crate::PRODUCT_LIST_DB_KEY;
    let i_r = db::DatabaseEnvironment::read(&db.env, &db.handle, &i_list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if i_r.is_empty() {
        error!("product index not found");
    }
    let str_r: String = bincode::deserialize(&i_r[..]).unwrap_or_default();
    let i_v_pid = str_r.split(",");
    let i_v: Vec<String> = i_v_pid.map(String::from).collect();
    let mut products: Vec<Product> = Vec::new();
    for p in i_v {
        let mut product: Product = find(&p).unwrap_or_default();
        if !product.pid.is_empty() {
            // don't return images
            product.image = Vec::new();
            products.push(product);
        }
    }
    Ok(products)
}

/// Modify product
pub fn modify(p: Json<Product>) -> Result<Product, NevekoError> {
    // TODO(c2m): don't allow modification to products with un-delivered orders
    info!("modify product: {}", &p.pid);
    let f_prod: Product = find(&p.pid)?;
    if f_prod.pid.is_empty() {
        error!("product not found");
        return Err(NevekoError::Database(MdbError::NotFound));
    }
    let u_prod = Product::update(f_prod, &p);
    let db = &DATABASE_LOCK;
    db::DatabaseEnvironment::delete(&db.env, &db.handle, u_prod.pid.as_bytes())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let product = bincode::serialize(&u_prod).unwrap_or_default();
    let db = &DATABASE_LOCK;
    db::write_chunks(&db.env, &db.handle, u_prod.pid.as_bytes(), &product)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(u_prod)
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
        .get(format!("http://{}/market/{}", contact, pid))
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
