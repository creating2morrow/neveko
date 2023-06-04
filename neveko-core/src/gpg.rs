use crate::{
    i2p,
    utils,
};
use gpgme::*;
use log::{
    debug,
    error,
    info,
};
use std::{
    error::Error,
    fs::File,
    io::Write,
    process::Command,
};

/// Searches for key, returns empty string if none exists
///
/// TODO(c2m): add more cli options
pub fn find_key() -> Result<String, Box<dyn Error>> {
    info!("searching for application gpg key");
    let proto = Protocol::OpenPgp;
    let mode = KeyListMode::LOCAL;
    let mut ctx = Context::from_protocol(proto)?;
    ctx.set_key_list_mode(mode)?;
    let name = i2p::get_destination(None);
    let mut keys = ctx.find_keys([&name])?;
    let mut k: String = utils::empty_string();
    for key in keys.by_ref().filter_map(|x| x.ok()) {
        let r_key: &str = key.id().unwrap_or("");
        if String::from(r_key) != utils::empty_string() {
            k = String::from(r_key);
            break;
        } else {
            error!("error finding gpg key");
        }
    }
    if keys.finish()?.is_truncated() {
        error!("key listing unexpectedly truncated");
    }
    Ok(k)
}

pub fn gen_key() {
    info!("creating gpg key");
    let output = Command::new("gpg")
        .args(["--batch", "--gen-key", "genkey-batch"])
        .spawn()
        .expect("gpg key generation failed");
    debug!("{:?}", output.stdout);
}

/// Export ascii armor app public gpg key
pub fn export_key() -> Result<Vec<u8>, Box<dyn Error>> {
    info!("exporting public key");
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    ctx.set_armor(true);
    let name = i2p::get_destination(None);
    let keys = {
        let mut key_iter = ctx.find_keys([&name])?;
        let keys: Vec<_> = key_iter.by_ref().collect::<Result<_, _>>()?;
        if key_iter.finish()?.is_truncated() {
            Err("key listing unexpectedly truncated")?;
        }
        keys
    };
    let mode = gpgme::ExportMode::empty();
    let mut output = Vec::new();
    ctx.export_keys(&keys, mode, &mut output)
        .map_err(|e| format!("export failed: {:?}", e))?;
    Ok(output)
}

/// Import gpg keys from contacts
pub fn import_key(cid: String, key: Vec<u8>) -> Result<(), Box<dyn Error>> {
    info!("importing key: {}", hex::encode(&key));
    let filename = format!("{}.neveko", &cid);
    let mut f = File::create(&filename)?;
    f.write_all(&key)?;
    let mut ctx = Context::from_protocol(gpgme::Protocol::OpenPgp)?;
    println!("reading file `{}'", &filename);
    let input = File::open(&filename)?;
    let mut data = Data::from_seekable_stream(input)?;
    let mode = None;
    mode.map(|m| data.set_encoding(m));
    ctx.import(&mut data)
        .map_err(|e| format!("import failed {:?}", e))?;
    utils::stage_cleanup(filename);
    Ok(())
}

pub fn encrypt(name: String, body: &Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let proto = Protocol::OpenPgp;
    let mut ctx = Context::from_protocol(proto)?;
    ctx.set_armor(true);
    let keys: Vec<Key> = ctx
        .find_keys([&name])?
        .filter_map(|x| x.ok())
        .filter(|k| k.can_encrypt())
        .collect();
    let filename = format!("{}.neveko", name);
    let mut f = File::create(&filename)?;
    f.write_all(body)?;
    let mut input =
        File::open(&filename).map_err(|e| format!("can't open file `{}': {}", filename, e))?;
    let mut output = Vec::new();
    ctx.encrypt(&keys, &mut input, &mut output)
        .map_err(|e| format!("encrypting failed: {:?}", e))?;
    debug!(
        "encrypted message body: {}",
        String::from_utf8(output.iter().cloned().collect()).unwrap_or(utils::empty_string())
    );
    utils::stage_cleanup(filename);
    Ok(output)
}

pub fn decrypt(mid: &String, body: &Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let proto = Protocol::OpenPgp;
    let mut ctx = Context::from_protocol(proto)?;
    ctx.set_armor(true);
    let filename = format!("{}.neveko", mid);
    let mut f = File::create(&filename)?;
    f.write_all(&body)?;
    let mut input =
        File::open(&filename).map_err(|e| format!("can't open file `{}': {}", filename, e))?;
    let mut output = Vec::new();
    ctx.decrypt(&mut input, &mut output)
        .map_err(|e| format!("decrypting failed: {:?}", e))?;
    utils::stage_cleanup(filename);
    Ok(output)
}

pub fn write_gen_batch() -> Result<(), Box<dyn Error>> {
    let name = i2p::get_destination(None);
    let data = format!(
        "%no-protection
        Key-Type: RSA
        Key-Length: 4096
        Subkey-Type: ECC
        Subkey-Curve: Curve25519
        Name-Real: {}
        Name-Email: {}
        Expire-Date: 0",
        name, name
    );
    let filename = format!("genkey-batch");
    let mut f = File::create(&filename)?;
    f.write_all(&data.into_bytes())?;
    Ok(())
}

pub fn sign_key(key: &str) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::from_protocol(gpgme::Protocol::OpenPgp)?;
    let mut keys = ctx.find_keys([key])?;
    let mut k: String = utils::empty_string();
    for ak in keys.by_ref().filter_map(|x| x.ok()) {
        let r_key: &str = ak.id().unwrap_or("");
        if String::from(r_key) != utils::empty_string() {
            k = String::from(r_key);
            break;
        } else {
            error!("error finding gpg key");
        }
    }
    debug!("key-id match: {}", k);
    let mut k2s_ctx = Context::from_protocol(gpgme::Protocol::OpenPgp)?;
    let key_to_sign = k2s_ctx
        .get_key(k)
        .map_err(|e| format!("no key matched given key-id: {:?}", e))?;
    let name = Some(i2p::get_destination(None));
    if let Some(app_key) = name {
        let key = k2s_ctx
            .get_secret_key(app_key)
            .map_err(|e| format!("unable to find signing key: {:?}", e))?;
        debug!("app key: {:?}", key.id());
        k2s_ctx
            .add_signer(&key)
            .map_err(|e| format!("add_signer() failed: {:?}", e))?;
    }

    k2s_ctx
        .sign_key(&key_to_sign, None::<String>, Default::default())
        .map_err(|e| format!("signing failed: {:?}", e))?;

    println!("Signed key for {}", key);
    Ok(())
}
