<script setup>
import { ref } from 'vue';
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';
import { useToast } from 'primevue/usetoast';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';

const authStore = useAuthStore();
const router = useRouter();
const toast = useToast();

const inputMessage = ref('');
const outputMessage = ref('');
const loading = ref(false);

const handleLogout = () => {
  authStore.clearToken();
  router.push('/login');
};

const handleEncrypt = async () => {
  if (!inputMessage.value) {
    toast.add({ severity: 'warn', summary: 'Uyarı', detail: 'Lütfen bir mesaj girin', life: 3000 });
    return;
  }

  loading.value = true;
  try {
    // Şifreleme isteği gönder
    const response = await fetch('http://localhost:8081/encrypt', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authStore.token}`
      },
      body: JSON.stringify({
        data: inputMessage.value  // Direkt mesajı gönder
      })
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Şifreleme başarısız');
    }
    
    const data = await response.json();
    outputMessage.value = data.result;
    toast.add({ severity: 'success', summary: 'Başarılı', detail: 'Mesaj şifrelendi', life: 3000 });
  } catch (error) {
    console.error('Şifreleme hatası:', error);
    toast.add({ 
      severity: 'error', 
      summary: 'Hata', 
      detail: error.message || 'Şifreleme işlemi başarısız', 
      life: 3000 
    });
  } finally {
    loading.value = false;
  }
};

const handleDecrypt = async () => {
  if (!inputMessage.value) {
    toast.add({ severity: 'warn', summary: 'Uyarı', detail: 'Lütfen şifreli mesaj girin', life: 3000 });
    return;
  }

  loading.value = true;
  try {
    // Şifre çözme isteği gönder
    const response = await fetch('http://localhost:8081/decrypt', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authStore.token}`
      },
      body: JSON.stringify({
        data: inputMessage.value  // Direkt şifreli metni gönder
      })
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Şifre çözme başarısız');
    }
    
    const data = await response.json();
    outputMessage.value = data.result;
    toast.add({ severity: 'success', summary: 'Başarılı', detail: 'Mesaj çözüldü', life: 3000 });
  } catch (error) {
    console.error('Şifre çözme hatası:', error);
    toast.add({ 
      severity: 'error', 
      summary: 'Hata', 
      detail: error.message || 'Şifre çözme işlemi başarısız', 
      life: 3000 
    });
  } finally {
    loading.value = false;
  }
};

const copyToClipboard = async () => {
  if (!outputMessage.value) {
    toast.add({ severity: 'warn', summary: 'Uyarı', detail: 'Kopyalanacak mesaj yok', life: 3000 });
    return;
  }

  try {
    await navigator.clipboard.writeText(outputMessage.value);
    toast.add({ severity: 'success', summary: 'Başarılı', detail: 'Mesaj kopyalandı', life: 3000 });
  } catch (error) {
    toast.add({ severity: 'error', summary: 'Hata', detail: 'Kopyalama başarısız', life: 3000 });
  }
};
</script>

<template>
  <div class="min-h-screen bg-gray-100">
    <!-- Navbar -->
    <nav class="bg-white shadow-sm">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex items-center">
            <h1 class="text-xl font-semibold text-gray-900">Şifreleme Paneli</h1>
          </div>
          <div class="flex items-center">
            <Button
              @click="handleLogout"
              severity="danger"
              label="Çıkış Yap"
              icon="pi pi-power-off"
              class="p-button-rounded"
            />
          </div>
        </div>
      </div>
    </nav>

    <!-- Main Content -->
    <main class="max-w-4xl mx-auto py-6 sm:px-6 lg:px-8">
      <div class="bg-white rounded-lg shadow p-6 space-y-6">
        <!-- Input Section -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700">
            Mesaj
          </label>
          <Textarea
            v-model="inputMessage"
            rows="4"
            placeholder="Şifrelemek veya çözmek istediğiniz mesajı girin..."
            class="w-full"
            :disabled="loading"
          />
        </div>

        <!-- Action Buttons -->
        <div class="flex justify-center space-x-4">
          <Button
            @click="handleEncrypt"
            :loading="loading"
            label="Şifrele"
            icon="pi pi-lock"
            class="p-button-rounded p-button-primary"
          />
          <Button
            @click="handleDecrypt"
            :loading="loading"
            label="Şifreyi Çöz"
            icon="pi pi-unlock"
            class="p-button-rounded p-button-secondary"
          />
        </div>

        <!-- Output Section -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700">
            Sonuç
          </label>
          <div class="relative">
            <Textarea
              v-model="outputMessage"
              rows="4"
              readonly
              placeholder="Sonuç burada görünecek..."
              class="w-full"
            />
            <Button
              v-if="outputMessage"
              @click="copyToClipboard"
              label="Kopyala"
              icon="pi pi-copy"
              class="p-button-rounded p-button-secondary absolute top-2 right-2"
              v-tooltip.top="'Kopyala'"
            />
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.p-button {
  margin-left: 0.5rem;
}

:deep(.p-inputtext),
:deep(.p-textarea) {
  width: 100%;
  border-radius: 0.5rem;
  color: #374151 !important;
  background-color: #fff !important;
}

:deep(.p-textarea):disabled {
  opacity: 0.7;
  cursor: not-allowed;
  background-color: #f3f4f6 !important;
}

:deep(.p-textarea)::placeholder {
  color: #9ca3af !important;
}

:deep(.p-textarea[readonly]) {
  background-color: #f8fafc !important;
  border-color: #e2e8f0 !important;
}
</style> 