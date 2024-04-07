use crate::{
    args,
    contact,
    db,
    dispute,
    i2p,
    message,
    models,
    monero,
    reqres,
    utils,
};
use clap::Parser;
use log::{
    debug,
    error,
    info,
    warn,
};
use rand_core::RngCore;
use rocket::serde::json::Json;
extern crate rpassword;
use rpassword::read_password;
use std::io::Write;

const ESTIMATE_FEE_FAILURE: u128 = 0;

/// Struct for the vendor / contact status window
pub struct ContactStatus {
    /// UNIX timestamp of expiration as string
    pub exp: String,
    /// human readable date of expiration as string
    pub h_exp: String,
    /// i2p address of current status check
    pub i2p: String,
    /// update vendor status of contact
    pub is_vendor: bool,
    /// JSON Web Proof of current status check
    pub jwp: String,
    /// Alias for contact
    pub nick: String,
    /// transaction proof signature of current status check
    pub txp: String,
}

impl Default for ContactStatus {
    fn default() -> Self {
        ContactStatus {
            exp: utils::empty_string(),
            h_exp: utils::empty_string(),
            i2p: utils::empty_string(),
            is_vendor: false,
            jwp: utils::empty_string(),
            nick: String::from("anon"),
            txp: utils::empty_string(),
        }
    }
}

/// Enum for selecting hash validation
#[derive(PartialEq)]
enum ExternalSoftware {
    I2PZero,
    XMR,
}

/// Handles the state for the installation manager popup
pub struct Installations {
    pub xmr: bool,
    pub i2p_zero: bool,
}

impl Default for Installations {
    fn default() -> Self {
        Installations {
            xmr: false,
            i2p_zero: false,
        }
    }
}

/// Handles the state for the connection manager popup
pub struct Connections {
    pub blockchain_dir: String,
    pub daemon_host: String,
    pub i2p_proxy_host: String,
    pub i2p_socks_host: String,
    /// path to manually created tunnels json
    pub i2p_tunnels_json: String,
    pub i2p_zero_dir: String,
    pub is_remote_node: bool,
    pub is_i2p_advanced: bool,
    pub mainnet: bool,
    pub monero_location: String,
    pub rpc_credential: String,
    pub rpc_username: String,
    pub rpc_host: String,
}

impl Default for Connections {
    fn default() -> Self {
        Connections {
            blockchain_dir: String::from("/home/user/.bitmonero"),
            daemon_host: String::from("http://127.0.0.1:38081"),
            i2p_proxy_host: String::from("http://127.0.0.1:4444"),
            i2p_socks_host: String::from("http://127.0.0.1:9051"),
            i2p_tunnels_json: String::from("/home/user/neveko/i2p-manual"),
            i2p_zero_dir: String::from("/home/user/i2p-zero-linux.v1.21"),
            is_remote_node: false,
            is_i2p_advanced: false,
            mainnet: false,
            monero_location: String::from("/home/user/monero-x86_64-linux-gnu-v0.18.3.1"),
            rpc_credential: String::from("pass"),
            rpc_username: String::from("user"),
            rpc_host: String::from("http://127.0.0.1:38083"),
        }
    }
}

#[derive(Debug)]
pub enum ApplicationErrors {
    LoginError,
    UnknownError,
}

impl ApplicationErrors {
    pub fn value(&self) -> String {
        match *self {
            ApplicationErrors::LoginError => String::from("LoginError"),
            ApplicationErrors::UnknownError => String::from("UnknownError"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ReleaseEnvironment {
    Development,
    Production,
}

impl ReleaseEnvironment {
    pub fn value(&self) -> String {
        match *self {
            ReleaseEnvironment::Development => String::from("development"),
            ReleaseEnvironment::Production => String::from("production"),
        }
    }
}

/// start core module from gui
pub fn start_core(conn: &Connections) {
    let env = if !conn.mainnet { "dev" } else { "prod" };
    let remote_node = if !conn.is_remote_node {
        "--full-node"
    } else {
        "--remote-node"
    };
    let i2p_advanced = if !conn.is_i2p_advanced {
        "--i2p-normal"
    } else {
        "--i2p-advanced"
    };
    let args = [
        "--monero-location",
        &conn.monero_location,
        "--monero-blockchain-dir",
        &conn.blockchain_dir,
        "--monero-rpc-host",
        &conn.rpc_host,
        "--monero-rpc-daemon",
        &conn.daemon_host,
        "--monero-rpc-username",
        &conn.rpc_username,
        "--monero-rpc-cred",
        &conn.rpc_credential,
        "--i2p-zero-dir",
        &conn.i2p_zero_dir,
        "-r",
        env,
        remote_node,
        i2p_advanced,
        "--i2p-tunnels-json",
        &conn.i2p_tunnels_json,
        "--i2p-proxy-host",
        &conn.i2p_proxy_host,
        "--i2p-socks-proxy-host",
        &conn.i2p_socks_host,
    ];
    if conn.is_i2p_advanced {
        // set the i2p proxy host for advanced user re-use
        std::env::set_var(crate::NEVEKO_I2P_PROXY_HOST, &conn.i2p_proxy_host.clone());
        std::env::set_var(
            crate::NEVEKO_I2P_TUNNELS_JSON,
            &conn.i2p_tunnels_json.clone(),
        );
        std::env::set_var(crate::NEVEKO_I2P_ADVANCED_MODE, String::from("1"));
    }
    if conn.is_remote_node {
        std::env::set_var(crate::MONERO_DAEMON_HOST, &conn.daemon_host.clone());
        std::env::set_var(crate::MONERO_WALLET_RPC_HOST, &conn.rpc_host.clone());
        std::env::set_var(crate::GUI_REMOTE_NODE, crate::GUI_SET_REMOTE_NODE)
    }
    let output = std::process::Command::new("./neveko")
        .args(args)
        .spawn()
        .expect("core module failed to start");
    debug!("{:?}", output.stdout);
}

/// Using remote node?
pub fn is_using_remote_node() -> bool {
    let args = args::Args::parse();
    let r = args.remote_node;
    if r {
        warn!("using a remote node may harm privacy");
    }
    r
}

/// Random data generation for auth / primary keys
pub fn generate_rnd() -> String {
    let mut data = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut data);
    hex::encode(data)
}

/// Helper for separation of dev and prod concerns
pub fn get_release_env() -> ReleaseEnvironment {
    let args = args::Args::parse();
    let env = String::from(args.release_env);
    if env == "prod" {
        return ReleaseEnvironment::Production;
    } else {
        return ReleaseEnvironment::Development;
    }
}

/// app port
pub fn get_app_port() -> u16 {
    let args = args::Args::parse();
    args.port
}

/// i2p http proxy
pub fn get_i2p_http_proxy() -> String {
    let args = args::Args::parse();
    let advanced_proxy = std::env::var(crate::NEVEKO_I2P_PROXY_HOST).unwrap_or(empty_string());
    if advanced_proxy == empty_string() {
        args.i2p_proxy_host
    } else {
        advanced_proxy
    }
}

/// wallet proxy host
pub fn get_i2p_wallet_proxy_host() -> String {
    let args = args::Args::parse();
    args.i2p_socks_proxy_host
}

/// app auth port
pub fn get_app_auth_port() -> u16 {
    let args = args::Args::parse();
    args.auth_port
}

/// app contact port
pub fn get_app_contact_port() -> u16 {
    let args = args::Args::parse();
    args.contact_port
}

/// app market port
pub fn get_app_market_port() -> u16 {
    let args = args::Args::parse();
    args.marketplace_port
}

/// app message port
pub fn get_app_message_port() -> u16 {
    let args = args::Args::parse();
    args.message_port
}

/// jwp confirmation limit
pub fn get_conf_threshold() -> u64 {
    let args = args::Args::parse();
    args.confirmation_threshold
}

/// jwp confirmation limit
pub fn get_payment_threshold() -> u128 {
    let args = args::Args::parse();
    args.payment_threshold
}

/// convert contact to json so only core module does the work
pub fn contact_to_json(c: &models::Contact) -> Json<models::Contact> {
    let r_contact: models::Contact = models::Contact {
        cid: String::from(&c.cid),
        i2p_address: String::from(&c.i2p_address),
        is_vendor: c.is_vendor,
        xmr_address: String::from(&c.xmr_address),
        nmpk: String::from(&c.nmpk),
    };
    Json(r_contact)
}

/// convert message to json so only core module does the work
pub fn message_to_json(m: &models::Message) -> Json<models::Message> {
    let r_message: models::Message = models::Message {
        body: String::from(&m.body),
        mid: String::from(&m.mid),
        uid: utils::empty_string(),
        created: m.created,
        from: String::from(&m.from),
        to: String::from(&m.to),
    };
    Json(r_message)
}

/// convert product to json so only core module does the work
pub fn product_to_json(m: &models::Product) -> Json<models::Product> {
    let r_product: models::Product = models::Product {
        pid: String::from(&m.pid),
        description: String::from(&m.description),
        image: m.image.iter().cloned().collect(),
        in_stock: m.in_stock,
        name: String::from(&m.name),
        price: m.price,
        qty: m.qty,
    };
    Json(r_product)
}

pub fn order_to_json(o: &reqres::OrderRequest) -> Json<reqres::OrderRequest> {
    let r_order: reqres::OrderRequest = reqres::OrderRequest {
        cid: String::from(&o.cid),
        adjudicator: String::from(&o.adjudicator),
        pid: String::from(&o.pid),
        ship_address: String::from(&o.ship_address),
        quantity: o.quantity,
    };
    Json(r_order)
}

pub fn dispute_to_json(d: &models::Dispute) -> Json<models::Dispute> {
    let dispute: models::Dispute = models::Dispute {
        created: d.created,
        did: String::from(&d.did),
        orid: String::from(&d.orid),
        tx_set: String::from(&d.tx_set),
    };
    Json(dispute)
}

/// Instead of putting `String::from("")`
pub fn empty_string() -> String {
    String::from("")
}

// DoS prevention
pub const fn string_limit() -> usize {
    512
}

pub const fn npmk_limit() -> usize {
    128
}

pub const fn message_limit() -> usize {
    9999
}

pub const fn image_limit() -> usize {
    9999
}

/// Handles panic! for missing wallet directory
fn create_wallet_dir() {
    let file_path = format!(
        "/home/{}/.neveko",
        std::env::var("USER").unwrap_or(String::from("user"))
    );
    let s_output = std::process::Command::new("mkdir")
        .args(["-p", &format!("{}/stagenet/wallet", file_path)])
        .spawn()
        .expect("failed to create dir");
    debug!("{:?}", s_output);
    let m_output = std::process::Command::new("mkdir")
        .args(["-p", &format!("{}/wallet", file_path)])
        .spawn()
        .expect("failed to create dir");
    debug!("{:?}", m_output);
}

/// Generate application wallet at startup if none exist
async fn gen_app_wallet(password: &String) {
    info!("fetching application wallet");
    let filename = String::from(crate::APP_NAME);
    let mut m_wallet = monero::open_wallet(&filename, password).await;
    if !m_wallet {
        m_wallet = monero::create_wallet(&filename, password).await;
        if !m_wallet {
            error!("failed to create wallet")
        } else {
            m_wallet = monero::open_wallet(&filename, password).await;
            if m_wallet {
                let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
                info!("app wallet address: {}", m_address.result.address)
            }
        }
    }
    monero::close_wallet(&filename, password).await;
}

/// Secret keys for signing internal/external auth tokens
fn gen_signing_keys() {
    info!("generating signing keys");
    let jwp = get_jwp_secret_key();
    let jwt = get_jwt_secret_key();
    // send to db
    let s = db::Interface::open();
    if jwp == utils::empty_string() {
        let rnd_jwp = generate_rnd();
        db::Interface::write(&s.env, &s.handle, crate::NEVEKO_JWP_SECRET_KEY, &rnd_jwp);
    }
    if jwt == utils::empty_string() {
        let rnd_jwt = generate_rnd();
        db::Interface::write(&s.env, &s.handle, crate::NEVEKO_JWT_SECRET_KEY, &rnd_jwt);
    }
}

/// TODO(c2m): add a button to gui to call this
///
/// dont' forget to generate new keys as well
pub fn revoke_signing_keys() {
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, crate::NEVEKO_JWT_SECRET_KEY);
    db::Interface::delete(&s.env, &s.handle, crate::NEVEKO_JWP_SECRET_KEY);
}

pub fn get_jwt_secret_key() -> String {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, crate::NEVEKO_JWT_SECRET_KEY);
    if r == utils::empty_string() {
        error!("JWT key not found");
        return Default::default();
    }
    r
}

pub fn get_jwp_secret_key() -> String {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, crate::NEVEKO_JWP_SECRET_KEY);
    if r == utils::empty_string() {
        error!("JWP key not found");
        return Default::default();
    }
    r
}

/// Start the remote access microservers `--remote-access` flag
fn start_micro_servers() {
    info!("starting auth server");
    let mut auth_path = "neveko-auth/target/debug/neveko_auth";
    let env = get_release_env();
    if env == ReleaseEnvironment::Production {
        auth_path = "neveko_auth";
    }
    let a_output = std::process::Command::new(auth_path)
        .spawn()
        .expect("failed to start auth server");
    debug!("{:?}", a_output.stdout);
    info!("starting contact server");
    let mut contact_path = "neveko-contact/target/debug/neveko_contact";
    if env == ReleaseEnvironment::Production {
        contact_path = "neveko_contact";
    }
    let c_output = std::process::Command::new(contact_path)
        .spawn()
        .expect("failed to start contact server");
    debug!("{:?}", c_output.stdout);
    info!("starting marketplace admin server");
    let mut market_path = "neveko-market/target/debug/neveko_market";
    if env == ReleaseEnvironment::Production {
        market_path = "neveko_market";
    }
    let market_output = std::process::Command::new(market_path)
        .spawn()
        .expect("failed to start marketplace server");
    debug!("{:?}", market_output.stdout);
    info!("starting message server");
    let mut message_path = "neveko-message/target/debug/neveko_message";
    if env == ReleaseEnvironment::Production {
        message_path = "neveko_message";
    }
    let m_output = std::process::Command::new(message_path)
        .spawn()
        .expect("failed to start message server");
    debug!("{:?}", m_output.stdout);
}

/// open gui from neveko core launch
fn start_gui() {
    let args = args::Args::parse();
    if args.gui {
        info!("starting gui");
        let mut gui_path = "neveko-gui/target/debug/neveko_gui";
        let env = get_release_env();
        if env == ReleaseEnvironment::Production {
            gui_path = "neveko-gui";
        }
        let g_output = std::process::Command::new(gui_path)
            .spawn()
            .expect("failed to start gui");
        debug!("{:?}", g_output.stdout);
    }
}

/// Put all app pre-checks here
pub async fn start_up() {
    info!("neveko is starting up");
    warn!("monero multisig is experimental and usage of neveko may lead to loss of funds");
    let args = args::Args::parse();
    if args.remote_access {
        start_micro_servers();
    }
    if args.clear_fts {
        clear_fts();
    }
    if args.clear_disputes {
        clear_disputes();
    }
    gen_signing_keys();
    if !is_using_remote_node() {
        monero::start_daemon();
    }
    create_wallet_dir();
    // wait for daemon for a bit
    tokio::time::sleep(std::time::Duration::new(5, 0)).await;
    monero::start_rpc();
    // wait for rpc server for a bit
    tokio::time::sleep(std::time::Duration::new(5, 0)).await;
    monero::check_rpc_connection().await;
    let mut wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(empty_string());
    if wallet_password == empty_string() {
        print!(
            "MONERO_WALLET_PASSWORD not set, enter neveko wallet password for monero-wallet-rpc: "
        );
        std::io::stdout().flush().unwrap();
        wallet_password = read_password().unwrap();
        std::env::set_var(crate::MONERO_WALLET_PASSWORD, &wallet_password);
    }
    let env: String = get_release_env().value();
    if !args.i2p_advanced {
        i2p::start().await;
    }
    gen_app_wallet(&wallet_password).await;
    start_gui();
    // start async background tasks here
    {
        tokio::spawn(async {
            message::retry_fts().await;
            dispute::settle_dispute().await;
        });
    }
    info!("{} - neveko is online", env);
}

/// Called by gui for cleaning up monerod, rpc, etc.
///
/// pass true from gui connection manager so not to kill neveko
pub fn kill_child_processes(cm: bool) {
    info!("stopping child processes");
    // TODO(c2m): prompt on gui letting user determine what background
    //            services to keep running
    if cm {
        let xmrd_output = std::process::Command::new("pkill")
            .arg("monerod")
            .spawn()
            .expect("monerod failed to stop");
        debug!("{:?}", xmrd_output.stdout);
    }
    if !cm {
        let neveko_output = std::process::Command::new("pkill")
            .arg("neveko")
            .spawn()
            .expect("neveko failed to stop");
        debug!("{:?}", neveko_output.stdout);
    }
    let rpc_output = std::process::Command::new("killall")
        .arg("monero-wallet-rpc")
        .spawn()
        .expect("monero-wallet-rpc failed to stop");
    debug!("{:?}", rpc_output.stdout);
}

/// We can restart fts from since it gets terminated when empty
pub fn restart_retry_fts() {
    tokio::spawn(async move {
        message::retry_fts().await;
    });
}

/// We can restart dispute auto-settle from since it gets terminated when empty
pub fn restart_dispute_auto_settle() {
    tokio::spawn(async move {
        dispute::settle_dispute().await;
    });
}

/// Called on app startup if `--clear-fts` flag is passed.
fn clear_fts() {
    info!("clear fts");
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, crate::FTS_DB_KEY);
}

/// Called on app startup if `--clear-dispute` flag is passed.
fn clear_disputes() {
    info!("clear_disputes");
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, crate::DISPUTE_LIST_DB_KEY);
}

/// Destroy temp files
pub fn stage_cleanup(f: String) {
    info!("staging {} for cleanup", &f);
    let output = std::process::Command::new("bash")
        .args(["-c", &format!("rm {}", &f)])
        .spawn()
        .expect("cleanup staging failed");
    debug!("{:?}", output.stdout);
}

/// Handle the request from user to additional software
///
/// from gui startup. Power users will most like install
///
/// software on their own. Note that software pull is over
///
/// clearnet. TODO(c2m): trusted download locations over tor.
pub async fn install_software(installations: Installations) -> bool {
    let mut valid_i2p_zero_hash = true;
    let mut valid_xmr_hash = true;
    if installations.i2p_zero {
        info!("installing i2p-zero");
        let i2p_version = crate::I2P_ZERO_RELEASE_VERSION;
        let i2p_zero_zip = format!("i2p-zero-linux.{}.zip", i2p_version);
        let link = format!(
            "https://github.com/creating2morrow/i2p-zero/releases/download/{}-neveko/{}",
            i2p_version, i2p_zero_zip
        );
        let curl = std::process::Command::new("curl")
            .args(["-LO#", &link])
            .status();
        match curl {
            Ok(curl_output) => {
                debug!("{:?}", curl_output);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let unzip_output = std::process::Command::new("unzip")
                    .arg(&i2p_zero_zip)
                    .spawn()
                    .expect("i2p unzip failed");
                debug!("{:?}", unzip_output.stdout);
            }
            _ => error!("i2p-zero download failed"),
        }
        valid_i2p_zero_hash = validate_installation_hash(ExternalSoftware::I2PZero, &i2p_zero_zip);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    if installations.xmr {
        info!("installing monero");
        let link = format!(
            "https://downloads.getmonero.org/cli/{}",
            crate::MONERO_RELEASE_VERSION
        );
        let curl = std::process::Command::new("curl")
            .args(["-O#", &link])
            .status();
        match curl {
            Ok(curl_output) => {
                debug!("{:?}", curl_output);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let tar_output = std::process::Command::new("tar")
                    .args(["-xvf", crate::MONERO_RELEASE_VERSION])
                    .spawn()
                    .expect("monero tar extraction failed");
                debug!("{:?}", tar_output.stdout);
            }
            _ => error!("monero download failed"),
        }
        valid_xmr_hash = validate_installation_hash(
            ExternalSoftware::XMR,
            &String::from(crate::MONERO_RELEASE_VERSION),
        );
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    valid_i2p_zero_hash && valid_xmr_hash
}

/// Linux specific hash validation using the command `sha256sum`
fn validate_installation_hash(sw: ExternalSoftware, filename: &String) -> bool {
    debug!("validating hash");
    let expected_hash = if sw == ExternalSoftware::I2PZero {
        String::from(crate::I2P_ZERO_RELEASH_HASH)
    } else {
        String::from(crate::MONERO_RELEASE_HASH)
    };
    let sha_output = std::process::Command::new("sha256sum")
        .arg(filename)
        .output()
        .expect("hash validation failed");
    let str_sha = String::from_utf8(sha_output.stdout).unwrap();
    let split1 = str_sha.split(" ");
    let mut v: Vec<String> = split1.map(|s| String::from(s)).collect();
    let actual_hash = v.remove(0);
    debug!("actual hash: {}", actual_hash);
    debug!("expected hash: {}", expected_hash);
    actual_hash == expected_hash
}

/// ### The highly ineffecient fee estimator.
///
/// Get the current height. Start fetching blocks
///
/// and checking the number of transactions. If
///
/// there were non-coinbase transactions in the block
///
/// extract the `txnFee` from the `as_json` field.
///
/// Once we have accumulated n>=30 fees paid return the
///
/// average fee paid from the most recent 30 transactions.
///
/// Note, it may take more than one block to do this,
///
/// especially on stagenet. Over i2p let's cheat and just FFE
///
/// (find first fee).
pub async fn estimate_fee() -> u128 {
    // loop intializer
    let mut height: u64 = 0;
    let mut count: u64 = 1;
    let mut v_fee: Vec<u128> = Vec::new();
    let mut r_height: reqres::XmrDaemonGetHeightResponse = Default::default();
    let remote_var = std::env::var(crate::GUI_REMOTE_NODE).unwrap_or(utils::empty_string());
    let remote_set = remote_var == String::from(crate::GUI_SET_REMOTE_NODE);
    if remote_set {
        let p_height = monero::p_get_height().await;
        r_height = p_height.unwrap_or(r_height);
    } else {
        r_height = monero::get_height().await;
    }
    if r_height.height == ESTIMATE_FEE_FAILURE as u64 {
        error!("error fetching height");
        return ESTIMATE_FEE_FAILURE;
    }
    loop {
        debug!("current height: {}", height);
        if v_fee.len() >= 30 {
            break;
        }
        // TODO(?): determine a more effecient fix than this for slow fee estimation
        // over i2p
        if v_fee.len() >= 1 && remote_set {
            break;
        }
        height = r_height.height - count;
        let mut block: reqres::XmrDaemonGetBlockResponse = Default::default();
        if remote_var == String::from(crate::GUI_SET_REMOTE_NODE) {
            let p_block = monero::p_get_block(height).await;
            block = p_block.unwrap_or(block);
        } else {
            block = monero::get_block(height).await;
        }
        if block.result.block_header.num_txes > 0 {
            let tx_hashes: Option<Vec<String>> = block.result.tx_hashes;
            let mut transactions: reqres::XmrDaemonGetTransactionsResponse = Default::default();
            if remote_set {
                let p_txs = monero::p_get_transactions(tx_hashes.unwrap()).await;
                transactions = p_txs.unwrap_or(transactions);
            } else {
                transactions = monero::get_transactions(tx_hashes.unwrap()).await;
            }
            for tx in transactions.txs_as_json {
                let pre_fee_split = tx.split("txnFee\":");
                let mut v1: Vec<String> = pre_fee_split.map(|s| String::from(s)).collect();
                let fee_split = v1.remove(1);
                let post_fee_split = fee_split.split(",");
                let mut v2: Vec<String> = post_fee_split.map(|s| String::from(s)).collect();
                let fee: u128 = match v2.remove(0).trim().parse::<u128>() {
                    Ok(n) => n,
                    Err(_e) => 0,
                };
                v_fee.push(fee);
            }
        }
        count += 1;
    }
    &v_fee.iter().sum::<u128>() / v_fee.len() as u128
}

/// Combine the results `estimate_fee()` and `get_balance()` to
///
/// determine whether or not a transfer for a given invoice is possible.
pub async fn can_transfer(invoice: u128) -> bool {
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let balance = monero::get_balance().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let fee = estimate_fee().await;
    if fee == ESTIMATE_FEE_FAILURE {
        return false;
    }
    debug!("fee estimated to: {}", fee);
    debug!("balance: {}", balance.result.unlocked_balance);
    debug!("fee + invoice = {}", invoice + fee);
    balance.result.unlocked_balance > (fee + invoice)
}

/// Gui toggle for vendor mode
pub fn toggle_vendor_enabled() -> bool {
    // TODO(c2m): Dont toggle vendors with orders status != Delivered
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, contact::NEVEKO_VENDOR_ENABLED);
    if r != contact::NEVEKO_VENDOR_MODE_ON {
        info!("neveko vendor mode enabled");
        db::Interface::write(
            &s.env,
            &s.handle,
            contact::NEVEKO_VENDOR_ENABLED,
            contact::NEVEKO_VENDOR_MODE_ON,
        );
        return true;
    } else {
        info!("neveko vendor mode disabled");
        db::Interface::write(
            &s.env,
            &s.handle,
            contact::NEVEKO_VENDOR_ENABLED,
            contact::NEVEKO_VENDOR_MODE_OFF,
        );
        return false;
    }
}

pub fn search_gui_db(f: String, data: String) -> String {
    let s = db::Interface::open();
    let k = format!("{}-{}", f, data);
    db::Interface::read(&s.env, &s.handle, &k)
}

pub fn write_gui_db(f: String, key: String, data: String) {
    let s = db::Interface::open();
    let k = format!("{}-{}", f, key);
    db::Interface::write(&s.env, &s.handle, &k, &data);
}

pub fn clear_gui_db(f: String, key: String) {
    let s = db::Interface::open();
    let k = format!("{}-{}", f, key);
    db::Interface::delete(&s.env, &s.handle, &k);
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_using_remote_node_test() {
        let expected = false;
        let actual = is_using_remote_node();
        assert_eq!(expected, actual);
    }

    #[test]
    fn generate_rnd_test() {
        let rnd = generate_rnd();
        let actual = rnd.len();
        let expected = 64;
        assert_eq!(expected, actual);
    }

    #[test]
    fn release_env_test() {
        let actual = get_release_env();
        let expected = ReleaseEnvironment::Development;
        assert_eq!(expected, actual)
    }

    #[test]
    fn app_port_test() {
        let actual: u16 = get_app_port();
        let expected: u16 = 9000;
        assert_eq!(expected, actual)
    }

    #[test]
    fn auth_port_test() {
        let actual: u16 = get_app_auth_port();
        let expected: u16 = 9043;
        assert_eq!(expected, actual)
    }

    #[test]
    fn contact_port_test() {
        let actual: u16 = get_app_contact_port();
        let expected: u16 = 9044;
        assert_eq!(expected, actual)
    }

    #[test]
    fn payment_threshold_test() {
        let actual: u128 = get_payment_threshold();
        let expected: u128 = 1;
        assert_eq!(expected, actual)
    }

    #[test]
    fn confirmation_threshold_test() {
        let actual: u64 = get_conf_threshold();
        let expected: u64 = 720;
        assert_eq!(expected, actual)
    }

    #[test]
    fn message_port_test() {
        let actual: u16 = get_app_message_port();
        let expected: u16 = 9045;
        assert_eq!(expected, actual)
    }

    #[test]
    fn contact_to_json_test() {
        let contact = models::Contact {
            cid: String::from("testid"),
            ..Default::default()
        };
        let actual = contact_to_json(&contact);
        let expected = &contact;
        assert_eq!(expected.cid, actual.cid)
    }

    #[test]
    fn message_to_json_test() {
        let message = models::Message {
            mid: String::from("testid"),
            ..Default::default()
        };
        let actual = message_to_json(&message);
        let expected = &message;
        assert_eq!(expected.mid, actual.mid)
    }

    #[test]
    fn can_transfer_test() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        std::thread::spawn(move || {
            rt.block_on(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                }
            })
        });
        tokio::spawn(async move {
            let actual = can_transfer(1).await;
            let expected = false;
            assert_eq!(expected, actual)
        });
    }
}
