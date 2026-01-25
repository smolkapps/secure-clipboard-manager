// Encryption module for sensitive clipboard data
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::fs;
use std::path::PathBuf;

const NONCE_SIZE: usize = 12; // 96 bits for ChaCha20Poly1305

pub struct Encryptor {
    cipher: ChaCha20Poly1305,
}

impl Encryptor {
    /// Create a new encryptor with a master key
    /// The key is stored securely in the user's data directory
    pub fn new(key_path: PathBuf) -> Result<Self, String> {
        let key = Self::load_or_create_key(key_path)?;
        let cipher = ChaCha20Poly1305::new(&key);
        Ok(Encryptor { cipher })
    }

    /// Load existing key or create a new one
    fn load_or_create_key(key_path: PathBuf) -> Result<chacha20poly1305::Key, String> {
        // Ensure parent directory exists
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create key directory: {}", e))?;
        }

        if key_path.exists() {
            // Load existing key
            let key_bytes = fs::read(&key_path)
                .map_err(|e| format!("Failed to read encryption key: {}", e))?;

            if key_bytes.len() != 32 {
                return Err("Invalid key length".to_string());
            }

            let mut key = chacha20poly1305::Key::default();
            key.copy_from_slice(&key_bytes);
            Ok(key)
        } else {
            // Generate new key
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);

            // Save key with restricted permissions (macOS will set 0600 by default for new files)
            fs::write(&key_path, &key)
                .map_err(|e| format!("Failed to save encryption key: {}", e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&key_path)
                    .map_err(|e| format!("Failed to get key file metadata: {}", e))?
                    .permissions();
                perms.set_mode(0o600); // Owner read/write only
                fs::set_permissions(&key_path, perms)
                    .map_err(|e| format!("Failed to set key file permissions: {}", e))?;
            }

            log::info!("🔑 Generated new encryption key at: {}", key_path.display());
            Ok(key)
        }
    }

    /// Encrypt data and return [nonce || ciphertext]
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data from [nonce || ciphertext]
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, String> {
        if encrypted.len() < NONCE_SIZE {
            return Err("Invalid encrypted data: too short".to_string());
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");
        let encryptor = Encryptor::new(key_path).unwrap();

        let plaintext = b"Secret API key: sk-1234567890abcdef";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // Encrypted data should be different
        assert_ne!(&encrypted[NONCE_SIZE..], plaintext);

        // Should be able to decrypt
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_nonces() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");
        let encryptor = Encryptor::new(key_path).unwrap();

        let plaintext = b"Same plaintext";
        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext due to different nonces
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt correctly
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_key_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");

        // Create first encryptor (generates key)
        let encryptor1 = Encryptor::new(key_path.clone()).unwrap();
        let plaintext = b"Test data";
        let encrypted = encryptor1.encrypt(plaintext).unwrap();

        // Create second encryptor (loads existing key)
        let encryptor2 = Encryptor::new(key_path).unwrap();

        // Should be able to decrypt with second encryptor
        let decrypted = encryptor2.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");
        let encryptor = Encryptor::new(key_path).unwrap();

        // Too short
        assert!(encryptor.decrypt(&[1, 2, 3]).is_err());

        // Invalid ciphertext
        let mut bad_data = vec![0u8; NONCE_SIZE + 16];
        OsRng.fill_bytes(&mut bad_data);
        assert!(encryptor.decrypt(&bad_data).is_err());
    }
}
