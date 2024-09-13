//! embedded i2p module

use std::{thread,fs::File, io::{self, BufRead}, path::Path};
use j4i2prs::router_wrapper as rw;
use j4i2prs::tunnel_control as tc;
use kn0sys_lmdb_rs::MdbError;
use log::*;
use serde::{
    Deserialize,
    Serialize,
};
use std::sync::mpsc::{
    Receiver,
    Sender,
};
use crate::{
    db::{self, DATABASE_LOCK}, error::NevekoError, monero::get_anon_inbound_port, utils, DEFAULT_APP_PORT, DEFAULT_SOCKS_PORT
};

struct Listener {
    is_running: bool,
    run_tx: Sender<bool>,
    run_rx: Receiver<bool>,
}

impl Default for Listener {
    fn default() -> Self {
        let is_running = false;
        let (run_tx, run_rx) = std::sync::mpsc::channel();
        Listener {
            is_running,
            run_tx,
            run_rx,
        }
    }
}

/// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

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

/// Extract i2p port from command line arg
fn get_i2p_proxy_port() -> String {
    let proxy_host = utils::get_i2p_http_proxy();
    let values = proxy_host.split(":");
    let mut v: Vec<String> = values.map(String::from).collect();
    v.remove(2)
}

/// Extract i2p socks port from command line arg
fn get_i2p_socks_proxy_port() -> String {
    let proxy_host = utils::get_i2p_wallet_proxy_host();
    let values = proxy_host.split(":");
    let mut v: Vec<String> = values.map(String::from).collect();
    v.remove(2)
}

/// This is the `dest` value of the app i2p tunnels
///
/// `st` - ServerTunnelType (App or AnonInbound)
pub fn get_destination(st: ServerTunnelType) -> Result<String, NevekoError> {
    let db = &DATABASE_LOCK;
    let r_anon_b32_dest = db::DatabaseEnvironment::read(
        &db.env,
        &db.handle,
        &crate::APP_ANON_IN_B32_DEST.as_bytes().to_vec(),
    ).map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let r_app_b32_dest = db::DatabaseEnvironment::read(
        &db.env,
        &db.handle,
        &crate::APP_I2P_SK.as_bytes().to_vec(),
    ).map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let anon_b32_dest: String = bincode::deserialize(&r_anon_b32_dest[..]).unwrap_or_default();
    let app_b32_dest: String = bincode::deserialize(&&r_app_b32_dest[..]).unwrap_or_default();
    match st {
        ServerTunnelType::App => Ok(app_b32_dest),
        ServerTunnelType::AnonInbound => Ok(anon_b32_dest)
    }
}

/// Ping our base 32 destination address over the http proxy
pub async fn check_connection() -> Result<ProxyStatus, NevekoError> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host).map_err(|_| NevekoError::I2P)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    let b32_dest = get_destination(ServerTunnelType::App)?;
    match client.map_err(|_| NevekoError::I2P)?
        .get(format!("http://{}/status", b32_dest))
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<HttpProxyStatus>().await;
            debug!("check_connection response: {:?}", res);
            return match res {
                Ok(r) => if r.open { Ok(ProxyStatus::Open) } else { Ok(ProxyStatus::Opening) },
                _ => Err(NevekoError::I2P),
            }
        }
        Err(e) => {
            error!("failed to generate invoice due to: {:?}", e);
            return Err(NevekoError::I2P);
        }
    }
}

#[derive(PartialEq)]
pub enum ServerTunnelType {
    App,
    AnonInbound,
}

/// Create app and anon inbound server tunnels if they don't exist yet
fn create_server_tunnel(st: ServerTunnelType) -> Result<tc::Tunnel, NevekoError> {
    let port: u16 = if st == ServerTunnelType::App {
        utils::get_app_port()
    } else {
        get_anon_inbound_port()
    };
    let b32_key = if st == ServerTunnelType::App {
        crate::APP_B32_DEST.as_bytes()
    } else {
        crate::APP_ANON_IN_B32_DEST.as_bytes()
    };
    let sk_key = if st == ServerTunnelType::App {
        crate::APP_I2P_SK.as_bytes()
    } else {
        crate::APP_ANON_IN_SK.as_bytes()
    };
    let db = &DATABASE_LOCK;
    let tunnel: tc::Tunnel = tc::Tunnel::new(
        "127.0.0.1".to_string(),
        port,
        tc::TunnelType::Server
    ).unwrap_or_default();
    let b32_dest: String = tunnel.get_destination();
    log::debug!("destination: {}", &b32_dest);
    let v_b32_dest = bincode::serialize(&b32_dest).unwrap_or_default();
    let v_sk = bincode::serialize(&tunnel.get_sk()).unwrap_or_default();
    db::write_chunks(
        &db.env,
        &db.handle,
        b32_key,
        &v_b32_dest,
    ).map_err(|_| NevekoError::Database(MdbError::Panic))?;
    db::write_chunks(
        &db.env,
        &db.handle,
        sk_key,
        &v_sk,
    ).map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(tunnel)
}

/// Start router and automatic i2p tunnel creation
/// 
/// We'll check for an existing i2p secret key. If it doesn't
/// 
/// exist create a new one.
pub fn start() -> Result<(), NevekoError> {
    let http_proxy_port: u16 = get_i2p_proxy_port().parse::<u16>()
        .unwrap_or(DEFAULT_APP_PORT);
    let socks_port: u16 = get_i2p_socks_proxy_port().parse::<u16>()
        .unwrap_or(DEFAULT_SOCKS_PORT);
    // check for existing app and anon inbound server tunnels
    let db = &DATABASE_LOCK;
    let r_anon_in_sk = db::DatabaseEnvironment::read(
        &db.env,
        &db.handle,
        &crate::APP_ANON_IN_SK.as_bytes().to_vec(),
    ).map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let r_app_sk = db::DatabaseEnvironment::read(
        &db.env,
        &db.handle,
        &crate::APP_I2P_SK.as_bytes().to_vec(),
    ).map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let anon_in_sk: String = bincode::deserialize(&r_anon_in_sk[..]).unwrap_or_default();
    let app_sk: String = bincode::deserialize(&r_app_sk[..]).unwrap_or_default();
    log::info!("starting j4i2prs...");
    let r = rw::Wrapper::create_router().map_err(|_| NevekoError::I2P)?;
    let mut l: Listener = Default::default();
    let run_tx = l.run_tx.clone();
    let _ = thread::spawn(move || {
        log::info!("run thread started");
        run_tx.send(true).unwrap_or_else(|_| log::error!("failed to run router"));
    });
    // run the main thread forever unless we get a router shutdown signal
    let _ = thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(10));
        loop {
            if let Ok(run) = l.run_rx.try_recv() {
                if run {
                    log::info!("starting router");
                    r.invoke_router(rw::METHOD_RUN).unwrap_or_else(|_| log::error!("failed to run router"));
                }
            }
            if !l.is_running {
                let is_router_on = r.is_running().unwrap_or_default();
                if !is_router_on {
                    log::info!("router is warming up, please wait...");
                }
                std::thread::sleep(std::time::Duration::from_secs(60));
                if is_router_on {
                    // check router config
                    if let Ok(lines) = read_lines("./router.config") {
                        for line in lines.map_while(Result::ok) {
                            if line.contains("i2np.udp.port") {
                                let port = line.split("=").collect::<Vec<&str>>()[1];
                                log::info!("router is running on external port = {}", port);
                                log::info!("open this port for better connectivity");
                                log::info!("this port was randomly assigned, keep it private");
                                l.is_running = true;
                                // start the http proxy
                                let http_proxy: tc::Tunnel = tc::Tunnel::new(
                                    "127.0.0.1".to_string(),
                                    http_proxy_port,
                                    tc::TunnelType::Http
                                ).unwrap_or_default();
                                let _ = http_proxy.start(None);
                                // start the socks proxy
                                let socks_proxy: tc::Tunnel = tc::Tunnel::new(
                                    "127.0.0.1".to_string(),
                                    socks_port,
                                    tc::TunnelType::Socks
                                ).unwrap_or_default();
                                let _ = socks_proxy.start(None);
                                log::info!("http proxy on port {}", http_proxy.get_port());
                                log::info!("socks proxy on port {}", socks_proxy.get_port());
                                if app_sk.is_empty() {
                                    let t = create_server_tunnel(ServerTunnelType::App)
                                        .unwrap_or_default();
                                    let _ = t.start(None);       
                                } else {
                                    let app_tunnel = tc::Tunnel::new(
                                        "127.0.0.1".to_string(),
                                        utils::get_app_port(),
                                        tc::TunnelType::ExistingServer
                                    ).unwrap_or_default();
                                    let _ = app_tunnel.start(Some(String::from(&app_sk)));
                                }
                                if anon_in_sk.is_empty() {
                                    let t = create_server_tunnel(ServerTunnelType::AnonInbound)
                                        .unwrap_or_default();
                                    let _ = t.start(None);
                                } else {
                                    let anon_tunnel = tc::Tunnel::new(
                                        "127.0.0.1".to_string(),
                                        get_anon_inbound_port(),
                                        tc::TunnelType::ExistingServer
                                    ).unwrap_or_default();
                                    let _ = anon_tunnel.start(Some(String::from(&anon_in_sk)));
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    Ok(())
}