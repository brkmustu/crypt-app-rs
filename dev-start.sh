#!/bin/bash

# Renk tanımlamaları
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Eski süreçleri temizle
echo -e "${BLUE}Eski süreçler temizleniyor...${NC}"

# PID dosyalarını kontrol et ve süreçleri sonlandır
for pid_file in /tmp/dev-*.pid; do
    if [ -f "$pid_file" ]; then
        kill $(cat "$pid_file") 2>/dev/null || true
        rm -f "$pid_file"
    fi
done

# Kullanılan portları kontrol et ve süreçleri sonlandır
PORTS=("5173" "8081" "8082" "8083")

for PORT in "${PORTS[@]}"; do
    if sudo lsof -i :"$PORT" >/dev/null 2>&1; then
        echo -e "${BLUE}${PORT} portu kullanımda. Port kapatılıyor...${NC}"
        sudo fuser -k "$PORT"/tcp
        sleep 1
    fi
done

# Tüm süreçlerin tamamen kapanması için kısa bir bekleme
sleep 2

echo -e "${BLUE}Development ortamı başlatılıyor...${NC}\n"

# Servis durumlarını kontrol etmek için fonksiyon
wait_for_service() {
    local port=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1

    echo -e "${BLUE}$service_name için bekleniyor...${NC}"
    while ! nc -z localhost $port; do
        if [ $attempt -eq $max_attempts ]; then
            echo -e "${RED}$service_name başlatılamadı!${NC}"
            exit 1
        fi
        attempt=$((attempt+1))
        sleep 1
    done
    echo -e "${GREEN}$service_name hazır!${NC}"
}

# Frontend için pnpm kontrol
if ! command -v pnpm &> /dev/null; then
    echo -e "${RED}pnpm bulunamadı. Lütfen pnpm'i yükleyin.${NC}"
    echo "npm install -g pnpm"
    exit 1
fi

# Rust servisleri için cargo kontrol
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}cargo bulunamadı. Lütfen Rust'ı yükleyin.${NC}"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# netcat kontrol
if ! command -v nc &> /dev/null; then
    echo -e "${RED}netcat bulunamadı. Lütfen netcat'i yükleyin.${NC}"
    echo "sudo apt-get install netcat # Ubuntu/Debian"
    echo "brew install netcat # macOS"
    exit 1
fi

# Servisleri paralel başlatma
echo -e "${GREEN}Backend servisleri başlatılıyor...${NC}"
(cd services/crypt-gate && RUST_LOG=info cargo run) &
CRYPT_GATE_PID=$!

(cd services/key-gate && RUST_LOG=info cargo run) &
KEY_GATE_PID=$!

(cd services/crypt-processor && RUST_LOG=info cargo run) &
CRYPT_PROCESSOR_PID=$!

# Tüm servislerin hazır olmasını bekle
echo -e "${BLUE}Servislerin hazır olması bekleniyor...${NC}"
wait_services() {
    local all_ready=false
    local attempts=0
    local max_attempts=30

    while [ $attempts -lt $max_attempts ]; do
        local crypt_gate_ready=false
        local key_gate_ready=false
        local crypt_processor_ready=false

        nc -z localhost 8081 && crypt_gate_ready=true
        nc -z localhost 8082 && key_gate_ready=true
        nc -z localhost 8083 && crypt_processor_ready=true

        if $crypt_gate_ready && $key_gate_ready && $crypt_processor_ready; then
            all_ready=true
            break
        fi

        attempts=$((attempts + 1))
        echo -ne "\rBekleniyor... ($attempts/$max_attempts) "
        echo -n "CryptGate: "
        $crypt_gate_ready && echo -n "✓" || echo -n "✗"
        echo -n " KeyGate: "
        $key_gate_ready && echo -n "✓" || echo -n "✗"
        echo -n " CryptProcessor: "
        $crypt_processor_ready && echo -n "✓" || echo -n "✗"
        
        sleep 1
    done
    echo

    if ! $all_ready; then
        echo -e "${RED}Servisler başlatılamadı!${NC}"
        exit 1
    fi

    echo -e "${GREEN}Tüm servisler hazır!${NC}"
}

wait_services

# Frontend başlatma
echo -e "${GREEN}Frontend başlatılıyor...${NC}"
cd frontend && pnpm install && pnpm run watch && pnpm dev &
FRONTEND_PID=$!

# Nginx'i yeniden başlat
echo -e "${GREEN}Nginx yeniden başlatılıyor...${NC}"

# PID'leri kaydet
echo $CRYPT_GATE_PID > /tmp/dev-crypt-gate.pid
echo $KEY_GATE_PID > /tmp/dev-key-gate.pid
echo $CRYPT_PROCESSOR_PID > /tmp/dev-crypt-processor.pid
echo $FRONTEND_PID > /tmp/dev-frontend.pid

echo -e "\n${BLUE}Tüm servisler başlatıldı!${NC}"
echo -e "Frontend: http://localhost:5173"
echo -e "Crypt Gate: http://localhost:8081"
echo -e "Key Gate: http://localhost:8082"
echo -e "Crypt Processor: http://localhost:8083"

# Nginx'i yeniden başlat
# Proje kök dizinini al (bir üst dizine çık çünkü scripts klasöründeyiz)
PROJECT_ROOT=$(pwd)

echo "Nginx yeniden başlatılıyor..."

# Çalışan tüm nginx process'lerini kontrol et ve durdur
if pgrep nginx > /dev/null; then
    echo "Nginx process'leri durduruluyor..."
    sudo systemctl stop nginx
    sudo pkill nginx
    sleep 2
fi

# 80 portunu kontrol et ve temizle
if lsof -Pi :80 -sTCP:LISTEN -t >/dev/null ; then
    echo "80 portu temizleniyor..."
    sudo fuser -k 80/tcp
    sleep 2
fi

# Nginx'in temel ayarlarını kontrol et
echo "Nginx konfigürasyonu test ediliyor..."
sudo nginx -t -c "$PROJECT_ROOT/nginx/nginx.conf"

if [ $? -eq 0 ]; then
    # Yeni konfigürasyon ile nginx'i başlat
    echo "Nginx yeni konfigürasyon ile başlatılıyor..."
    sudo nginx -c "$PROJECT_ROOT/nginx/nginx.conf"

    # Başlatma durumunu kontrol et
    if [ $? -eq 0 ]; then
        echo "Nginx başarıyla başlatıldı!"
        echo "Nginx process'leri:"
        ps aux | grep nginx
    else
        echo "Nginx başlatılırken hata oluştu!"
        exit 1
    fi
else
    echo "Nginx konfigürasyon testi başarısız!"
    exit 1
fi 

# CTRL+C yakalanması
trap 'kill $FRONTEND_PID $CRYPT_GATE_PID $KEY_GATE_PID $CRYPT_PROCESSOR_PID; rm /tmp/dev-*.pid; echo -e "\n${BLUE}Tüm servisler durduruldu${NC}"; exit' INT

# Servislerin çalışmasını bekle
wait