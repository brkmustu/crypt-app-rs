#!/bin/bash

# Renk tanımlamaları
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

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

# Crypt Gate başlatma
echo -e "${GREEN}Crypt Gate servisi başlatılıyor...${NC}"
(cd services && cd crypt-gate && RUST_LOG=info cargo run) &
CRYPT_GATE_PID=$!
wait_for_service 8081 "Crypt Gate"

# Key Gate başlatma
echo -e "${GREEN}Key Gate servisi başlatılıyor...${NC}"
(cd services && cd key-gate && RUST_LOG=info cargo run) &
KEY_GATE_PID=$!
wait_for_service 8082 "Key Gate"

# Crypt Processor başlatma
echo -e "${GREEN}Crypt Processor servisi başlatılıyor...${NC}"
(cd services && cd crypt-processor && RUST_LOG=info cargo run) &
CRYPT_PROCESSOR_PID=$!
wait_for_service 8083 "Crypt Processor"

# Frontend başlatma
echo -e "${GREEN}Frontend başlatılıyor...${NC}"
cd frontend && pnpm install && pnpm run watch && pnpm dev &
FRONTEND_PID=$!

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

# CTRL+C yakalanması
trap 'kill $FRONTEND_PID $CRYPT_GATE_PID $KEY_GATE_PID $CRYPT_PROCESSOR_PID; rm /tmp/dev-*.pid; echo -e "\n${BLUE}Tüm servisler durduruldu${NC}"; exit' INT

# Servislerin çalışmasını bekle
wait 