use crate::{
    args,
    utils,
};
use clap::Parser;
use log::{
    debug,
    info,
    warn,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    env,
    fs,
    process::Command,
    time::Duration,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpProxyStatus {
    pub open: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ProxyStatus {
    Opening,
    Open,
}

impl ProxyStatus {
    pub fn value(&self) -> String {
        match *self {
            ProxyStatus::Opening => String::from("opening\n"),
            ProxyStatus::Open => String::from("open\n"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Tunnel {
    // http proxy tunnel wont have this field
    dest: Option<String>,
    port: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tunnels {
    tunnels: Vec<Tunnel>,
}

impl Default for Tunnels {
    fn default() -> Self {
        Tunnels {
            tunnels: Vec::new(),
        }
    }
}

async fn find_tunnels() {
    let app_port = utils::get_app_port();
    let file_path = format!(
        "/home/{}/.i2p-zero/config/tunnels.json",
        env::var("USER").unwrap_or(String::from("user"))
    );
    let contents = fs::read_to_string(file_path).unwrap_or(utils::empty_string());
    debug!("i2p tunnels: {}", contents);
    let has_app_tunnel = contents.contains(&format!("{}", app_port));
    let proxy_port = get_i2p_proxy_port();
    let has_http_tunnel = contents.contains(&proxy_port);
    if !has_app_tunnel || !has_http_tunnel {
        tokio::time::sleep(Duration::new(120, 0)).await;
    }
    if !has_app_tunnel {
        debug!("creating app tunnel");
        create_tunnel();
    }
    if !has_http_tunnel {
        debug!("creating http tunnel");
        create_http_proxy();
    }
}

pub async fn start() {
    info!("starting i2p-zero");
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    let output = Command::new(format!("{}/router/bin/i2p-zero", path)).spawn();
    match output {
        Ok(child) => debug!("{:?}", child.stdout),
        _ => {
            warn!("i2p-zero not installed, manual tunnel creation required");
            ()
        }
    }
    find_tunnels().await;
    {
        tokio::spawn(async move {
            let tick: std::sync::mpsc::Receiver<()> = schedule_recv::periodic_ms(600000);
            loop {
                tick.recv().unwrap();
                check_connection().await;
            }
        });
    }
}

fn create_tunnel() {
    info!("creating tunnel");
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    let output = Command::new(format!("{}/router/bin/tunnel-control.sh", path))
        .args([
            "server.create",
            "127.0.0.1",
            &format!("{}", utils::get_app_port()),
        ])
        .spawn()
        .expect("i2p-zero failed to create a tunnel");
    debug!("{:?}", output.stdout);
}

/// Extract i2p port from command line arg
fn get_i2p_proxy_port() -> String {
    let proxy_host = utils::get_i2p_http_proxy();
    let values = proxy_host.split(":");
    let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
    let port = v.remove(2);
    port
}
/// Create the http proxy if it doesn't exist
fn create_http_proxy() {
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    info!("creating http proxy");
    let port = get_i2p_proxy_port();
    let output = Command::new(format!("{}/router/bin/tunnel-control.sh", path))
        .args(["http.create", &port])
        .spawn()
        .expect("i2p-zero failed to create a http proxy");
    debug!("{:?}", output.stdout);
}

pub fn get_destination() -> String {
    let file_path = format!(
        "/home/{}/.i2p-zero/config/tunnels.json",
        env::var("USER").unwrap_or(String::from("user"))
    );
    // Don't panic if i2p-zero isn't installed
    let contents = match fs::read_to_string(file_path) {
        Ok(file) => file,
        _ => utils::empty_string(),
    };
    if contents != utils::empty_string() {
        let input = format!(r#"{contents}"#);
        let j: Tunnels = serde_json::from_str(&input).unwrap_or(Default::default());
        let mut destination: String = utils::empty_string();
        let tunnels: Vec<Tunnel> = j.tunnels;
        for tunnel in tunnels {
            if tunnel.port == format!("{}", utils::get_app_port()) {
                destination = tunnel.dest.unwrap_or(utils::empty_string());
            }
        }
        return destination;
    }
    utils::empty_string()
}

pub async fn check_connection() -> ProxyStatus {
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    let port = get_i2p_proxy_port();
    let output = Command::new(format!("{}/router/bin/tunnel-control.sh", path))
        .args(["http.state", &port])
        .output()
        .expect("check i2p connection failed");
    let str_status = String::from_utf8(output.stdout).unwrap();
    if str_status == ProxyStatus::Open.value() {
        ProxyStatus::Open
    } else {
        ProxyStatus::Opening
    }
}
