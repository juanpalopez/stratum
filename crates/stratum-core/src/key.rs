use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fmt;

use crate::hash::Hash;
use crate::utils::fmt_hex;

#[derive(Clone)]
pub struct PrivateKey([u8; 32]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicKey([u8; 32]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address([u8; 20]);

impl PrivateKey {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self(signing_key.to_bytes())
    }

    pub fn public_key(&self) -> PublicKey {
        let signing_key = SigningKey::from_bytes(&self.0);
        PublicKey(signing_key.verifying_key().to_bytes())
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

impl PublicKey {
    pub fn to_address(&self) -> Address {
        let hash = Hash::of(&self.0);
        let mut truncated = [0u8; 20];
        truncated.copy_from_slice(&hash.as_bytes()[..20]);
        Address(truncated)
    }
}

impl Address {
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_hex(self.as_bytes(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //PrivateKey
    #[test]
    fn private_key_generation() {
        let private_key = PrivateKey::generate();
        assert_eq!(private_key.0.len(), 32);
        assert_eq!(private_key.public_key().0.len(), 32);
    }

    #[test]
    fn private_key_debug() {
        let private_key = PrivateKey::generate();
        let debug_pk = format!("{:?}", private_key);
        assert_eq!(debug_pk, "PrivateKey([REDACTED])");
    }

    #[test]
    fn private_key_same_public_key() {
        let private_key = PrivateKey::generate();
        let public_1 = private_key.public_key();
        let public_2 = private_key.public_key();
        assert_eq!(public_1, public_2);
    }

    #[test]
    fn two_private_keys_two_public_keys() {
        let private_1 = PrivateKey::generate();
        let private_2 = PrivateKey::generate();
        assert_ne!(private_1.public_key(), private_2.public_key());
    }

    // PublicKey
    #[test]
    fn public_key_to_address() {
        let private_key = PrivateKey::generate();
        assert_eq!(private_key.public_key().to_address().0.len(), 20);
    }

    #[test]
    fn public_key_same_address() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let address_1 = public_key.to_address();
        let address_2 = public_key.to_address();
        assert_eq!(address_1, address_2);
    }

    #[test]
    fn two_public_keys_two_addresses() {
        let private_1 = PrivateKey::generate();
        let public_1 = private_1.public_key();
        let address_1 = public_1.to_address();

        let private_2 = PrivateKey::generate();
        let public_2 = private_2.public_key();
        let address_2 = public_2.to_address();

        assert_ne!(address_1, address_2);
    }

    // Address
    #[test]
    fn address_display_is_hex() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let address = public_key.to_address();
        let s = address.to_string();
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn address_from_bytes() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let address_1 = public_key.to_address();
        let addres_2 = Address::from_bytes(*address_1.as_bytes());
        assert_eq!(address_1, addres_2);
    }
}
