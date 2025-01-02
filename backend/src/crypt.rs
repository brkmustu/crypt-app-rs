use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey};
use rsa::Pkcs1v15Encrypt;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::{thread_rng, RngCore};
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Serialize, Deserialize)]
struct EncryptedData {
    encrypted_key: String,    // RSA ile şifrelenmiş AES anahtarı (base64)
    nonce: String,           // AES nonce (base64)
    data: String,            // AES ile şifrelenmiş veri (base64)
}

pub struct CryptService {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl CryptService {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate private key");
        let public_key = RsaPublicKey::from(&private_key);
        
        Self {
            private_key,
            public_key,
        }
    }

    pub fn get_public_key(&self) -> String {
        self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .expect("failed to encode public key")
    }

    pub fn encrypt_data(&self, data: &str) -> Result<String, String> {
        // AES-256 anahtarı oluştur
        let mut aes_key = [0u8; 32];
        thread_rng().fill_bytes(&mut aes_key);

        // AES nonce oluştur
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);

        // AES anahtarını RSA ile şifrele
        let encrypted_key = self.public_key
            .encrypt(
                &mut thread_rng(),
                Pkcs1v15Encrypt,
                &aes_key
            )
            .map_err(|e| format!("RSA encryption error: {}", e))?;

        // Veriyi AES ile şifrele
        let cipher = Aes256Gcm::new_from_slice(&aes_key)
            .map_err(|e| format!("AES key error: {}", e))?;
        
        let encrypted_data = cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_bytes())
            .map_err(|e| format!("AES encryption error: {}", e))?;

        // Tüm verileri birleştir ve JSON olarak kodla
        let result = EncryptedData {
            encrypted_key: BASE64.encode(encrypted_key),
            nonce: BASE64.encode(nonce),
            data: BASE64.encode(encrypted_data),
        };

        serde_json::to_string(&result)
            .map_err(|e| format!("JSON encoding error: {}", e))
    }

    pub fn decrypt_data(&self, encrypted_json: &str) -> Result<String, String> {
        // JSON'ı parse et
        let encrypted: EncryptedData = serde_json::from_str(encrypted_json)
            .map_err(|e| format!("JSON parsing error: {}", e))?;

        // Base64 decode
        let encrypted_key = BASE64.decode(encrypted.encrypted_key)
            .map_err(|e| format!("Base64 decode error (key): {}", e))?;
        let nonce = BASE64.decode(encrypted.nonce)
            .map_err(|e| format!("Base64 decode error (nonce): {}", e))?;
        let encrypted_data = BASE64.decode(encrypted.data)
            .map_err(|e| format!("Base64 decode error (data): {}", e))?;

        // RSA ile AES anahtarını çöz
        let aes_key = self.private_key
            .decrypt(Pkcs1v15Encrypt, &encrypted_key)
            .map_err(|e| format!("RSA decryption error: {}", e))?;

        // AES ile veriyi çöz
        let cipher = Aes256Gcm::new_from_slice(&aes_key)
            .map_err(|e| format!("AES key error: {}", e))?;

        let decrypted = cipher
            .decrypt(Nonce::from_slice(&nonce), encrypted_data.as_ref())
            .map_err(|e| format!("AES decryption error: {}", e))?;

        String::from_utf8(decrypted)
            .map_err(|_| "Invalid UTF-8 in decrypted data".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        // CryptService oluştur
        let service = CryptService::new();
        
        // Test verisi
        let original_data = "Test mesajı 123!";
        
        // Şifrele
        let encrypted = service.encrypt_data(original_data).unwrap();
        println!("Encrypted: {}", encrypted);
        
        // Çöz
        let decrypted = service.decrypt_data(&encrypted).unwrap();
        println!("Decrypted: {}", decrypted);
        
        // Kontrol et
        assert_eq!(original_data, decrypted);
    }

    #[test]
    fn test_long_message() {
        let service = CryptService::new();
        let long_message = "Bu çok uzun bir mesaj olacak. ".repeat(50);
        
        let encrypted = service.encrypt_data(&long_message).expect("Şifreleme başarısız");
        let decrypted = service.decrypt_data(&encrypted).expect("Çözme başarısız");
        
        assert_eq!(long_message, decrypted);
    }

    #[test]
    fn test_special_chars() {
        let service = CryptService::new();
        let special_chars = "öçşğüıİĞÜŞÇÖ 你好 🌟 !@#$%^&*()";
        
        let encrypted = service.encrypt_data(special_chars).expect("Şifreleme başarısız");
        let decrypted = service.decrypt_data(&encrypted).expect("Çözme başarısız");
        
        assert_eq!(special_chars, decrypted);
    }
}
