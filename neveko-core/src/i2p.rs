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

/// Looks for the `tunnels-config.json` at /home/$USER/.i2p-zero/config/
///
/// and attempts to extract the app and http proxy tunnel information.
async fn find_tunnels() {
    let args = args::Args::parse();
    let app_port = utils::get_app_port();
    let file_path = format!(
        "/home/{}/.i2p-zero/config/tunnels.json",
        env::var("USER").unwrap_or(String::from("user"))
    );
    let contents = fs::read_to_string(file_path).unwrap_or(utils::empty_string());
    debug!("i2p tunnels: {}", contents);
    let has_app_tunnel = contents.contains(&format!("{}", app_port));
    let proxy_port = get_i2p_proxy_port();
    let socks_proxy_port = get_i2p_socks_proxy_port();
    let has_http_tunnel = contents.contains(&proxy_port);
    let has_socks_proxy_tunnel = contents.contains(&format!("{}", &socks_proxy_port));
    let has_anon_inbound_tunnel = contents.contains(&format!("{}", args.anon_inbound_port));
    if !has_app_tunnel || !has_http_tunnel || !has_anon_inbound_tunnel || !has_socks_proxy_tunnel {
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
    if !has_anon_inbound_tunnel {
        debug!("creating anon inbound tunnel");
        create_anon_inbound_tunnel();
    }
    if !has_socks_proxy_tunnel {
        debug!("creating socks proxy tunnel");
        create_socks_proxy_tunnel();
    }
}

/// Called on application startup for i2p tunnel creation,
///
/// proxy tunnel, etc. Logs proxy status every 10 minutes.
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
            let tick: std::sync::mpsc::Receiver<()> =
                schedule_recv::periodic_ms(crate::I2P_CONNECTIVITY_CHECK_INTERVAL);
            loop {
                tick.recv().unwrap();
                check_connection().await;
            }
        });
    }
}

/// Create an i2p tunnel for the NEVEKO application
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
        .expect("i2p-zero failed to create a app tunnel");
    debug!("{:?}", output.stdout);
}

/// Create an i2p tunnel for the monero wallet socks proxy
fn create_socks_proxy_tunnel() {
    info!("creating monerod socks proxy tunnel");
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    let output = Command::new(format!("{}/router/bin/tunnel-control.sh", path))
        .args(["socks.create", &format!("{}", get_i2p_socks_proxy_port())])
        .spawn()
        .expect("i2p-zero failed to create a socks proxy tunnel");
    debug!("{:?}", output.stdout);
}

/// Create an i2p tunnel for the monero tx proxy
fn create_anon_inbound_tunnel() {
    info!("creating monerod anon inbound proxy tunnel");
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    let output = Command::new(format!("{}/router/bin/tunnel-control.sh", path))
        .args([
            "server.create",
            "127.0.0.1",
            &format!("{}", args.anon_inbound_port),
        ])
        .spawn()
        .expect("i2p-zero failed to create a anon inbound tunnel");
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

/// Extract i2p socks port from command line arg
fn get_i2p_socks_proxy_port() -> String {
    let proxy_host = utils::get_i2p_wallet_proxy_host();
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

/// This is the `dest` value of the app i2p tunnels
///
/// in `tunnels-config.json`.
///
/// `port` - the port of the tunnel (e.g. `utils::get_app_port()`)
pub fn get_destination(port: Option<u16>) -> String {
    let mut file_path = format!(
        "/home/{}/.i2p-zero/config/tunnels.json",
        env::var("USER").unwrap_or(String::from("user"))
    );
    let args = args::Args::parse();
    let is_advanced_mode =
        std::env::var(crate::NEVEKO_I2P_ADVANCED_MODE).unwrap_or(utils::empty_string());
    if args.i2p_advanced || is_advanced_mode == String::from("1") {
        let advanced_tunnel =
            std::env::var(crate::NEVEKO_I2P_TUNNELS_JSON).unwrap_or(utils::empty_string());
        let manual_tunnel = if advanced_tunnel == utils::empty_string() {
            args.i2p_tunnels_json
        } else {
            advanced_tunnel
        };
        file_path = format!("{}/tunnels.json", manual_tunnel);
    }
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
            if tunnel.port == format!("{}", port.unwrap_or(utils::get_app_port())) {
                destination = tunnel.dest.unwrap_or(utils::empty_string());
            }
        }
        return destination;
    }
    utils::empty_string()
}

/// Ping the i2p-zero http proxy `tunnel-control http.state <port>`
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
        debug!("http proxy is open");
        ProxyStatus::Open
    } else {
        debug!("http proxy is opening");
        ProxyStatus::Opening
    }
}
