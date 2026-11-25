//! A private vault: secrets are encrypted with
//! ChaCha20-Poly1305 and stored in the database; the one key lives in the
//! system keychain. Each secret is bound to its name and to what it is for
//! (the tracker address for a token), so changing either in the database
//! makes it unreadable instead of sending it elsewhere.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use memorius_core::store::Store;

const KEYCHAIN_SERVICE: &str = "app.memorius.desktop";
const KEYCHAIN_ACCOUNT: &str = "vault-key";

pub struct Vault {
    database: PathBuf,
    /// Read from the keychain once, so macOS asks at most once per run.
    key: Mutex<Option<Key>>,
}

impl Vault {
    pub fn new(database: &Path) -> Self {
        Self {
            database: database.to_owned(),
            key: Mutex::new(None),
        }
    }

    fn cipher(&self) -> Result<ChaCha20Poly1305, String> {
        let mut key = self.key.lock().unwrap_or_else(PoisonError::into_inner);
        if key.is_none() {
            *key = Some(keychain_key()?);
        }
        Ok(ChaCha20Poly1305::new(key.as_ref().ok_or("no vault key")?))
    }

    /// The secret saved under `name` for `bound_to`; none when there is
    /// none or it was saved for something else (it must be saved again).
    pub fn get(&self, name: &str, bound_to: &str) -> Result<Option<String>, String> {
        let stored = Store::open(&self.database)
            .and_then(|store| store.secret(name))
            .map_err(|error| error.to_string())?;
        let Some((nonce, ciphertext)) = stored else {
            return Ok(None);
        };
        Ok(open(&self.cipher()?, &aad(name, bound_to), &nonce, &ciphertext).ok())
    }

    pub fn put(&self, name: &str, bound_to: &str, value: &str) -> Result<(), String> {
        let (nonce, ciphertext) = seal(&self.cipher()?, &aad(name, bound_to), value)?;
        Store::open(&self.database)
            .and_then(|store| store.set_secret(name, &nonce, &ciphertext))
            .map_err(|error| error.to_string())
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        Store::open(&self.database)
            .and_then(|store| store.delete_secret(name))
            .map_err(|error| error.to_string())
    }
}

fn aad(name: &str, bound_to: &str) -> String {
    format!("{name}\n{bound_to}")
}

/// The vault key from the keychain, made on first use.
fn keychain_key() -> Result<Key, String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|error| error.to_string())?;
    match entry.get_secret() {
        Ok(bytes) if bytes.len() == 32 => Ok(*Key::from_slice(&bytes)),
        Ok(_) => Err("The vault key in the keychain is damaged".into()),
        Err(keyring::Error::NoEntry) => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            entry
                .set_secret(&key)
                .map_err(|error| format!("Cannot save the vault key in the keychain: {error}"))?;
            Ok(key)
        }
        Err(error) => Err(format!(
            "Cannot read the vault key from the keychain: {error}"
        )),
    }
}

fn seal(cipher: &ChaCha20Poly1305, aad: &str, value: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: value.as_bytes(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| "Cannot encrypt the secret".to_owned())?;
    Ok((nonce.to_vec(), ciphertext))
}

fn open(
    cipher: &ChaCha20Poly1305,
    aad: &str,
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<String, String> {
    if nonce.len() != 12 {
        return Err("A stored secret is damaged".into());
    }
    let plain = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| "A stored secret cannot be decrypted with this keychain's key".to_owned())?;
    String::from_utf8(plain).map_err(|_| "A stored secret is damaged".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_opens_only_under_its_own_name_address_and_key() {
        let cipher = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));
        let other = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));
        let bound = aad("youtrack-token", "https://tracker.example");

        let (nonce, ciphertext) = seal(&cipher, &bound, "perm:abc").unwrap();

        assert_ne!(ciphertext, b"perm:abc");
        assert_eq!(
            open(&cipher, &bound, &nonce, &ciphertext).unwrap(),
            "perm:abc"
        );
        let moved = aad("youtrack-token", "https://attacker.example");
        assert!(open(&cipher, &moved, &nonce, &ciphertext).is_err());
        assert!(
            open(
                &cipher,
                &aad("other", "https://tracker.example"),
                &nonce,
                &ciphertext
            )
            .is_err()
        );
        assert!(open(&other, &bound, &nonce, &ciphertext).is_err());
    }
}
