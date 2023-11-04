use crate::{
    args,
    i2p,
    proof,
    reqres,
    utils,
};
use clap::Parser;
use diqwest::WithDigestAuth;
use log::{
    debug,
    error,
    info,
    warn,
};
use std::{
    io::Write,
    error::Error,
    process::{Command, Stdio}
};

use lazy_static::lazy_static;
use std::sync::Mutex;

pub const INVALID_VERSION: u32 = 0;

// global variable
lazy_static! {
    /// used to avoid multisig wallet collision
    static ref IS_WALLET_BUSY: Mutex<bool> = Mutex::new(false);
}

/// Current xmr ring size updated here.
const RING_SIZE: u32 = 0x10;

struct RpcLogin {
    username: String,
    credential: String,
}

pub enum TransactionType {
    Failed,
    In,
    Out,
    Pending,
    Pool,
}

impl TransactionType {
    pub fn value(&self) -> String {
        match *self {
            Self::Failed => String::from("failed"),
            Self::In => String::from("in"),
            Self::Out => String::from("out"),
            Self::Pending => String::from("pending"),
            Self::Pool => String::from("pool"),
        }
    }
    pub fn propagated(tx_type: String) -> bool {
        tx_type == Self::In.value() || tx_type == Self::Pool.value()
    }
}

enum RpcFields {
    Address,
    Balance,
    CheckTxProof,
    Close,
    CreateAddress,
    CreateWallet,
    DescribeTransfer,
    ExchangeMultisigKeys,
    Export,
    GetTxProof,
    GetTxById,
    GetVersion,
    Id,
    Import,
    JsonRpcVersion,
    Make,
    Open,
    Prepare,
    Refresh,
    SignMultisig,
    SubmitMultisig,
    SweepAll,
    Transfer,
    ValidateAddress,
    Verify,
}

impl RpcFields {
    pub fn value(&self) -> String {
        match *self {
            RpcFields::Address => String::from("get_address"),
            RpcFields::Balance => String::from("get_balance"),
            RpcFields::CheckTxProof => String::from("check_tx_proof"),
            RpcFields::Close => String::from("close_wallet"),
            RpcFields::CreateAddress => String::from("create_address"),
            RpcFields::CreateWallet => String::from("create_wallet"),
            RpcFields::DescribeTransfer => String::from("describe_transfer"),
            RpcFields::ExchangeMultisigKeys => String::from("exchange_multisig_keys"),
            RpcFields::Export => String::from("export_multisig_info"),
            RpcFields::GetTxProof => String::from("get_tx_proof"),
            RpcFields::GetTxById => String::from("get_transfer_by_txid"),
            RpcFields::GetVersion => String::from("get_version"),
            RpcFields::Id => String::from("0"),
            RpcFields::Import => String::from("import_multisig_info"),
            RpcFields::JsonRpcVersion => String::from("2.0"),
            RpcFields::Make => String::from("make_multisig"),
            RpcFields::Open => String::from("open_wallet"),
            RpcFields::Prepare => String::from("prepare_multisig"),
            RpcFields::Refresh => String::from("refresh"),
            RpcFields::SignMultisig => String::from("sign_multisig"),
            RpcFields::SubmitMultisig => String::from("submit_multisig"),
            RpcFields::SweepAll => String::from("sweep_all"),
            RpcFields::Transfer => String::from("transfer"),
            RpcFields::ValidateAddress => String::from("validate_address"),
            RpcFields::Verify => String::from("verify"),
        }
    }
}

enum DaemonFields {
    GetBlock,
    GetHeight,
    GetInfo,
    GetTransactions,
    Id,
    Version,
}

impl DaemonFields {
    pub fn value(&self) -> String {
        match *self {
            DaemonFields::GetBlock => String::from("get_block"),
            DaemonFields::GetHeight => String::from("get_height"),
            DaemonFields::GetInfo => String::from("get_info"),
            DaemonFields::GetTransactions => String::from("get_transactions"),
            DaemonFields::Id => String::from("0"),
            DaemonFields::Version => String::from("2.0"),
        }
    }
}

pub enum LockTimeLimit {
    Blocks,
}

impl LockTimeLimit {
    pub fn value(&self) -> u64 {
        match *self {
            LockTimeLimit::Blocks => 20,
        }
    }
}

/// Start monerod from the `--monero-location` flag
///
/// default: /home/$USER/monero-xxx-xxx
pub fn start_daemon() {
    info!("starting monerod");
    let blockchain_dir = get_blockchain_dir();
    let bin_dir = get_monero_location();
    let release_env = utils::get_release_env();
    let mut socks_proxy_host = utils::get_i2p_wallet_proxy_host();
    if socks_proxy_host.contains("http://") {
        let values = socks_proxy_host.split("http://");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        socks_proxy_host = v.remove(1);
    };
    let tx_proxy = format!("i2p,{}", socks_proxy_host);
    // proxy host can't have protocol
    let anon_in_port = get_anon_inbound_port();
    let destination = i2p::get_destination(Some(anon_in_port));
    let anon_inbound = format!("{},127.0.0.1:{}", destination, anon_in_port);
    let mut args = vec!["--data-dir", &blockchain_dir, "--detach"];
    if release_env == utils::ReleaseEnvironment::Development {
        args.push("--stagenet");
        let output = Command::new(format!("{}/monerod", bin_dir))
            .args(args)
            .spawn()
            .expect("monerod failed to start");
        debug!("{:?}", output.stdout);
    } else {
        args.push("--tx-proxy");
        args.push(&tx_proxy);
        args.push("--anonymous-inbound");
        args.push(&anon_inbound);
        let output = Command::new(format!("{}/monerod", bin_dir))
            .args(args)
            .spawn()
            .expect("monerod failed to start");
        debug!("{:?}", output.stdout);
    }
}

/// Start monero-wallet-rpc
pub fn start_rpc() {
    info!("starting monero-wallet-rpc");
    let cli_args = args::Args::parse();
    let bin_dir = get_monero_location();
    let port = get_rpc_port();
    let login = get_rpc_creds();
    let daemon_address = cli_args.monero_rpc_daemon;
    let rpc_login = format!("{}:{}", &login.username, &login.credential);
    let release_env = utils::get_release_env();
    let is_dev = release_env == utils::ReleaseEnvironment::Development;
    let wallet_path = if is_dev {
        ".neveko/stagenet/wallet/"
    } else {
        ".neveko/wallet/"
    };
    let wallet_dir = format!(
        "/home/{}/{}",
        std::env::var("USER").unwrap_or(String::from("user")),
        wallet_path
    );
    if cli_args.remote_node && !&daemon_address.contains(".i2p") {
        warn!("invalid i2p monero remote node detected");
    }
    // proxy host can't have protocol
    let mut proxy_host = utils::get_i2p_wallet_proxy_host();
    if proxy_host.contains("http://") {
        let values = proxy_host.split("http://");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        proxy_host = v.remove(1);
    }
    let mut args = vec![
        "--rpc-bind-port",
        &port,
        "--wallet-dir",
        &wallet_dir,
        "--rpc-login",
        &rpc_login,
    ];
    if cli_args.remote_node {
        args.push("--proxy");
        args.push(&proxy_host);
        args.push("--daemon-address");
        args.push(&daemon_address);
        args.push("--trusted-daemon");
        args.push("--daemon-ssl-allow-any-cert");
    } else {
        args.push("--daemon-address");
        args.push(&daemon_address);
    }
    if is_dev {
        args.push("--stagenet");
        let output = Command::new(format!("{}/monero-wallet-rpc", bin_dir))
            .args(args)
            .spawn()
            .expect("monero-wallet-rpc failed to start");
        debug!("{:?}", output.stdout);
    } else {
        let output = Command::new(format!("{}/monero-wallet-rpc", bin_dir))
            .args(args)
            .spawn()
            .expect("monero-wallet-rpc failed to start");
        debug!("{:?}", output.stdout);
    }
}

fn get_rpc_port() -> String {
    let args = args::Args::parse();
    let rpc = String::from(args.monero_rpc_host);
    let values = rpc.split(":");
    let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
    let port = v.remove(2);
    debug!("monero-wallet-rpc port: {}", port);
    port
}

pub fn get_daemon_port() -> u16 {
    let args = args::Args::parse();
    let rpc = String::from(args.monero_rpc_daemon);
    let values = rpc.split(":");
    let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
    let port = v.remove(2);
    debug!("monerod port: {}", port);
    match port.parse::<u16>() {
        Ok(p) => p,
        Err(_) => 0,
    }
}

pub fn get_tx_proxy_port() -> u16 {
    let args = args::Args::parse();
    let rpc = String::from(args.i2p_socks_proxy_host);
    let values = rpc.split(":");
    let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
    let port = v.remove(2);
    debug!("i2p socks port: {}", port);
    match port.parse::<u16>() {
        Ok(p) => p,
        Err(_) => 0,
    }
}

pub fn get_anon_inbound_port() -> u16 {
    let args = args::Args::parse();
    args.anon_inbound_port
}

/// Get monero rpc host from command line argument
fn get_blockchain_dir() -> String {
    let args = args::Args::parse();
    String::from(args.monero_blockchain_dir)
}

/// Get monero download location
fn get_monero_location() -> String {
    let args = args::Args::parse();
    let user = std::env::var("USER").unwrap_or(utils::empty_string());
    format!("/home/{}/{}", &user, &args.monero_location)
}

/// Get monero rpc host from the `--monero-rpc-host` cli arg
fn get_rpc_host() -> String {
    let args = args::Args::parse();
    let gui_host = std::env::var(crate::MONERO_WALLET_RPC_HOST).unwrap_or(utils::empty_string());
    let rpc = if gui_host == utils::empty_string() {
        String::from(args.monero_rpc_host)
    } else {
        gui_host
    };
    format!("{}/json_rpc", rpc)
}

/// Get creds from the `--monero-rpc-daemon` cli arg
fn get_rpc_creds() -> RpcLogin {
    let args = args::Args::parse();
    let username = String::from(args.monero_rpc_username);
    let credential = String::from(args.monero_rpc_cred);
    RpcLogin {
        username,
        credential,
    }
}

fn get_rpc_daemon() -> String {
    let args = args::Args::parse();
    let gui_host = std::env::var(crate::MONERO_DAEMON_HOST).unwrap_or(utils::empty_string());
    if gui_host == utils::empty_string() {
        String::from(args.monero_rpc_daemon)
    } else {
        gui_host
    }
}

/// Performs rpc 'get_version' method
pub async fn get_version() -> reqres::XmrRpcVersionResponse {
    info!("executing {}", RpcFields::GetVersion.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::GetVersion.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcVersionResponse>().await;
            debug!("{} response: {:?}", RpcFields::GetVersion.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_e) => Default::default(),
    }
}

/// Helper function for checking xmr rpc online during app startup
pub async fn check_rpc_connection() -> () {
    let res: reqres::XmrRpcVersionResponse = get_version().await;
    if res.result.version == INVALID_VERSION {
        error!("failed to connect to monero-wallet-rpc");
    }
}

/// Performs the xmr rpc 'verify' method
pub async fn verify(address: String, data: String, signature: String) -> bool {
    info!("executing {}", RpcFields::Verify.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcVerifyParams {
        address,
        data,
        signature,
    };
    let req = reqres::XmrRpcVerifyRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Verify.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcVerifyResponse>().await;
            debug!("{} response: {:?}", RpcFields::Verify.value(), res);
            match res {
                Ok(res) => {
                    if res.result.good {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        Err(_e) => false,
    }
}

/// Performs the xmr rpc 'create_wallet' method
pub async fn create_wallet(filename: &String, password: &String) -> bool {
    info!("executing {}", RpcFields::CreateWallet.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcCreateWalletParams {
        filename: String::from(filename),
        language: String::from("English"),
        password: String::from(password),
    };
    let req = reqres::XmrRpcCreateRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::CreateWallet.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            // The result from wallet creation is empty
            let res = response.text().await;
            debug!("{} response: {:?}", RpcFields::CreateWallet.value(), res);
            match res {
                Ok(r) => {
                    if r.contains("-1") {
                        return false;
                    }
                    true
                }
                _ => false,
            }
        }
        Err(_) => false,
    }
}

/// Set the wallet lock to true during operations to avoid collisons
///
/// on different types of wallet (e.g. order versus NEVEKO instance).
///
/// The open functionality will break on `false` if busy.
fn update_wallet_lock(filename: &String, closing: bool) -> bool {
    let is_busy: bool = match IS_WALLET_BUSY.lock() {
        Ok(m) => *m,
        Err(_) => true,
    };
    if is_busy && !closing {
        debug!("wallet {} is busy", filename);
        return false;
    }
    if !closing {
        *IS_WALLET_BUSY.lock().unwrap() = true;
        return true;
    } else {
        *IS_WALLET_BUSY.lock().unwrap() = false;
        return true;
    }
}

// TODO(c2m): search the app and use this output to either bail (false) or
// continue (true)

/// Performs the xmr rpc 'open_wallet' method
pub async fn open_wallet(filename: &String, password: &String) -> bool {
    let updated = update_wallet_lock(filename, false);
    if !updated {
        return updated;
    }
    info!("executing {}", RpcFields::Open.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcOpenWalletParams {
        filename: String::from(filename),
        password: String::from(password),
    };
    let req = reqres::XmrRpcOpenRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Open.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            // The result from wallet operation is empty
            let res = response.text().await;
            debug!("{} response: {:?}", RpcFields::Open.value(), res);
            match res {
                Ok(r) => {
                    if r.contains("-1") {
                        return false;
                    }
                    return true;
                }
                _ => false,
            }
        }
        Err(_) => false,
    }
}

/// Performs the xmr rpc 'close_wallet' method
pub async fn close_wallet(filename: &String, password: &String) -> bool {
    let updated = update_wallet_lock(filename, true);
    if !updated {
        return updated;
    }
    info!("executing {}", RpcFields::Close.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcOpenWalletParams {
        filename: String::from(filename),
        password: String::from(password),
    };
    let req = reqres::XmrRpcOpenRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Close.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            // The result from wallet operation is empty
            let res = response.text().await;
            debug!("{} response: {:?}", RpcFields::Close.value(), res);
            match res {
                Ok(_) => true,
                _ => false,
            }
        }
        Err(_) => false,
    }
}

/// Performs the xmr rpc 'get_balance' method
pub async fn get_balance() -> reqres::XmrRpcBalanceResponse {
    info!("executing {}", RpcFields::Balance.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcBalanceParams = reqres::XmrRpcBalanceParams {
        account_index: 0,
        address_indices: vec![0],
        all_accounts: false,
        strict: false,
    };
    let req = reqres::XmrRpcBalanceRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Balance.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcBalanceResponse>().await;
            debug!("{} response: {:?}", RpcFields::Balance.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'get_address' method
pub async fn get_address() -> reqres::XmrRpcAddressResponse {
    info!("executing {}", RpcFields::Address.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcAddressParams = reqres::XmrRpcAddressParams { account_index: 0 };
    let req = reqres::XmrRpcAddressRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Address.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcAddressResponse>().await;
            debug!("{} response: {:?}", RpcFields::Address.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'get_address' method
pub async fn validate_address(address: &String) -> reqres::XmrRpcValidateAddressResponse {
    info!("executing {}", RpcFields::ValidateAddress.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcValidateAddressParams = reqres::XmrRpcValidateAddressParams {
        address: String::from(address),
        any_net_type: false,
        allow_openalias: true,
    };
    let req = reqres::XmrRpcValidateAddressRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::ValidateAddress.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response
                .json::<reqres::XmrRpcValidateAddressResponse>()
                .await;
            debug!("{} response: {:?}", RpcFields::ValidateAddress.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}
// START Multisig

/// Performs the xmr rpc 'prepare_multisig' method
pub async fn prepare_wallet() -> reqres::XmrRpcPrepareResponse {
    info!("executing {}", RpcFields::Prepare.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Prepare.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcPrepareResponse>().await;
            debug!("{} response: {:?}", RpcFields::Prepare.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'make_multisig' method
pub async fn make_wallet(info: Vec<String>) -> reqres::XmrRpcMakeResponse {
    info!("executing {}", RpcFields::Make.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcMakeParams {
        multisig_info: info,
        threshold: 2,
    };
    let req = reqres::XmrRpcMakeRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Make.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcMakeResponse>().await;
            debug!("{} response: {:?}", RpcFields::Make.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'export_multisig_info' method
pub async fn export_multisig_info() -> reqres::XmrRpcExportResponse {
    info!("executing {}", RpcFields::Export.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Export.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcExportResponse>().await;
            debug!("{} response: {:?}", RpcFields::Export.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'import_multisig_info' method
pub async fn import_multisig_info(info: Vec<String>) -> reqres::XmrRpcImportResponse {
    info!("executing {}", RpcFields::Import.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcImportParams { info };
    let req = reqres::XmrRpcImportRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Import.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcImportResponse>().await;
            debug!("{} response: {:?}", RpcFields::Import.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'sign_multisig' method
pub async fn sign_multisig(tx_data_hex: String) -> reqres::XmrRpcSignMultisigResponse {
    info!("executing {}", RpcFields::SignMultisig.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcSignMultisigParams { tx_data_hex };
    let req = reqres::XmrRpcSignMultisigRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::SignMultisig.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcSignMultisigResponse>().await;
            debug!("{} response: {:?}", RpcFields::SignMultisig.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'submit_multisig' method
pub async fn submit_multisig(tx_data_hex: String) -> reqres::XmrRpcSubmitMultisigResponse {
    info!("executing {}", RpcFields::SubmitMultisig.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcSignMultisigParams { tx_data_hex };
    let req = reqres::XmrRpcSignMultisigRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::SubmitMultisig.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response
                .json::<reqres::XmrRpcSubmitMultisigResponse>()
                .await;
            debug!("{} response: {:?}", RpcFields::SubmitMultisig.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'exchange_multisig_keys' method
pub async fn exchange_multisig_keys(
    force_update_use_with_caution: bool,
    multisig_info: Vec<String>,
    password: &String,
) -> reqres::XmrRpcExchangeMultisigKeysResponse {
    info!("executing: {}", RpcFields::ExchangeMultisigKeys.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcExchangeMultisigKeysParams {
        force_update_use_with_caution,
        password: String::from(password),
        multisig_info,
    };
    let req = reqres::XmrRpcExchangeMultisigKeysRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::ExchangeMultisigKeys.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response
                .json::<reqres::XmrRpcExchangeMultisigKeysResponse>()
                .await;
            debug!(
                "{} response: {:?}",
                RpcFields::ExchangeMultisigKeys.value(),
                res
            );
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}
// END Multisig

/// Performs the xmr rpc 'check_tx_proof' method
pub async fn check_tx_proof(txp: &proof::TxProof) -> reqres::XmrRpcCheckTxProofResponse {
    info!("executing {}", RpcFields::CheckTxProof.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcCheckTxProofParams = reqres::XmrRpcCheckTxProofParams {
        address: String::from(&txp.subaddress),
        message: String::from(&txp.message),
        signature: String::from(&txp.signature),
        txid: String::from(&txp.hash),
    };
    let req = reqres::XmrRpcCheckTxProofRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::CheckTxProof.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcCheckTxProofResponse>().await;
            debug!("{} response: {:?}", RpcFields::CheckTxProof.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'get_tx_proof' method
pub async fn get_tx_proof(ptxp: proof::TxProof) -> reqres::XmrRpcGetTxProofResponse {
    info!("executing {}", RpcFields::GetTxProof.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcGetTxProofParams = reqres::XmrRpcGetTxProofParams {
        address: String::from(&ptxp.subaddress),
        message: String::from(&ptxp.message),
        txid: String::from(&ptxp.hash),
    };
    let req = reqres::XmrRpcGetTxProofRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::GetTxProof.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcGetTxProofResponse>().await;
            debug!("{} response: {:?}", RpcFields::GetTxProof.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'get_transfer_by_txid' method
pub async fn get_transfer_by_txid(txid: &str) -> reqres::XmrRpcGetTxByIdResponse {
    info!("executing: {}", RpcFields::GetTxById.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcGetTxByIdParams = reqres::XmrRpcGetTxByIdParams {
        txid: String::from(txid),
    };
    let req = reqres::XmrRpcGetTxByIdRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::GetTxById.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcGetTxByIdResponse>().await;
            debug!("{} response: {:?}", RpcFields::GetTxById.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'transfer' method
pub async fn transfer(d: reqres::Destination) -> reqres::XmrRpcTransferResponse {
    info!("executing {}", RpcFields::Transfer.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let mut destinations: Vec<reqres::Destination> = Vec::new();
    destinations.push(d);
    let params: reqres::XmrRpcTransferParams = reqres::XmrRpcTransferParams {
        account_index: 0,
        destinations,
        get_tx_key: false,
        priority: 0,
        ring_size: RING_SIZE,
        subaddr_indices: vec![0],
    };
    let req = reqres::XmrRpcTransfrerRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Transfer.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcTransferResponse>().await;
            debug!("{} response: {:?}", RpcFields::Transfer.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'describe_transfer' method
pub async fn describe_transfer(multisig_txset: &String) -> reqres::XmrRpcDescribeTransferResponse {
    info!("executing {}", RpcFields::DescribeTransfer.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcDescribeTransferParams = reqres::XmrRpcDescribeTransferParams {
        multisig_txset: String::from(multisig_txset),
    };
    let req = reqres::XmrRpcDescribeTransfrerRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::DescribeTransfer.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response
                .json::<reqres::XmrRpcDescribeTransferResponse>()
                .await;
            debug!(
                "{} response: {:?}",
                RpcFields::DescribeTransfer.value(),
                res
            );
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'sweep_all' method
pub async fn sweep_all(address: String) -> reqres::XmrRpcSweepAllResponse {
    info!("executing {}", RpcFields::SweepAll.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcSweepAllParams = reqres::XmrRpcSweepAllParams { address };
    let req = reqres::XmrRpcSweepAllRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::SweepAll.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcSweepAllResponse>().await;
            debug!("{} response: {:?}", RpcFields::SweepAll.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'create_address' method
pub async fn create_address() -> reqres::XmrRpcCreateAddressResponse {
    info!("executing {}", RpcFields::CreateAddress.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcCreateAddressParams =
        reqres::XmrRpcCreateAddressParams { account_index: 0 };
    let req = reqres::XmrRpcCreateAddressRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::CreateAddress.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcCreateAddressResponse>().await;
            debug!("{} response: {:?}", RpcFields::CreateAddress.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr rpc 'refresh' method
pub async fn refresh() -> reqres::XmrRpcRefreshResponse {
    info!("executing {}", RpcFields::Refresh.value());
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Refresh.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client
        .post(host)
        .json(&req)
        .send_with_digest_auth(&login.username, &login.credential)
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcRefreshResponse>().await;
            debug!("{} response: {:?}", RpcFields::Refresh.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

// Daemon requests
//-------------------------------------------------------------------

/// Performs the xmr daemon 'get_info' method
pub async fn get_info() -> reqres::XmrDaemonGetInfoResponse {
    info!("fetching daemon info");
    let client = reqwest::Client::new();
    let host = format!("{}/json_rpc", get_rpc_daemon());
    let req = reqres::XmrRpcRequest {
        jsonrpc: DaemonFields::Version.value(),
        id: DaemonFields::Id.value(),
        method: DaemonFields::GetInfo.value(),
    };
    match client.post(host).json(&req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetInfoResponse>().await;
            // add debug log here if needed for adding more info to home screen in gui
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr daemon 'get_info' method for proxied daemons
pub async fn p_get_info() -> Result<reqres::XmrDaemonGetInfoResponse, Box<dyn Error>> {
    info!("fetching proxy daemon info");
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    let host = format!("{}/json_rpc", get_rpc_daemon());
    let req = reqres::XmrRpcRequest {
        jsonrpc: DaemonFields::Version.value(),
        id: DaemonFields::Id.value(),
        method: DaemonFields::GetInfo.value(),
    };
    match client?.post(host).json(&req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetInfoResponse>().await;
            // add debug log here if needed for adding more info to home screen in gui
            match res {
                Ok(res) => Ok(res),
                _ => Ok(Default::default()),
            }
        }
        Err(_) => Ok(Default::default()),
    }
}

/// Performs the xmr daemon 'get_height' method
pub async fn get_height() -> reqres::XmrDaemonGetHeightResponse {
    info!("fetching daemon height");
    let client = reqwest::Client::new();
    let daemon = get_rpc_daemon();
    let req = format!("{}/{}", daemon, DaemonFields::GetHeight.value());
    match client.post(req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetHeightResponse>().await;
            // don't log this one. The fee estimator blows up logs (T_T)
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr daemon 'get_height' method for remote daemon
pub async fn p_get_height() -> Result<reqres::XmrDaemonGetHeightResponse, Box<dyn Error>> {
    info!("fetching proxy daemon height");
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    let daemon = get_rpc_daemon();
    let req = format!("{}/{}", daemon, DaemonFields::GetHeight.value());
    match client?.post(req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetHeightResponse>().await;
            // don't log this one. The fee estimator blows up logs (T_T)
            match res {
                Ok(res) => Ok(res),
                _ => Ok(Default::default()),
            }
        }
        Err(_) => Ok(Default::default()),
    }
}

/// Performs the xmr daemon 'get_block' method
pub async fn get_block(height: u64) -> reqres::XmrDaemonGetBlockResponse {
    info!("fetching block at height: {}", height);
    let client = reqwest::Client::new();
    let host = format!("{}/json_rpc", get_rpc_daemon());
    let params: reqres::XmrDaemonGetBlockParams = reqres::XmrDaemonGetBlockParams { height };
    let req = reqres::XmrDaemonGetBlockRequest {
        jsonrpc: DaemonFields::Version.value(),
        id: DaemonFields::Id.value(),
        method: DaemonFields::GetBlock.value(),
        params,
    };
    match client.post(host).json(&req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetBlockResponse>().await;
            // don't log this one. The fee estimator blows up logs (T_T)
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr daemon 'get_block' method for remone daemon
pub async fn p_get_block(height: u64) -> Result<reqres::XmrDaemonGetBlockResponse, Box<dyn Error>> {
    info!("fetching proxy block at height: {}", height);
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    let host = format!("{}/json_rpc", get_rpc_daemon());
    let params: reqres::XmrDaemonGetBlockParams = reqres::XmrDaemonGetBlockParams { height };
    let req = reqres::XmrDaemonGetBlockRequest {
        jsonrpc: DaemonFields::Version.value(),
        id: DaemonFields::Id.value(),
        method: DaemonFields::GetBlock.value(),
        params,
    };
    match client?.post(host).json(&req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetBlockResponse>().await;
            // don't log this one. The fee estimator blows up logs (T_T)
            match res {
                Ok(res) => Ok(res),
                _ => Ok(Default::default()),
            }
        }
        Err(_) => Ok(Default::default()),
    }
}

/// Performs the xmr daemon 'get_transactions' method
pub async fn get_transactions(txs_hashes: Vec<String>) -> reqres::XmrDaemonGetTransactionsResponse {
    info!("fetching {} transactions", txs_hashes.len());
    let client = reqwest::Client::new();
    let host = get_rpc_daemon();
    let url = format!("{}/{}", host, DaemonFields::GetTransactions.value());
    let req = reqres::XmrDaemonGetTransactionsRequest {
        txs_hashes,
        decode_as_json: true,
    };
    match client.post(url).json(&req).send().await {
        Ok(response) => {
            let res = response
                .json::<reqres::XmrDaemonGetTransactionsResponse>()
                .await;
            // don't log this one. The fee estimator blows up logs (T_T)
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default(),
    }
}

/// Performs the xmr daemon 'get_transactions' method for remote daemon
pub async fn p_get_transactions(
    txs_hashes: Vec<String>,
) -> Result<reqres::XmrDaemonGetTransactionsResponse, Box<dyn Error>> {
    info!("fetching {} transactions", txs_hashes.len());
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    let daemon = get_rpc_daemon();
    let url = format!("{}/{}", daemon, DaemonFields::GetTransactions.value());
    let req = reqres::XmrDaemonGetTransactionsRequest {
        txs_hashes,
        decode_as_json: true,
    };
    match client?.post(url).json(&req).send().await {
        Ok(response) => {
            let res = response
                .json::<reqres::XmrDaemonGetTransactionsResponse>()
                .await;
            // don't log this one. The fee estimator blows up logs (T_T)
            match res {
                Ok(res) => Ok(res),
                _ => Ok(Default::default()),
            }
        }
        Err(_) => Ok(Default::default()),
    }
}

/// enable multisig - `monero-wallet-cli --password <> --wallet-file <> set enable-multisig-experimental 1`
pub fn enable_experimental_multisig(wallet_file: &String) {
    warn!("Enabling experimental multisig...");
    let bin_dir = get_monero_location();
    let user = std::env::var("USER").unwrap_or(utils::empty_string());
    let file_path = format!("/home/{}/.{}/stagenet/wallet/{}", &user, crate::APP_NAME, &wallet_file);
    let wallet_password = utils::empty_string();
    let release_env = utils::get_release_env();
    let args = if release_env == utils::ReleaseEnvironment::Production {
        vec![
            "--password",       &wallet_password, 
            "--wallet-file",    &file_path,
            "set", "enable-multisig-experimental", "1"
        ]
    } else { 
        vec![
            "--stagenet",
            "--password",       &wallet_password, 
            "--wallet-file",    &file_path,
            "set", "enable-multisig-experimental", "1"
        ]
    };
    let mut output = Command::new(format!("{}/monero-wallet-cli", bin_dir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(args)
            .spawn()
            .expect("failed to enable experimental msig");
    let _ = std::io::stdout().flush();
    let mut stdin = output.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&wallet_password.as_bytes()).expect("Failed to write to stdin");
    });
    let d_output = output.wait_with_output().expect("Failed to read stdout");
    debug!("{:?}", d_output);
}
