use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use base64::Engine;
use md5::{Digest, Md5};

fn derive_key(password: &str) -> [u8; 16] {
    let digest = Md5::digest(password.as_bytes());
    let mut key = [0u8; 16];
    key.copy_from_slice(&digest[..16]);
    key
}

fn derive_key_kotlin(password: &str) -> [u8; 16] {
    let digest = Md5::digest(password.as_bytes());
    let hex = format!("{digest:x}");
    let mut key = [0u8; 16];
    key.copy_from_slice(&hex.as_bytes()[..16]);
    key
}

fn encrypt_with_key(data: &[u8], key: [u8; 16]) -> Vec<u8> {
    let cipher = Aes128::new_from_slice(&key).expect("valid aes key");
    let pad = 16 - (data.len() % 16);
    let mut buf = Vec::with_capacity(data.len() + pad);
    buf.extend_from_slice(data);
    buf.extend(std::iter::repeat(pad as u8).take(pad));
    for chunk in buf.chunks_mut(16) {
        cipher.encrypt_block(chunk.into());
    }
    buf
}

fn decrypt_with_key(data: &[u8], key: [u8; 16]) -> Option<Vec<u8>> {
    if data.is_empty() || data.len() % 16 != 0 {
        return None;
    }
    let cipher = Aes128::new_from_slice(&key).ok()?;
    let mut buf = data.to_vec();
    for chunk in buf.chunks_mut(16) {
        cipher.decrypt_block(chunk.into());
    }
    let pad = *buf.last()? as usize;
    if pad == 0 || pad > 16 || pad > buf.len() {
        return None;
    }
    buf.truncate(buf.len() - pad);
    Some(buf)
}

pub fn encrypt(data: &[u8], password: &str) -> Vec<u8> {
    let key = derive_key(password);
    let cipher = Aes128::new_from_slice(&key).expect("valid aes key");
    let pad = 16 - (data.len() % 16);
    let mut buf = Vec::with_capacity(data.len() + pad);
    buf.extend_from_slice(data);
    buf.extend(std::iter::repeat(pad as u8).take(pad));
    for chunk in buf.chunks_mut(16) {
        cipher.encrypt_block(chunk.into());
    }
    buf
}

pub fn decrypt(data: &[u8], password: &str) -> Option<Vec<u8>> {
    if data.is_empty() || data.len() % 16 != 0 {
        return None;
    }
    let key = derive_key(password);
    let cipher = Aes128::new_from_slice(&key).ok()?;
    let mut buf = data.to_vec();
    for chunk in buf.chunks_mut(16) {
        cipher.decrypt_block(chunk.into());
    }
    let pad = *buf.last()? as usize;
    if pad == 0 || pad > 16 || pad > buf.len() {
        return None;
    }
    buf.truncate(buf.len() - pad);
    Some(buf)
}

pub fn encrypt_kotlin_base64(data: &[u8], password: &str) -> String {
    let encrypted = encrypt_with_key(data, derive_key_kotlin(password));
    base64::engine::general_purpose::STANDARD.encode(encrypted)
}

pub fn decrypt_kotlin_base64(text: &str, password: &str) -> Option<Vec<u8>> {
    let data = base64::engine::general_purpose::STANDARD.decode(text).ok()?;
    decrypt_with_key(&data, derive_key_kotlin(password))
}

pub fn decrypt_any(text: &str, password: &str) -> Option<Vec<u8>> {
    if let Some(bytes) = from_hex(text) {
        if let Some(plain) = decrypt(&bytes, password) {
            return Some(plain);
        }
    }
    decrypt_kotlin_base64(text, password)
}

pub fn to_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

pub fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let high = (bytes[i] as char).to_digit(16)?;
        let low = (bytes[i + 1] as char).to_digit(16)?;
        out.push(((high << 4) | low) as u8);
        i += 2;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_round_trip() {
        let data = b"hello legado backup password";
        let encrypted = encrypt(data, "123456");
        assert_ne!(encrypted, data);
        assert_eq!(decrypt(&encrypted, "123456").unwrap(), data);
    }

    #[test]
    fn hex_round_trip() {
        let data = vec![0, 1, 15, 16, 255];
        assert_eq!(from_hex(&to_hex(&data)).unwrap(), data);
    }
}
