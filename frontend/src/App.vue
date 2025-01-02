<script setup>
import { onMounted } from 'vue';

onMounted(() => {
  document.documentElement.style.margin = '0';
  document.documentElement.style.padding = '0';
  document.body.style.margin = '0';
  document.body.style.padding = '0';
});
</script>

<template>
  <Toast position="top-right" />
  <router-view />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
}

/* Toast stillerini özelleştirme */
.p-toast {
  opacity: 0.95;
}

.p-toast-message {
  background: #1e1e1e !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  backdrop-filter: blur(8px);
}

.p-toast-message-content {
  padding: 1rem !important;
}

.p-toast-message-text {
  color: rgba(255, 255, 255, 0.9) !important;
}

.p-toast-message-icon,
.p-toast-icon-close {
  color: rgba(255, 255, 255, 0.7) !important;
}

.p-toast-icon-close:hover {
  color: rgba(255, 255, 255, 0.9) !important;
}

/* Toast tipleri için özel renkler */
.p-toast-message-error {
  background: #2d1a1a !important;
  border-left: 6px solid #ef4444 !important;
}

.p-toast-message-success {
  background: #1a2d1a !important;
  border-left: 6px solid #22c55e !important;
}
</style>

