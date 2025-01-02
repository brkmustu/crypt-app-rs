<script setup>
import { ref } from 'vue';
import { useToast } from 'primevue/usetoast';
import InputText from 'primevue/inputtext';
import Password from 'primevue/password';
import Button from 'primevue/button';
import Card from 'primevue/card';
import Cookies from 'js-cookie';
import { useAuthStore } from '../stores/auth'
import { useRouter } from 'vue-router'

const toast = useToast();
const username = ref('');
const password = ref('');
const loading = ref(false);
const authStore = useAuthStore()
const router = useRouter()

const handleLogin = async () => {
    if (!username.value || !password.value) {
        toast.add({ severity: 'error', summary: 'Hata', detail: 'Lütfen tüm alanları doldurun', life: 3000 });
        return;
    }

    loading.value = true;
    try {
        const success = await authStore.login(username.value, password.value);
        
        if (success) {
            toast.add({ severity: 'success', summary: 'Başarılı', detail: 'Giriş yapıldı', life: 3000 });
            router.push('/');
        } else {
            throw new Error('Login failed');
        }
    } catch (error) {
        console.error('Login error:', error);
        toast.add({ 
            severity: 'error', 
            summary: 'Hata', 
            detail: 'Kullanıcı adı veya şifre hatalı', 
            life: 3000 
        });
    } finally {
        loading.value = false;
    }
};
</script>

<template>
  <div class="overflow-hidden">
    <!-- Background Image Container -->
    <div 
      class="absolute inset-0 z-0"
      style="background: url('https://images.unsplash.com/photo-1734597949864-0ee6637b0c3f?q=80&w=1934&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D?auto=format&fit=crop&w=2000&q=80') center/cover no-repeat"
    >
      <!-- Overlay -->
      <div class="absolute inset-0 bg-gradient-to-b from-[rgba(0,0,0,0.2)] to-[rgba(76,29,29,0.4)]"></div>
    </div>

    <!-- Content Container -->
    <div class="relative z-10 min-h-screen w-full flex flex-col items-center justify-center px-4">
      <!-- Logo ve Form Container -->
      <div class="flex flex-col items-center justify-center -mt-20">
        <!-- Logo -->
        <div class="mb-20">
          <div class="w-20 h-20 relative">
            <div class="absolute -top-3 left-1/2 transform -translate-x-1/2">
              <div class="w-6 h-3 bg-white/10 rounded-t-full"></div>
            </div>
            <div class="w-full h-full bg-white/10 backdrop-blur-sm flex items-center justify-center rounded-xl">
              <span class="text-4xl font-bold text-white">N</span>
            </div>
          </div>
        </div>

        <!-- Form Container -->
        <div class="w-full max-w-[400px] space-y-5">
          <div class="relative">
            <i class="pi pi-user absolute left-6 top-1/2 -translate-y-1/2 text-white/40 text-xl z-10"></i>
            <InputText
              v-model="username"
              placeholder="User Name..."
              class="w-full"
              :class="{'p-invalid': !username && loading}"
            />
          </div>
          
          <div class="relative">
            <i class="pi pi-lock absolute left-6 top-1/2 -translate-y-1/2 text-white/40 text-xl z-10"></i>
            <InputText
              v-model="password"
              type="password"
              placeholder="Password..."
              class="w-full"
              :class="{'p-invalid': !password && loading}"
              @keyup.enter="handleLogin"
            />
          </div>

          <Button
            type="submit"
            :loading="loading"
            label="Login"
            class="w-full !mt-10"
            @click="handleLogin"
          />

          <div class="flex justify-between text-[13px] text-primary-500 pt-6 font-semibold tracking-wide">
            <a href="#" class="hover:text-primary-600 transition-colors">CREATE ACCOUNT</a>
            <a href="#" class="hover:text-primary-600 transition-colors">NEED HELP?</a>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
/* :deep yerine global stiller kullanalım */
.p-inputtext {
  background: rgba(255, 255, 255, 0.05) !important;
  backdrop-filter: blur(8px) !important;
  border: none !important;
  border-radius: 24px !important;
  color: white !important;
  height: 60px !important;
  padding: 0 2rem 0 4rem !important;
  font-size: 1rem !important;
  letter-spacing: 0.025em !important;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05) !important;
  width: 100% !important;
  transform: translateY(0);
  transition: all 0.3s ease !important;

  &::placeholder {
    color: rgba(255, 255, 255, 0.4) !important;
    letter-spacing: 0.025em !important;
  }

  &:hover {
    background: rgba(255, 255, 255, 0.1) !important;
    transform: translateY(-2px) !important;
    box-shadow: 0 8px 12px -1px rgba(0, 0, 0, 0.1) !important;
  }

  &:focus {
    background: rgba(255, 255, 255, 0.1) !important;
    box-shadow: none !important;
    outline: none !important;
    transform: translateY(0) !important;
  }

  /* Input dolu olduğunda arka plan rengini değiştir */
  &:not(:placeholder-shown) {
    background: rgba(255, 255, 255, 0.15) !important;
  }

  &.p-invalid {
    border: 1px solid rgba(248, 113, 113, 0.5) !important;
  }
}

.p-button {
  background: #ff3e1d !important;
  border: none !important;
  border-radius: 9999px !important;
  height: 60px !important;
  transition: all 0.3s !important;
  font-size: 1rem !important;
  letter-spacing: 0.025em !important;
  font-weight: 400 !important;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1) !important;

  &:hover {
    background: #ff2b03 !important;
    transform: translateY(-2px) !important;
    box-shadow: 0 15px 20px -3px rgba(0, 0, 0, 0.15) !important;
  }

  &:active {
    transform: translateY(0) !important;
  }

  &:disabled {
    opacity: 0.7 !important;
  }

  .p-button-label {
    font-weight: 400 !important;
    letter-spacing: 0.025em !important;
  }
}
</style>

