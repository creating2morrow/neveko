//! Structs for all http requests

use serde::{
    Deserialize,
    Serialize,
};

// All http requests and responses are here

// START XMR Structs
// Reference: https://www.getmonero.org/resources/developer-guides/wallet-rpc.html
//            https://www.getmonero.org/resources/developer-guides/daemon-rpc.html

// params
#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcValidateAddressParams {
    pub address: String,
    pub any_net_type: bool,
    pub allow_openalias: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcSignParams {
    pub data: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcVerifyParams {
    pub address: String,
    pub data: String,
    pub signature: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcCreateWalletParams {
    pub filename: String,
    pub language: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcOpenWalletParams {
    pub filename: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcPrepareParams {
    pub enable_experimental_multisig: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcMakeParams {
    pub multisig_info: Vec<String>,
    pub threshold: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcImportParams {
    pub info: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcSignMultisigParams {
    pub tx_data_hex: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcExchangeMultisigKeysParams {
    pub force_update_use_with_caution: bool,
    pub multisig_info: Vec<String>,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcBalanceParams {
    pub account_index: u8,
    pub address_indices: Vec<u8>,
    pub all_accounts: bool,
    pub strict: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcAddressParams {
    pub account_index: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcCheckTxProofParams {
    pub address: String,
    pub message: String,
    pub signature: String,
    pub txid: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcTxProofParams {
    pub address: String,
    pub message: String,
    pub txid: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcGetTxProofParams {
    pub address: String,
    pub message: String,
    pub txid: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcGetTxByIdParams {
    pub txid: String,
}

#[derive(Default, Deserialize, Serialize, Debug, PartialEq)]
pub struct Destination {
    pub address: String,
    pub amount: u128,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcTransferParams {
    pub destinations: Vec<Destination>,
    pub account_index: u32,
    pub subaddr_indices: Vec<u32>,
    pub priority: u8,
    pub ring_size: u32,
    pub get_tx_key: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcDescribeTransferParams {
    pub multisig_txset: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcSweepAllParams {
    pub address: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcCreateAddressParams {
    pub account_index: u8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrDaemonGetBlockParams {
    pub height: u64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcChangePasswordParams {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcQueryKeyParams {
    pub key_type: String,
}

// requests
#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcValidateAddressRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcValidateAddressParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcCreateRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcCreateWalletParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcOpenRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcOpenWalletParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrDaemonGetBlockRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrDaemonGetBlockParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrDaemonGetTransactionsRequest {
    pub txs_hashes: Vec<String>,
    pub decode_as_json: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcAddressRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcAddressParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcBalanceRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcBalanceParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcPrepareRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcPrepareParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcMakeRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcMakeParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcImportRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcImportParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcSignMultisigRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcSignMultisigParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcExchangeMultisigKeysRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcExchangeMultisigKeysParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcSignRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcSignParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcVerifyRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcVerifyParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcCheckTxProofRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcCheckTxProofParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcGetTxProofRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcGetTxProofParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcGetTxByIdRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcGetTxByIdParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcTransfrerRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcTransferParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcDescribeTransfrerRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcDescribeTransferParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcSweepAllRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcSweepAllParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XmrRpcCreateAddressRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcCreateAddressParams,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcChangePasswordRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcChangePasswordParams,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcQueryKeyRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: XmrRpcQueryKeyParams,
}

// results
#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcValidateAddressResult {
    pub integrated: bool,
    pub nettype: String,
    pub openalias_address: String,
    pub subaddress: bool,
    pub valid: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSignResult {
    pub signature: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcVerifyResult {
    pub good: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcVersionResult {
    pub version: u32,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcPrepareResult {
    pub multisig_info: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcMakeResult {
    pub address: String,
    pub multisig_info: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcExportResult {
    pub info: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcImportResult {
    pub n_outputs: u8,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSignMultisigResult {
    pub tx_data_hex: String,
    pub tx_hash_list: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSubmitMultisigResult {
    pub tx_hash_list: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcExchangeMultisigKeysResult {
    pub address: String,
    pub multisig_info: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct SubAddressInfo {
    pub account_index: u64,
    pub address_index: u64,
    pub address: String,
    pub balance: u128,
    pub unlocked_balance: u128,
    pub label: String,
    pub num_unspent_outputs: u64,
    pub time_to_unlock: u128,
    pub blocks_to_unlock: u64,
}

#[derive(Debug, Default, Deserialize)]
pub struct Address {
    pub address: String,
    pub address_index: u64,
    pub label: String,
    pub used: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcAddressResult {
    pub address: String,
    pub addresses: Vec<Address>,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcBalanceResult {
    pub balance: u128,
    pub unlocked_balance: u128,
    pub multisig_import_needed: bool,
    pub time_to_unlock: u128,
    pub blocks_to_unlock: u64,
    pub per_subaddress: Vec<SubAddressInfo>,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcCheckTxProofResult {
    pub confirmations: u64,
    pub good: bool,
    pub in_pool: bool,
    pub received: u128,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcGetTxProofResult {
    pub signature: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct SubAddressIndex {
    pub major: u64,
    pub minor: u64,
}

#[derive(Debug, Default, Deserialize)]
pub struct Transfer {
    pub address: String,
    pub amount: u128,
    pub amounts: Vec<u128>,
    /// On zero conf this field is missing
    pub confirmations: Option<u64>,
    pub double_spend_seen: bool,
    pub fee: u128,
    pub height: u64,
    pub locked: bool,
    pub note: String,
    pub payment_id: String,
    pub subaddr_index: SubAddressIndex,
    pub subaddr_indices: Vec<SubAddressIndex>,
    pub suggested_confirmations_threshold: u64,
    pub timestamp: u64,
    pub txid: String,
    pub r#type: String,
    pub unlock_time: u64,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcGetTxByIdResult {
    pub transfer: Transfer,
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcTranferResult {
    pub amount: u128,
    pub fee: u128,
    pub multisig_txset: String,
    pub tx_blob: String,
    pub tx_hash: String,
    pub tx_key: String,
    pub tx_metadata: String,
    pub unsigned_txset: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TransferDescription {
    pub amount_in: u128,
    pub amount_out: u128,
    pub recepients: Vec<Destination>,
    pub change_address: String,
    pub change_amount: u128,
    pub fee: u128,
    pub ring_size: u64,
    pub unlock_time: u64,
    pub dummy_outputs: u64,
    pub extra: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcDescribeTranferResult {
    pub desc: Vec<TransferDescription>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct KeyImageList {
    key_images: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcSweepAllResult {
    pub amount_list: Vec<u128>,
    pub fee_list: Vec<u128>,
    pub multisig_txset: String,
    pub spent_key_images_list: Vec<KeyImageList>,
    pub tx_hash_list: Option<Vec<String>>,
    pub unsigned_txset: String,
    pub weight_list: Vec<u128>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcCreateAddressResult {
    pub address: String,
    pub address_index: u64,
    pub address_indices: Vec<u64>,
    pub addresses: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcRefreshResult {
    pub blocks_fetched: u64,
    pub received_money: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcIsMultisigResult {
    pub multisig: bool,
    pub ready: bool,
    pub threshold: u16,
    pub total: u16,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcGetHeightResult {
    pub height: u64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcQueryKeyResult {
    pub key: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrDaemonGetInfoResult {
    pub adjusted_time: u64,
    pub alt_blocks_count: u64,
    pub block_size_limit: u64,
    pub block_size_median: u64,
    pub block_weight_median: u64,
    pub bootstrap_daemon_address: String,
    pub busy_syncing: bool,
    pub credits: u64,
    pub cumulative_difficulty: u64,
    pub cumulative_difficulty_top64: u64,
    pub database_size: u64,
    pub difficulty: u64,
    pub difficulty_top64: u64,
    pub free_space: u64,
    pub grey_peerlist_size: u64,
    pub height: u64,
    pub height_without_bootstrap: u64,
    pub incoming_connections_count: u32,
    pub mainnet: bool,
    pub nettype: String,
    pub offline: bool,
    pub outgoing_connections_count: u32,
    pub restricted: bool,
    pub rpc_connections_count: u32,
    pub stagenet: bool,
    pub start_time: u64,
    pub status: String,
    pub synchronized: bool,
    pub target: u32,
    pub target_height: u32,
    pub testnet: bool,
    pub top_block_hash: String,
    pub top_hash: String,
    pub tx_count: u64,
    pub tx_pool_size: u32,
    pub untrusted: bool,
    pub update_available: bool,
    pub version: String,
    pub was_bootstrap_ever_used: bool,
    pub white_peerlist_size: u32,
    pub wide_cumulative_difficulty: String,
    pub wide_difficulty: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BlockHeader {
    pub block_size: u32,
    pub block_weight: u32,
    pub cumulative_difficulty: u128,
    pub cumulative_difficulty_top64: u128,
    pub depth: u32,
    pub difficulty: u128,
    pub difficulty_top64: u128,
    pub hash: String,
    pub height: u64,
    pub long_term_weight: u64,
    pub major_version: u32,
    pub miner_tx_hash: String,
    pub minor_version: u32,
    pub nonce: u32,
    pub num_txes: u64,
    pub orphan_status: bool,
    pub pow_hash: String,
    pub prev_hash: String,
    pub reward: u64,
    pub timestamp: u64,
    pub wide_cumulative_difficulty: String,
    pub wide_difficulty: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrDaemonGetBlockResult {
    pub blob: String,
    pub block_header: BlockHeader,
    pub credits: u64,
    pub json: String,
    pub miner_tx_hash: String,
    pub status: String,
    pub top_hash: String,
    /// For some reason this field just disappears on non-
    ///
    /// coinbase transactions instead of being an empty list.
    pub tx_hashes: Option<Vec<String>>,
    pub untrusted: bool,
}
// responses

#[derive(Debug, Default, Deserialize)]
pub struct XmrDaemonGetHeightResponse {
    pub hash: String,
    pub height: u64,
    pub status: String,
    pub untrusted: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrDaemonGetInfoResponse {
    pub result: XmrDaemonGetInfoResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrDaemonGetBlockResponse {
    pub result: XmrDaemonGetBlockResult,
}

/// Only extract the json string. TODO(c2m): map to a struct
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrDaemonGetTransactionsResponse {
    pub txs_as_json: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSignResponse {
    pub result: XmrRpcSignResult,
}

#[derive(Deserialize, Debug)]
pub struct XmrRpcVerifyResponse {
    pub result: XmrRpcVerifyResult,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmrRpcVersionResponse {
    pub result: XmrRpcVersionResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcPrepareResponse {
    pub result: XmrRpcPrepareResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcBalanceResponse {
    pub result: XmrRpcBalanceResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcValidateAddressResponse {
    pub result: XmrRpcValidateAddressResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcAddressResponse {
    pub result: XmrRpcAddressResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcMakeResponse {
    pub result: XmrRpcMakeResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcExportResponse {
    pub result: XmrRpcExportResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcImportResponse {
    pub result: XmrRpcImportResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSignMultisigResponse {
    pub result: XmrRpcSignMultisigResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSubmitMultisigResponse {
    pub result: XmrRpcSubmitMultisigResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcExchangeMultisigKeysResponse {
    pub result: XmrRpcExchangeMultisigKeysResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcCheckTxProofResponse {
    pub result: XmrRpcCheckTxProofResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcGetTxProofResponse {
    pub result: XmrRpcGetTxProofResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcGetTxByIdResponse {
    pub result: XmrRpcGetTxByIdResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcTransferResponse {
    pub result: XmrRpcTranferResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcDescribeTransferResponse {
    pub result: XmrRpcDescribeTranferResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcSweepAllResponse {
    pub result: XmrRpcSweepAllResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcCreateAddressResponse {
    pub result: XmrRpcCreateAddressResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcRefreshResponse {
    pub result: XmrRpcRefreshResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcIsMultisigResponse {
    pub result: XmrRpcIsMultisigResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcGetHeightResponse {
    pub result: XmrRpcGetHeightResult,
}

#[derive(Debug, Default, Deserialize)]
pub struct XmrRpcQueryKeyResponse {
    pub result: XmrRpcQueryKeyResult,
}
// END XMR Structs

/// Container for the message decipher
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DecipheredMessageBody {
    pub mid: String,
    pub body: String,
}

/// Invoice response for host.b32.i2p/invoice
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Invoice {
    pub address: String,
    pub pay_threshold: u128,
    pub conf_threshold: u64,
}

/// Not to be confused with the PaymentProof guard.
///
/// This is the response when proving payment
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Jwp {
    pub jwp: String,
}

/// For handling 402, 404 and 500 error responses
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub error: String,
}

/// Handle intial order information for request
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct OrderRequest {
    pub cid: String,
    pub adjudicator: String,
    pub pid: String,
    pub ship_address: String,
    pub quantity: u128,
}

/// Handle multisig info requests
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct MultisigInfoRequest {
    pub contact: String,
    /// Send empty array on prepare info request
    pub info: Vec<String>,
    /// flag for adjudicator to create create multisig wallet for order
    pub init_adjudicator: bool,
    /// We need to know when the first kex round occurs
    pub kex_init: bool,
    /// valid values are found in lines 21-26 of market.rs
    pub msig_type: String,
    pub orid: String,
}

/// Request for signing and submitting the unsigned txset
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SignAndSubmitRequest {
    pub orid: String,
    pub txset: String,
}

/// Response for the order finalization
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FinalizeOrderResponse {
    pub orid: String,
    /// This is enciphered by the customer Neveko Message Secret Key
    pub delivery_info: String,
    /// This is used to finalize delivery confirmations
    pub vendor_update_success: bool,
}

/// Response for the vendor mode
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct VendorModeResponse {
    pub mode: bool,
}
