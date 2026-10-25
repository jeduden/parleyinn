//! The macOS Keychain (the plan's ruling 2): ParleyInn's keys as generic
//! passwords of the service `parinn`, through `security-framework`,
//! whose `unsafe` stays inside that crate.

use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

use super::Keychain;

/// The keychain service every ParleyInn key is kept under.
const SERVICE: &str = "parinn";
/// The status the Keychain answers when it holds no such item.
const ITEM_NOT_FOUND: i32 = -25300;

/// The login keychain of the running OS user.
#[derive(Debug, Clone, Copy)]
pub struct MacKeychain;

impl Keychain for MacKeychain {
    fn get(&self, account: &str) -> Result<Option<String>, String> {
        match get_generic_password(SERVICE, account) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| "the keychain item is not UTF-8".to_owned()),
            Err(e) if e.code() == ITEM_NOT_FOUND => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn add(&self, account: &str, secret: &str) -> Result<bool, String> {
        // `set_generic_password` would replace an item; a key is kept
        // once, so one already there is left as it is.
        if self.get(account)?.is_some() {
            return Ok(false);
        }
        set_generic_password(SERVICE, account, secret.as_bytes())
            .map(|()| true)
            .map_err(|e| e.to_string())
    }

    fn delete(&self, account: &str) -> Result<(), String> {
        match delete_generic_password(SERVICE, account) {
            Err(e) if e.code() != ITEM_NOT_FOUND => Err(e.to_string()),
            _ => Ok(()),
        }
    }
}
