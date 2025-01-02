import { defineStore } from 'pinia'
import { ref } from 'vue'
import Cookies from 'js-cookie'
import { JSEncrypt } from 'jsencrypt'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(Cookies.get('auth_token') || null)
  const publicKey = ref(Cookies.get('public_key') || null)
  const isAuthenticated = ref(!!token.value)
  let jsEncrypt = new JSEncrypt()

  // Public key varsa JSEncrypt'e set et
  if (publicKey.value) {
    jsEncrypt.setPublicKey(publicKey.value)
  }

  async function login(username, password) {
    try {
      const response = await fetch('http://localhost:8082/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      })

      const data = await response.json()

      if (!response.ok) {
        throw new Error(data.error || 'Login failed')
      }

      if (!data.token || !data.public_key) {
        throw new Error('Invalid server response')
      }

      setToken(data.token)
      await setPublicKey(data.public_key)
      return true
    } catch (error) {
      console.error('Login error:', error)
      return false
    }
  }

  function setToken(newToken) {
    token.value = newToken
    isAuthenticated.value = true
    Cookies.set('auth_token', newToken, {
      expires: 1,
      secure: true,
      sameSite: 'strict',
      path: '/'
    })
  }

  function setPublicKey(key) {
    console.log('Received key:', key);
    
    try {
      // URL-encoded public key'i decode et
      const decodedKey = decodeURIComponent(key);
      console.log('Decoded key:', decodedKey);

      // Yeni bir JSEncrypt instance'ı oluştur
      const cryptInstance = new JSEncrypt();
      
      // Public key'i doğrudan set et
      cryptInstance.setPublicKey(decodedKey);

      // Test şifreleme
      const testResult = cryptInstance.encrypt('test');
      if (!testResult) {
        throw new Error('Encryption test failed');
      }

      // Başarılı ise değerleri güncelle
      publicKey.value = decodedKey;
      jsEncrypt = cryptInstance;
      
      // Cookie'ye kaydet
      Cookies.set('public_key', decodedKey, {
        expires: 1,
        secure: true,
        sameSite: 'strict',
        path: '/'
      });

    } catch (error) {
      console.error('Public key processing error:', error);
      throw new Error(`Invalid public key format: ${error.message}`);
    }
  }

  function encryptData(data) {
    if (!publicKey.value) {
      throw new Error('Public key not set')
    }
    return jsEncrypt.encrypt(data)
  }

  function clearToken() {
    token.value = null
    isAuthenticated.value = false
    publicKey.value = null
    Cookies.remove('auth_token')
    Cookies.remove('public_key')
  }

  async function encryptWithPublicKey(data) {
    if (!publicKey.value) {
      throw new Error('Public key not set');
    }

    try {
      // Yeni bir JSEncrypt instance'ı oluştur
      const cryptInstance = new JSEncrypt();
      
      // Public key'i set et
      cryptInstance.setPublicKey(publicKey.value);
      
      // Veriyi RSA ile şifrele
      const encrypted = cryptInstance.encrypt(data);
      if (!encrypted) {
        throw new Error('Encryption failed');
      }

      return encrypted; // JSEncrypt otomatik olarak base64 formatında döndürür
    } catch (error) {
      console.error('Encryption error:', error);
      throw new Error(`RSA encryption failed: ${error.message}`);
    }
  }

  return {
    token,
    isAuthenticated,
    publicKey,
    login,
    setToken,
    setPublicKey,
    clearToken,
    encryptData,
    encryptWithPublicKey
  }
}) 