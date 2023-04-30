use crate::{args, reqres, utils::{self, get_release_env, ReleaseEnvironment}, proof};
use clap::Parser;
use diqwest::WithDigestAuth;
use log::{debug, error, info};
use std::process::Command;

/// Current xmr ring size updated here.
const RING_SIZE: u32 = 0x10;

struct RpcLogin {
    username: String,
    credential: String,
}

enum RpcFields {
    Address,
    Balance,
    CheckTxProof,
    Close,
    Create,
    Export,
    Finalize,
    GetTxProof,
    GetTxById,
    GetVersion,
    Id,
    Import,
    JsonRpcVersion,
    Make,
    Open,
    Prepare,
    SignMultisig,
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
            RpcFields::Create => String::from("create_wallet"),
            RpcFields::Export => String::from("export_multisig_info"),
            RpcFields::Finalize => String::from("finalize_multisig"),
            RpcFields::GetTxProof => String::from("get_tx_proof"),
            RpcFields::GetTxById => String::from("get_transfer_by_txid"),
            RpcFields::GetVersion => String::from("get_version"),
            RpcFields::Id => String::from("0"),
            RpcFields::Import => String::from("import_multisig_info"),
            RpcFields::JsonRpcVersion => String::from("2.0"),
            RpcFields::Make => String::from("make_multisig"),
            RpcFields::Open => String::from("open_wallet"),
            RpcFields::Prepare => String::from("prepare_multisig"),
            RpcFields::SignMultisig => String::from("sign_multisig"),
            RpcFields::SweepAll => String::from("sweep_all"),
            RpcFields::Transfer => String::from("transfer"),
            RpcFields::ValidateAddress => String::from("validate_address"),
            RpcFields::Verify => String::from("verify"),
        }
    }
}

enum DaemonFields {
    GetInfo,
    Id,
    Version,
}

impl DaemonFields {
    pub fn value(&self) -> String {
        match *self {
            DaemonFields::GetInfo => String::from("get_info"),
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
        match *self { LockTimeLimit::Blocks => 20, }
    }
}

/// Start monerod from the -`-monero-location` flag
/// 
/// default: /home/$USER/monero-xxx-xxx
pub fn start_daemon() {
    info!("starting monerod");
    let blockchain_dir = get_blockchain_dir();
    let bin_dir = get_monero_location();
    let release_env = get_release_env();
    if release_env == ReleaseEnvironment::Development {
        let args = ["--data-dir", &blockchain_dir, "--stagenet", "--detach"];
        let output = Command::new(format!("{}/monerod", bin_dir))
            .args(args)
            .spawn()
            .expect("monerod failed to start");
        debug!("{:?}", output.stdout);
    } else {
        let args = ["--data-dir", &blockchain_dir, "--detach"];
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
    let bin_dir = get_monero_location();
    let port = get_rpc_port();
    let login = get_rpc_creds();
    let daemon_address = get_rpc_daemon();
    let rpc_login = format!("{}:{}", &login.username, &login.credential);
    let mut wallet_dir = format!("/home/{}/.nevmes/stagenet/wallet/",
        std::env::var("USER").unwrap_or(String::from("user")),
    );
    let release_env = get_release_env();
    if release_env == ReleaseEnvironment::Development {
        let args = [
            "--rpc-bind-port", &port, 
            "--wallet-dir", &wallet_dir,
            "--rpc-login", &rpc_login, 
            "--daemon-address", &daemon_address, 
            "--stagenet"
        ];
        let output = Command::new(format!("{}/monero-wallet-rpc", bin_dir))
            .args(args)
            .spawn()
            .expect("monero-wallet-rpc failed to start");
        debug!("{:?}", output.stdout);
    } else {
        wallet_dir = format!("/home/{}/.nevmes/wallet/",
            std::env::var("USER").unwrap_or(String::from("user")),
        );
        let args = ["--rpc-bind-port", &port, "--wallet-dir", &wallet_dir,
        "--rpc-login", &rpc_login, "--daemon-address", &daemon_address];
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
    let  port = v.remove(2);
    debug!("monero-wallet-rpc port: {}", port);
    port
}

/// Get monero rpc host from command line argument
fn get_blockchain_dir() -> String {
    let args = args::Args::parse();
    String::from(args.monero_blockchain_dir)
}

/// Get monero download location
fn get_monero_location() -> String {
    let args = args::Args::parse();
    String::from(args.monero_location)
}

/// Get monero rpc host from the `--monero-rpc-host` cli arg
fn get_rpc_host() -> String {
    let args = args::Args::parse();
    let rpc = String::from(args.monero_rpc_host);
    format!("{}/json_rpc", rpc)
}

/// Get creds from the `--monero-rpc-daemon` cli arg
fn get_rpc_creds() -> RpcLogin {
    let args = args::Args::parse();
    let username = String::from(args.monero_rpc_username);
    let credential = String::from(args.monero_rpc_cred);
    RpcLogin { username, credential }
}

fn get_rpc_daemon() -> String {
    let args = args::Args::parse();
    let daemon = String::from(args.monero_rpc_daemon);
    format!("{}/json_rpc", daemon)
}

/// Performs rpc 'get_version' method
pub async fn get_version() -> reqres::XmrRpcVersionResponse {
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::GetVersion.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcVersionResponse>().await;
            debug!("get version response: {:?}", res);
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
    if res.result.version == 0 {
        error!("failed to connect to monero-wallet-rpc");
    }
}

/// Performs the xmr rpc 'verify' method
pub async fn verify_signature(address: String, data: String, signature: String) -> String {
    info!("signature verification in progress");
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
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcVerifyResponse>().await;
            debug!("verify response: {:?}", res);
            match res {
                Ok(res) => {
                    if res.result.good {
                        req.params.address
                    } else {
                        utils::ApplicationErrors::LoginError.value()
                    }
                }
                _ => utils::ApplicationErrors::LoginError.value(),
            }
        }
        Err(_e) => utils::ApplicationErrors::LoginError.value(),
    }
}

/// Performs the xmr rpc 'create_wallet' method
pub async fn create_wallet(filename: String) -> bool {
    info!("creating wallet: {}", &filename);
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcCreateWalletParams {
        filename,
        language: String::from("English"),
    };
    let req = reqres::XmrRpcCreateRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Create.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            // The result from wallet creation is empty
            let res = response.text().await;
            debug!("create response: {:?}", res);
            match res {
                Ok(r) => {
                    if r.contains("-1") {
                        return false;
                    }
                    true
                },
                _ => false,
            }
        }
        Err(_) => false
    }
}

/// Performs the xmr rpc 'open_wallet' method
pub async fn open_wallet(filename: String) -> bool {
    info!("opening wallet for {}", &filename);
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcOpenWalletParams {
        filename,
    };
    let req = reqres::XmrRpcOpenRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Open.value(),
        params,
    };
    debug!("open request: {:?}", req);
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            // The result from wallet operation is empty
            let res = response.text().await;
            debug!("open response: {:?}", res);
            match res {
                Ok(r) => {
                    if r.contains("-1") {
                        return false;
                    }
                    return true;
                },
                _ => false,
            }
        }
        Err(_) => false
    }
}

/// Performs the xmr rpc 'close_wallet' method
pub async fn close_wallet(filename: String) -> bool {
    info!("closing wallet for {}", &filename);
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcOpenWalletParams {
        filename,
    };
    let req = reqres::XmrRpcOpenRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Close.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            // The result from wallet operation is empty
            let res = response.text().await;
            debug!("close response: {:?}", res);
            match res {
                Ok(_) => true,
                _ => false,
            }
        }
        Err(_) => false
    }
}

/// Performs the xmr rpc 'get_balance' method
pub async fn get_balance() -> reqres::XmrRpcBalanceResponse {
    info!("fetching wallet balance");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcBalanceParams = reqres::XmrRpcBalanceParams { 
        account_index: 0, address_indices: vec![0], all_accounts: false, strict: false, 
    };
    let req = reqres::XmrRpcBalanceRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Balance.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcBalanceResponse>().await;
            debug!("balance response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'get_address' method
pub async fn get_address() -> reqres::XmrRpcAddressResponse {
    info!("fetching wallet address");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcAddressParams = reqres::XmrRpcAddressParams { 
        account_index: 0, address_index: vec![0], 
    };
    let req = reqres::XmrRpcAddressRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Address.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcAddressResponse>().await;
            debug!("address response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'get_address' method
pub async fn validate_address(address: &String) -> reqres::XmrRpcValidateAddressResponse {
    info!("validating wallet address");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcValidateAddressParams = reqres::XmrRpcValidateAddressParams { 
        address: String::from(address), any_net_type: false, allow_openalias: true, 
    };
    let req = reqres::XmrRpcValidateAddressRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::ValidateAddress.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcValidateAddressResponse>().await;
            debug!("validate_address response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}
// START Multisig

/// Performs the xmr rpc 'prepare_multisig' method
pub async fn prepare_wallet() -> reqres::XmrRpcPrepareResponse {
    info!("prepare msig wallet");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Prepare.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcPrepareResponse>().await;
            debug!("prepare response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'make_multisig' method
pub async fn make_wallet(info: Vec<String>) -> reqres::XmrRpcMakeResponse {
    info!("make msig wallet");
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
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcMakeResponse>().await;
            debug!("make response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'finalize_multisig' method
pub async fn finalize_wallet(info: Vec<String>) -> reqres::XmrRpcFinalizeResponse {
    info!("finalize msig wallet");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcFinalizeParams {
        multisig_info: info,
    };
    let req = reqres::XmrRpcFinalizeRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Finalize.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcFinalizeResponse>().await;
            debug!("finalize response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'export_multisig_info' method
pub async fn export_multisig_info() -> reqres::XmrRpcExportResponse {
    info!("export msig info");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let req = reqres::XmrRpcRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Export.value(),
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcExportResponse>().await;
            debug!("export msig response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'import_multisig_info' method
pub async fn import_multisig_info(info: Vec<String>) -> reqres::XmrRpcImportResponse {
    info!("import msig wallet");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcImportParams {
        info,
    };
    let req = reqres::XmrRpcImportRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::Import.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcImportResponse>().await;
            debug!("import msig info response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'sign_multisig' method
pub async fn sign_multisig(tx_data_hex: String) -> reqres::XmrRpcSignMultisigResponse {
    info!("sign msig txset");
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params = reqres::XmrRpcSignMultisigParams {
        tx_data_hex,
    };
    let req = reqres::XmrRpcSignMultisigRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::SignMultisig.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcSignMultisigResponse>().await;
            debug!("sign msig txset response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}
// END Multisig

/// Performs the xmr rpc 'check_tx_proof' method
pub async fn check_tx_proof(txp: &proof::TxProof) -> reqres::XmrRpcCheckTxProofResponse {
    info!("check_tx_proof proof: {:?}", txp);
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcCheckTxProofParams = reqres::XmrRpcCheckTxProofParams { 
        address: String::from(&txp.address),
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
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcCheckTxProofResponse>().await;
            debug!("check_tx_proof response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'get_tx_proof' method
pub async fn get_tx_proof(ptxp: proof::TxProof) -> reqres::XmrRpcGetTxProofResponse {
    info!("fetching proof: {:?}", &ptxp.hash);
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcGetTxProofParams = reqres::XmrRpcGetTxProofParams { 
        address: String::from(&ptxp.address),
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
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcGetTxProofResponse>().await;
            debug!("get_tx_proof response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'get_transfer_by_txid' method
pub async fn get_transfer_by_txid(txid: &str) -> reqres::XmrRpcGetTxByIdResponse {
    info!("fetching tx: {:?}", txid);
    let client = reqwest::Client::new();
    let host = get_rpc_host();
    let params: reqres::XmrRpcGetTxByIdParams = reqres::XmrRpcGetTxByIdParams { 
        txid: String::from(txid)
    };
    let req = reqres::XmrRpcGetTxByIdRequest {
        jsonrpc: RpcFields::JsonRpcVersion.value(),
        id: RpcFields::Id.value(),
        method: RpcFields::GetTxById.value(),
        params,
    };
    let login: RpcLogin = get_rpc_creds();
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcGetTxByIdResponse>().await;
            debug!("get_transfer_by_txid response: {:?}", res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'transfer' method
pub async fn transfer(d: reqres::Destination) -> reqres::XmrRpcTransferResponse {
    info!("transfer");
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
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcTransferResponse>().await;
            debug!("{} response: {:?}", RpcFields::Transfer.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

/// Performs the xmr rpc 'sweep_all' method
pub async fn sweep_all(address: String) -> reqres::XmrRpcSweepAllResponse {
    info!("sweep_all");
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
    match client.post(host).json(&req)
    .send_with_digest_auth(&login.username, &login.credential).await {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcSweepAllResponse>().await;
            debug!("{} response: {:?}", RpcFields::SweepAll.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}

// Daemon requests
//-------------------------------------------------------------------
/// Performs the xmr daemon 'get_info' method
pub async fn get_info() -> reqres::XmrDaemonGetInfoResponse {
    info!("fetching daemon info");
    let client = reqwest::Client::new();
    let host = get_rpc_daemon();
    let req = reqres::XmrRpcRequest {
        jsonrpc: DaemonFields::Version.value(),
        id: DaemonFields::Id.value(),
        method: DaemonFields::GetInfo.value(),
    };
    match client.post(host).json(&req).send().await {
        Ok(response) => {
            let res = response.json::<reqres::XmrDaemonGetInfoResponse>().await;
            debug!("{} response: {:?}", DaemonFields::GetInfo.value(), res);
            match res {
                Ok(res) => res,
                _ => Default::default(),
            }
        }
        Err(_) => Default::default()
    }
}
