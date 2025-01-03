import { ref } from 'vue';

export function useWebSocket() {
  const ws = ref(null);
  const messageCallbacks = new Map();

  const connectWebSocket = () => {
    if (ws.value?.readyState === WebSocket.OPEN) {
      console.log('WebSocket zaten bağlı');
      return;
    }

    console.log('WebSocket bağlantısı kuruluyor...');
    ws.value = new WebSocket('ws://localhost:8083/ws');
    
    ws.value.onopen = () => {
      console.log('WebSocket bağlantısı açıldı');
    };

    ws.value.onmessage = handleWebSocketMessage;
    ws.value.onclose = handleWebSocketClose;
    ws.value.onerror = (error) => {
      console.error('WebSocket hatası:', error);
    };
  };

  const handleWebSocketMessage = (event) => {
    try {
      const response = JSON.parse(event.data);
      console.log('WebSocket mesajı alındı:', response);
      
      const callback = messageCallbacks.get(response.message_id);
      if (callback) {
        console.log('Callback çalıştırılıyor, message_id:', response.message_id);
        callback({
          success: response.success,
          data: response.data,
          error: response.error
        });
        messageCallbacks.delete(response.message_id);
      }
    } catch (error) {
      console.error('WebSocket mesaj işleme hatası:', error, 'Raw data:', event.data);
    }
  };

  const handleWebSocketClose = (event) => {
    console.log('WebSocket bağlantısı kapandı, kod:', event.code, 'neden:', event.reason);
    if (!event.wasClean) {
      console.log('Bağlantı beklenmedik şekilde kapandı, yeniden bağlanılıyor...');
      setTimeout(connectWebSocket, 1000);
    }
  };

  const waitForMessage = async (messageId, timeout = 30000) => {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        messageCallbacks.delete(messageId);
        reject(new Error('İşlem zaman aşımına uğradı'));
      }, timeout);

      messageCallbacks.set(messageId, (result) => {
        clearTimeout(timeoutId);
        if (result.error) {
          reject(new Error(result.error));
        } else {
          resolve(result.data);
        }
      });
    });
  };

  const disconnect = () => {
    if (ws.value) {
      ws.value.onclose = null;
      ws.value.close();
      ws.value = null;
    }
  };

  return {
    connectWebSocket,
    waitForMessage,
    disconnect,
    ensureConnection: async () => {
      if (!ws.value || ws.value.readyState !== WebSocket.OPEN) {
        connectWebSocket();
        await new Promise((resolve, reject) => {
          const timeout = setTimeout(() => {
            reject(new Error('WebSocket bağlantı zaman aşımı'));
          }, 5000);

          const checkConnection = setInterval(() => {
            if (ws.value?.readyState === WebSocket.OPEN) {
              clearInterval(checkConnection);
              clearTimeout(timeout);
              resolve();
            }
          }, 100);
        });
      }
    }
  };
} 