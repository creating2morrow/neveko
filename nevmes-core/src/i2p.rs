use crate::{
    args,
    utils,
};
use clap::Parser;
use log::{
    debug,
    error,
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

// TODO(c2m): debug i2p-zero http proxy

#[derive(Debug)]
pub enum I2pStatus {
    Accept,
    Reject,
}

impl I2pStatus {
    pub fn value(&self) -> String {
        match *self {
            I2pStatus::Accept => String::from("Accepting tunnels"),
            I2pStatus::Reject => String::from("Rejecting tunnels: Starting up"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpProxyStatus {
    pub open: bool,
}

#[derive(Debug)]
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
    // let has_http_tunnel = contents.contains("4444");
    if !has_app_tunnel {
        tokio::time::sleep(Duration::new(120, 0)).await;
        create_tunnel();
    }
    // TODO(c2m): why is i2p-zero http proxy always giving "destination not
    // found" error? if  !has_http_tunnel { create_http_proxy(); }
}

pub async fn start() {
    info!("starting i2p-zero");
    let args = args::Args::parse();
    let path = args.i2p_zero_dir;
    let output = Command::new(format!("{}/router/bin/i2p-zero", path))
        .spawn();
    match output {
        Ok(child) => debug!("{:?}", child.stdout),
        _=> {
            warn!("i2p-zero not installed, manual tunnel creation required");
            ()
        },
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

// TODO(c2m): use i2p-zero http proxy
// fn create_http_proxy() {
//     info!("creating http proxy");
//     let output =
// Command::new("./i2p-zero-linux.v1.20/router/bin/tunnel-control.sh")
//         .args(["http.create", "4444"])
//         .spawn()
//         .expect("i2p-zero failed to create a http proxy");
//     debug!("{:?}", output.stdout);
// }

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

pub async fn check_connection() -> HttpProxyStatus {
    let client: reqwest::Client = reqwest::Client::new();
    let host: &str = "http://localhost:7657/tunnels";
    match client.get(host).send().await {
        Ok(response) => {
            // do some parsing here to check the status
            let res = response.text().await;
            match res {
                Ok(res) => {
                    // split the html from the local i2p tunnels page
                    let split1 = res.split("<h4><span class=\"tunnelBuildStatus\">");
                    let mut v1: Vec<String> = split1.map(|s| String::from(s)).collect();
                    let s1 = v1.remove(1);
                    let v2 = s1.split("</span></h4>");
                    let mut split2: Vec<String> = v2.map(|s| String::from(s)).collect();
                    let status: String = split2.remove(0);
                    if status == I2pStatus::Accept.value() {
                        info!("i2p is currently accepting tunnels");
                        return HttpProxyStatus { open: true };
                    } else if status == I2pStatus::Reject.value() {
                        info!("i2p is currently rejecting tunnels");
                        return HttpProxyStatus { open: false };
                    } else {
                        info!("i2p is unstable");
                        return HttpProxyStatus { open: true };
                    }
                }
                _ => {
                    error!("i2p status check failure");
                    return HttpProxyStatus { open: false };
                }
            }
        }
        Err(e) => {
            warn!("i2p-zero http proxy is degraded");
            warn!("please install i2pd from geti2p.org");
            warn!("start i2p with /path-to-i2p/i2prouter start");
            error!("{}", e.to_string());
            return HttpProxyStatus { open: false };
        }
    }
}
