use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey,
};
use std::fmt;

use crate::hash::Hash;
use crate::utils::fmt_hex;

/// 65-byte bundle: [recovery_id (1)] + [compact signature (64)].
/// Carrying the recovery ID is what allows us to reconstruct the public key
/// from the signature alone — no need to store the public key in the transaction.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 65]);

#[derive(Clone)]
pub struct PrivateKey(SecretKey);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PublicKey(Secp256k1PublicKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address([u8; 20]);

// ── PrivateKey ────────────────────────────────────────────────────────────────

impl PrivateKey {
    pub fn generate() -> Self {
        Self(SecretKey::new(&mut secp256k1::rand::rng()))
    }

    pub fn public_key(&self) -> PublicKey {
        let secp = Secp256k1::new();
        PublicKey(Secp256k1PublicKey::from_secret_key(&secp, &self.0))
    }

    /// Signs a 32-byte message hash. The resulting Signature embeds the
    /// recovery ID so the public key can be recovered at verify time.
    pub fn sign(&self, message_hash: &[u8; 32]) -> Signature {
        let secp = Secp256k1::new();
        let msg = Message::from_digest(*message_hash);
        let recoverable = secp.sign_ecdsa_recoverable(msg, &self.0);
        let (recovery_id, sig_bytes) = recoverable.serialize_compact();

        let mut bytes = [0u8; 65];
        bytes[0] = i32::from(recovery_id) as u8;
        bytes[1..].copy_from_slice(&sig_bytes);
        Signature(bytes)
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

// ── PublicKey ─────────────────────────────────────────────────────────────────

impl PublicKey {
    /// Derives a 20-byte address by hashing the compressed public key (33 bytes)
    /// with blake3 and taking the first 20 bytes — same approach as Ethereum.
    pub fn to_address(&self) -> Address {
        let compressed = self.0.serialize(); // 33 bytes, compressed form
        let hash = Hash::of(&compressed);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash.as_bytes()[..20]);
        Address(addr)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey(")?;
        fmt_hex(&self.0.serialize(), f)?;
        write!(f, ")")
    }
}

// ── Signature ─────────────────────────────────────────────────────────────────

impl Signature {
    /// Recovers the public key that produced this signature for the given
    /// message hash. Returns None if the bytes are malformed.
    pub fn recover(&self, message_hash: &[u8; 32]) -> Option<PublicKey> {
        let secp = Secp256k1::new();
        let msg = Message::from_digest(*message_hash);
        let recovery_id = RecoveryId::try_from(self.0[0] as i32).ok()?;
        let recoverable = RecoverableSignature::from_compact(&self.0[1..], recovery_id).ok()?;
        secp.recover_ecdsa(msg, &recoverable).ok().map(PublicKey)
    }

    pub fn as_bytes(&self) -> &[u8; 65] {
        &self.0
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature(")?;
        fmt_hex(&self.0, f)?;
        write!(f, ")")
    }
}

// ── Address ───────────────────────────────────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // PrivateKey

    #[test]
    fn private_key_generation() {
        let pk = PrivateKey::generate();
        assert_eq!(pk.public_key().0.serialize().len(), 33);
    }

    #[test]
    fn private_key_debug_redacted() {
        let pk = PrivateKey::generate();
        assert_eq!(format!("{:?}", pk), "PrivateKey([REDACTED])");
    }

    #[test]
    fn same_private_key_same_public_key() {
        let pk = PrivateKey::generate();
        assert_eq!(pk.public_key(), pk.public_key());
    }

    #[test]
    fn different_private_keys_different_public_keys() {
        let pk1 = PrivateKey::generate();
        let pk2 = PrivateKey::generate();
        assert_ne!(pk1.public_key(), pk2.public_key());
    }

    // PublicKey

    #[test]
    fn public_key_to_address_is_20_bytes() {
        let addr = PrivateKey::generate().public_key().to_address();
        assert_eq!(addr.as_bytes().len(), 20);
    }

    #[test]
    fn same_public_key_same_address() {
        let pk = PrivateKey::generate().public_key();
        assert_eq!(pk.to_address(), pk.to_address());
    }

    #[test]
    fn different_public_keys_different_addresses() {
        let addr1 = PrivateKey::generate().public_key().to_address();
        let addr2 = PrivateKey::generate().public_key().to_address();
        assert_ne!(addr1, addr2);
    }

    // Signature / recovery

    #[test]
    fn sign_recover_roundtrip() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let message_hash = Hash::of(b"transfer 50 tokens to Bob");

        let sig = private_key.sign(message_hash.as_bytes());
        let recovered = sig.recover(message_hash.as_bytes()).expect("recovery failed");

        assert_eq!(recovered, public_key);
    }

    #[test]
    fn tampered_message_wrong_key() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();

        let original = Hash::of(b"transfer 50 tokens to Bob");
        let tampered = Hash::of(b"transfer 9999 tokens to Eve");

        let sig = private_key.sign(original.as_bytes());
        let recovered = sig.recover(tampered.as_bytes()).expect("recovery produced a key");

        // Recovery succeeds (it always produces *some* key) but the key won't match
        assert_ne!(recovered, public_key);
    }

    // Address

    #[test]
    fn address_display_is_hex() {
        let addr = PrivateKey::generate().public_key().to_address();
        let s = addr.to_string();
        assert_eq!(s.len(), 40); // 20 bytes × 2 hex chars
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn address_from_bytes_roundtrip() {
        let addr = PrivateKey::generate().public_key().to_address();
        assert_eq!(addr, Address::from_bytes(*addr.as_bytes()));
    }
}
