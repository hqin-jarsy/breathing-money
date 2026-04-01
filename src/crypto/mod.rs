//! Cryptographic primitives: SHA-256 hashing, ML-DSA (Dilithium) post-quantum signatures.
//!
//! This chain uses ML-DSA (CRYSTALS-Dilithium, FIPS 204) from genesis.
//! No elliptic curve cryptography. No secp256k1.
//!
//! On March 30, 2026, Google Quantum AI demonstrated that breaking secp256k1
//! requires only ~1,200 logical qubits — fewer than 500,000 physical qubits
//! on superconducting architectures. This chain does not carry that risk.

use sha2::{Sha256, Digest};
use pqc_dilithium::Keypair;
use serde::{Serialize, Deserialize};

/// 32-byte hash
pub type Hash256 = [u8; 32];

/// Dilithium public key size (ML-DSA-65)
pub const PK_SIZE: usize = 1952;

/// Dilithium signature size (ML-DSA-65)
pub const SIG_SIZE: usize = 3293;

/// Double SHA-256 (Bitcoin-style)
pub fn double_sha256(data: &[u8]) -> Hash256 {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut out = [0u8; 32];
    out.copy_from_slice(&second);
    out
}

/// Single SHA-256
pub fn sha256(data: &[u8]) -> Hash256 {
    let h = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&h);
    out
}

/// Post-quantum keypair wrapper (ML-DSA-65 / Dilithium3)
#[derive(Clone)]
pub struct KeyPairPQ {
    inner: Keypair,
}

impl KeyPairPQ {
    pub fn generate() -> Self {
        KeyPairPQ {
            inner: Keypair::generate(),
        }
    }

    /// Reconstruct from stored key bytes
    /// TODO: proper deserialization when persistent wallet is added
    pub fn from_bytes(_public: &[u8], _secret: &[u8]) -> Self {
        // pqc_dilithium::Keypair doesn't expose from_bytes in 0.2
        // For prototype: generate fresh keypair as placeholder
        KeyPairPQ {
            inner: Keypair::generate(),
        }
    }

    /// Sign a message
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.inner.sign(msg).to_vec()
    }

    /// Public key bytes (1952 bytes for ML-DSA-65)
    pub fn pubkey_bytes(&self) -> Vec<u8> {
        self.inner.public.to_vec()
    }

    /// Address: SHA-256 of public key, truncated to 20 bytes
    /// Version byte 0x02 prefix indicates Dilithium address
    pub fn address(&self) -> Address {
        let h = sha256(&self.inner.public);
        let mut addr = [0u8; 21];
        addr[0] = ADDR_VERSION_DILITHIUM;
        addr[1..].copy_from_slice(&h[..20]);
        Address(addr)
    }
}

/// Address version bytes — algorithm agility from genesis
pub const ADDR_VERSION_DILITHIUM: u8 = 0x02;

/// 21-byte address: 1 version byte + 20 hash bytes
/// Version byte enables future signature scheme upgrades via soft fork
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Address(pub [u8; 21]);

impl Address {
    pub fn zero() -> Self {
        Address([0u8; 21])
    }

    /// The genesis seal burn address — version 0x00 + all 0xDE bytes
    pub fn genesis_seal() -> Self {
        let mut addr = [0xDE; 21];
        addr[0] = 0x00; // version 0 = burn address
        Address(addr)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 21 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut addr = [0u8; 21];
        addr.copy_from_slice(&bytes);
        Ok(Address(addr))
    }

    /// Get the version byte
    pub fn version(&self) -> u8 {
        self.0[0]
    }
}

/// Verify a Dilithium signature
pub fn verify_signature(pubkey: &[u8], msg: &[u8], sig: &[u8]) -> bool {
    pqc_dilithium::verify(sig, msg, pubkey).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_sha256() {
        let h = double_sha256(b"breathing money");
        assert_ne!(h, [0u8; 32]);
    }

    #[test]
    fn test_keypair_sign_verify() {
        let kp = KeyPairPQ::generate();
        let msg = b"test message";
        let sig = kp.sign(msg);
        let pk = kp.pubkey_bytes();
        assert!(verify_signature(&pk, msg, &sig));
        // Tampered message should fail
        assert!(!verify_signature(&pk, b"wrong message", &sig));
    }

    #[test]
    fn test_genesis_seal_address() {
        let addr = Address::genesis_seal();
        assert_eq!(addr.version(), 0x00);
    }

    #[test]
    fn test_address_version() {
        let kp = KeyPairPQ::generate();
        let addr = kp.address();
        assert_eq!(addr.version(), ADDR_VERSION_DILITHIUM);
    }

    #[test]
    fn test_dilithium_sizes() {
        let kp = KeyPairPQ::generate();
        assert_eq!(kp.pubkey_bytes().len(), PK_SIZE);
        let sig = kp.sign(b"test");
        assert_eq!(sig.len(), SIG_SIZE);
    }
}
