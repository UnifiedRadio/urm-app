// ECDH key exchange and AES-128-CBC session encryption for the Xiaomi Lite BLE protocol.
//
// Protocol assumption derived from blelib/utils/a.java behaviour:
//   Curve       : P-256 (secp256r1); uncompressed public key = 65 bytes (0x04 || x || y)
//   Shared secret: raw 32-byte x-coordinate of ECDH output point
//   Transport key: shared_secret[0..16] → AES-128 key; shared_secret[16..32] → IV
//                  Used to decrypt the encrypted_session_data blob in LoginResponse.
//   Session key  : extracted from decrypted LoginResponse payload:
//                  plaintext[0..16] = session AES key
//                  plaintext[16..32] = session AES IV
//                  plaintext[32..] = token (stored for re-login)
//   All subsequent encrypted commands (20003, 20007, 20012) use the session AES-128-CBC key+IV.

use anyhow::{bail, Result};
use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use p256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};
use rand_core::OsRng;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

// ── ECDH keypair ──────────────────────────────────────────────────────────────

pub struct EcdhKeypair {
    secret: EphemeralSecret,
    /// Uncompressed SEC1 public key (65 bytes: 0x04 || x || y).
    pub public_key_bytes: Vec<u8>,
}

impl EcdhKeypair {
    pub fn generate() -> Self {
        let secret = EphemeralSecret::random(&mut OsRng);
        let public_key_bytes = secret.public_key().to_encoded_point(false).as_bytes().to_vec();
        Self { secret, public_key_bytes }
    }

    /// Derive a `TransportKey` from the device's SEC1 public key bytes.
    /// Consumes `self` because `EphemeralSecret` is single-use.
    pub fn derive_transport_key(self, peer_sec1_bytes: &[u8]) -> Result<TransportKey> {
        let peer = PublicKey::from_sec1_bytes(peer_sec1_bytes)
            .map_err(|e| anyhow::anyhow!("invalid device public key: {e}"))?;
        let shared = self.secret.diffie_hellman(&peer);
        let raw = shared.raw_secret_bytes();
        if raw.len() < 32 {
            bail!("ECDH shared secret too short: {} bytes", raw.len());
        }
        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        key.copy_from_slice(&raw[..16]);
        iv.copy_from_slice(&raw[16..32]);
        Ok(TransportKey { key, iv })
    }
}

// ── Transport key (ECDH-derived, decrypts LoginResponse) ─────────────────────

pub struct TransportKey {
    key: [u8; 16],
    iv: [u8; 16],
}

impl TransportKey {
    pub fn decrypt_session(&self, ciphertext: &[u8]) -> Result<(SessionKey, Vec<u8>)> {
        let plaintext = aes128_cbc_decrypt(&self.key, &self.iv, ciphertext)?;
        SessionKey::from_plaintext(&plaintext)
    }
}

// ── Session key (used for all subsequent encrypted commands) ──────────────────

#[derive(Clone)]
pub struct SessionKey {
    key: [u8; 16],
    iv: [u8; 16],
}

impl SessionKey {
    /// Parse session key from the decrypted LoginResponse payload.
    /// Layout: aes_key[16] || aes_iv[16] || token[..]
    fn from_plaintext(plaintext: &[u8]) -> Result<(Self, Vec<u8>)> {
        if plaintext.len() < 32 {
            bail!("decrypted LoginResponse too short: {} bytes", plaintext.len());
        }
        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        key.copy_from_slice(&plaintext[..16]);
        iv.copy_from_slice(&plaintext[16..32]);
        let token = plaintext[32..].to_vec();
        Ok((Self { key, iv }, token))
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        aes128_cbc_encrypt(&self.key, &self.iv, plaintext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        aes128_cbc_decrypt(&self.key, &self.iv, ciphertext)
    }
}

// ── AES-128-CBC helpers ───────────────────────────────────────────────────────

fn aes128_cbc_decrypt(key: &[u8; 16], iv: &[u8; 16], ciphertext: &[u8]) -> Result<Vec<u8>> {
    Aes128CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| anyhow::anyhow!("AES-128-CBC decrypt: {e:?}"))
}

fn aes128_cbc_encrypt(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    Aes128CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}
