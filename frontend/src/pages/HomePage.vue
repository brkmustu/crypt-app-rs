<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';
import { useToast } from 'primevue/usetoast';
import { useWebSocket } from '../composables/useWebSocket';
import { CryptService } from '../services/cryptService';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';

const authStore = useAuthStore();
const router = useRouter();
const toast = useToast();
const { connectWebSocket, waitForMessage, disconnect, ensureConnection } = useWebSocket();

const handleLogout = () => {
  authStore.clearToken();
  router.push('/login');
};

const inputMessage = ref('');
const outputMessage = ref('');
const loading = ref(false);

onMounted(() => {
  connectWebSocket();
});

onUnmounted(() => {
  disconnect();
});

const handleEncrypt = async () => {
  if (!inputMessage.value) {
    toast.add({ severity: 'warn', summary: 'Uyarı', detail: 'Lütfen bir mesaj girin', life: 3000 });
    return;
  }

  loading.value = true;
  try {
    await ensureConnection();
    const { message_id } = await CryptService.encrypt(inputMessage.value);
    outputMessage.value = await waitForMessage(message_id);
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
    let encryptedData;
    try {
      encryptedData = JSON.parse(inputMessage.value);
    } catch {
      throw new Error('Geçersiz şifreli mesaj formatı');
    }

    await ensureConnection();
    const { message_id } = await CryptService.decrypt(encryptedData);
    outputMessage.value = await waitForMessage(message_id);

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
  border-radius: 0.5rem !important;
  height: 40px !important;
  padding: 0 1.5rem !important;
}

:deep(.p-inputtext),
:deep(.p-textarea) {
  width: 100%;
  border-radius: 0.5rem !important;
  color: #374151 !important;
  background-color: #fff !important;
  transition: all 0.2s ease-in-out;
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

:deep(.p-button.p-button-primary) {
  background: theme('colors.primary.500') !important;
  border-color: theme('colors.primary.500') !important;
}

:deep(.p-button.p-button-primary:hover) {
  background: theme('colors.primary.600') !important;
  border-color: theme('colors.primary.600') !important;
}

:deep(.p-button.p-button-secondary) {
  background: #fff !important;
  border-color: #e2e8f0 !important;
  color: #64748b !important;
}

:deep(.p-button.p-button-secondary:hover) {
  background: #f8fafc !important;
  border-color: #cbd5e1 !important;
}
</style> 