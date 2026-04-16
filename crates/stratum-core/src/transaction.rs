use crate::{
    hash::Hash,
    key::{Address, PrivateKey, Signature},
};

#[derive(Clone, Debug)]
pub struct Transaction {
    pub nonce: u64,
    pub from: Address,
    pub to: Address,
    pub value: u64,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub signature: Signature,
}

impl Transaction {
    fn compute_signing_hash(
        nonce: u64,
        from: &Address,
        to: &Address,
        value: u64,
        data: &[u8],
        gas_limit: u64,
        gas_price: u64,
    ) -> Hash {
        let mut buf = Vec::new();
        buf.extend_from_slice(&nonce.to_le_bytes());
        buf.extend_from_slice(from.as_bytes());
        buf.extend_from_slice(to.as_bytes());
        buf.extend_from_slice(&value.to_le_bytes());
        buf.extend_from_slice(&(data.len() as u64).to_le_bytes());
        buf.extend_from_slice(data);
        buf.extend_from_slice(&gas_limit.to_le_bytes());
        buf.extend_from_slice(&gas_price.to_le_bytes());
        Hash::of(&buf)
    }

    pub fn sign(
        key: &PrivateKey,
        nonce: u64,
        to: Address,
        value: u64,
        data: Vec<u8>,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        let from = key.public_key().to_address();
        let message_hash =
            Self::compute_signing_hash(nonce, &from, &to, value, &data, gas_limit, gas_price);
        let signature = key.sign(message_hash.as_bytes());
        Self {
            nonce,
            from,
            to,
            value,
            data,
            gas_limit,
            gas_price,
            signature,
        }
    }

    pub fn verify(&self) -> bool {
        let message_hash = Self::compute_signing_hash(
            self.nonce,
            &self.from,
            &self.to,
            self.value,
            &self.data,
            self.gas_limit,
            self.gas_price,
        );
        match self.signature.recover(message_hash.as_bytes()) {
            Some(pk) => pk.to_address() == self.from,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(key: &PrivateKey) -> Transaction {
        let to = PrivateKey::generate().public_key().to_address();
        Transaction::sign(key, 1, to, 100, vec![], 21_000, 1)
    }

    #[test]
    fn sign_transaction() {
        let from_key = PrivateKey::generate();
        let transaction = make_tx(&from_key);
        assert!(transaction.verify());
        assert_eq!(transaction.from, from_key.public_key().to_address());
    }

    #[test]
    fn two_signers_two_addresses() {
        let from_key = PrivateKey::generate();
        let another_key = PrivateKey::generate();
        let transaction_1 = make_tx(&from_key);
        let transaction_2 = make_tx(&another_key);
        assert_ne!(transaction_1.from, transaction_2.from);
    }

    #[test]
    fn tampered_value_fails_verify() {
        let from_key = PrivateKey::generate();
        let mut transaction = make_tx(&from_key);
        transaction.value = 2;
        assert!(!transaction.verify())
    }

    #[test]
    fn tampered_nonce_fails_verify() {
        let from_key = PrivateKey::generate();
        let mut transaction = make_tx(&from_key);
        transaction.nonce = 2;
        assert!(!transaction.verify())
    }

    #[test]
    fn tampered_from_fails_verify() {
        let from_key = PrivateKey::generate();
        let another_key = PrivateKey::generate();
        let mut transaction = make_tx(&from_key);
        transaction.from = another_key.public_key().to_address();
        assert!(!transaction.verify())
    }
}
