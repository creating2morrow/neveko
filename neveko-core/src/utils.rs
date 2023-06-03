use crate::{
    args,
    db,
    gpg,
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
use std::time::Duration;

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
    pub i2p_zero_dir: String,
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
            daemon_host: String::from("http://localhost:38081"),
            i2p_zero_dir: String::from("/home/user/i2p-zero-linux.v1.21"),
            mainnet: false,
            monero_location: String::from("/home/user/monero-x86_64-linux-gnu-v0.18.2.2"),
            rpc_credential: String::from("pass"),
            rpc_username: String::from("user"),
            rpc_host: String::from("http://localhost:38083"),
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
    ];
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
    args.i2p_proxy_host
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

/// app message port
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
        gpg_key: c.gpg_key.iter().cloned().collect(),
    };
    Json(r_contact)
}

/// convert message to json so only core module does the work
pub fn message_to_json(m: &models::Message) -> Json<models::Message> {
    let r_message: models::Message = models::Message {
        body: m.body.iter().cloned().collect(),
        mid: String::from(&m.mid),
        uid: utils::empty_string(),
        created: m.created,
        from: String::from(&m.from),
        to: String::from(&m.to),
    };
    Json(r_message)
}

/// Instead of putting `String::from("")`
pub fn empty_string() -> String {
    String::from("")
}

// DoS prevention
pub const fn string_limit() -> usize {
    512
}
pub const fn gpg_key_limit() -> usize {
    4096
}
pub const fn message_limit() -> usize {
    9999
}

/// Generate application gpg keys at startup if none exist
async fn gen_app_gpg() {
    let mut gpg_key = gpg::find_key().unwrap_or(utils::empty_string());
    if gpg_key == utils::empty_string() {
        info!("no gpg key found for neveko, creating it...");
        // wait for key gen
        gpg::write_gen_batch().unwrap();
        gpg::gen_key();
        tokio::time::sleep(Duration::new(9, 0)).await;
        gpg_key = gpg::find_key().unwrap_or(utils::empty_string());
    }
    debug!("gpg key: {}", gpg_key);
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
    let mut market_path = "neveko-contact/target/debug/neveko_market";
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
    let args = args::Args::parse();
    if args.remote_access {
        start_micro_servers();
    }
    if args.clear_fts {
        clear_fts();
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
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    gen_app_wallet(&wallet_password).await;
    i2p::start().await;
    gen_app_gpg().await;
    let env: String = get_release_env().value();
    start_gui();
    {
        tokio::spawn(async {
            message::retry_fts().await;
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
    let i2pz_output = std::process::Command::new("pkill")
        .arg("i2p-zero")
        .spawn()
        .expect("i2p-zero failed to stop");
    debug!("{:?}", i2pz_output.stdout);
}

/// We can restart fts from since it gets terminated when empty
pub fn restart_retry_fts() {
    tokio::spawn(async move {
        message::retry_fts().await;
    });
}

/// Called on app startup if `--clear-fts` flag is passed.
fn clear_fts() {
    info!("clear fts");
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, "fts");
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
/// clearnet. TODO(c2m): trusted download locations over i2p.
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
/// especially on stagenet.
pub async fn estimate_fee() -> u128 {
    let mut height: u64 = 0;
    let mut count: u64 = 1;
    let mut v_fee: Vec<u128> = Vec::new();
    loop {
        debug!("current height: {}", height);
        if v_fee.len() >= 30 {
            break;
        }
        let r_height = monero::get_height().await;
        if r_height.height == 0 {
            error!("error fetching height");
            return 0;
        }
        height = r_height.height - count;
        let block = monero::get_block(height).await;
        if block.result.block_header.num_txes > 0 {
            debug!("fetching {} txs", block.result.block_header.num_txes);
            let tx_hashes: Option<Vec<String>> = block.result.tx_hashes;
            let transactions = monero::get_transactions(tx_hashes.unwrap()).await;
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
    &v_fee.iter().sum() / v_fee.len() as u128
}

/// Combine the results `estimate_fee()` and `get_balance()` to
///
/// determine whether or not a transfer for a given invoice is possible.
pub async fn can_transfer(invoice: u128) -> bool {
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD)
        .unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let balance = monero::get_balance().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let fee = estimate_fee().await;
    debug!("fee estimated to: {}", fee);
    debug!("balance: {}", balance.result.unlocked_balance);
    debug!("fee + invoice = {}", invoice + fee);
    balance.result.unlocked_balance > (fee + invoice)
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
                    tokio::time::sleep(Duration::from_secs(3600)).await;
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
