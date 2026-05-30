use keyring_core::{Entry, Error};

use crate::error::AppResult;

const SERVICE: &str = "com.v3rdant.exprepo";
const ACCOUNT: &str = "github-token";

/// Select the platform-native credential store. keyring 4.x splits store
/// selection (the `keyring` facade) from the `Entry` API (`keyring_core`); a
/// store must be registered once before any `Entry` is used. `true` avoids the
/// non-persistent Linux keyutils store in favour of Secret Service.
///
/// Non-fatal: on a headless box with no Secret Service running this fails, and
/// token storage is simply unavailable until a session keyring exists.
pub fn init_store() {
    if let Err(e) = keyring::use_native_store(true) {
        eprintln!("keyring: native store unavailable, secrets disabled: {e}");
    }
}

/// Store the GitHub PAT in the OS keyring (Secret Service / Keychain /
/// Credential Manager / Android Keystore depending on platform).
pub fn set_token(token: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, ACCOUNT)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_token() -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE, ACCOUNT)?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn has_token() -> bool {
    matches!(get_token(), Ok(Some(_)))
}

pub fn delete_token() -> AppResult<()> {
    let entry = Entry::new(SERVICE, ACCOUNT)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
