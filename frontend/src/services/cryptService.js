import { useAuthStore } from '../stores/auth';

export class CryptService {
  static async encrypt(message) {
    const authStore = useAuthStore();
    const response = await fetch('http://localhost:8081/encrypt', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authStore.token}`
      },
      body: JSON.stringify(message)
    });

    if (!response.ok) {
      throw new Error('Şifreleme başarısız');
    }

    return response.json();
  }

  static async decrypt(encryptedData) {
    const authStore = useAuthStore();
    const response = await fetch('http://localhost:8081/decrypt', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authStore.token}`
      },
      body: JSON.stringify(encryptedData)
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Şifre çözme başarısız');
    }

    return response.json();
  }
} 