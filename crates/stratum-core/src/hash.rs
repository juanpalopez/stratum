use core::fmt;

use crate::utils::fmt_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn of(data: &[u8]) -> Self {
        Self::from_bytes(*blake3::hash(data).as_bytes())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_hex(self.as_bytes(), f)
    }
}

pub trait Hashable {
    fn hash(&self) -> Hash;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_hash() {
        let h1 = Hash::of(b"stratum");
        let h2 = Hash::of(b"stratum");
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_input_different_hash() {
        let h1 = Hash::of(b"stratum");
        let h2 = Hash::of(b"stratum!");
        assert_ne!(h1, h2);
    }

    #[test]
    fn display_is_hex() {
        let h = Hash::of(b"stratum");
        let s = h.to_string();
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
