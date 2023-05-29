// Product repo/service layer
use nevmes_core::{db, models::*, utils};
use log::{debug, error, info};
use rocket::serde::json::Json;

/// Create a new product
pub fn create(d: Json<Product>) -> Product {
    let pid: String = format!("product{}", utils::generate_rnd());
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
    debug!("writing product index {} for id: {}", product_list, list_key);
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
        let product: Product = find(&p);
        if product.pid != utils::empty_string() {
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
    let u_prod = Product::update(
        f_prod,
        String::from(&p.description),
        p.image.iter().cloned().collect(),
        p.in_stock,
        String::from(&p.description),
        p.price,
        p.qty,
    );
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, &u_prod.pid);
    db::Interface::write(&s.env, &s.handle, &u_prod.pid, &Product::to_db(&u_prod));
    return u_prod;
}
