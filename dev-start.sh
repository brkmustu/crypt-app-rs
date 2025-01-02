#!/bin/bash

# Renk tanımlamaları
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Development ortamı başlatılıyor...${NC}\n"

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

# Frontend başlatma
echo -e "${GREEN}Frontend başlatılıyor...${NC}"
cd frontend && pnpm install && pnpm run watch && pnpm dev &
FRONTEND_PID=$!

# Crypt Gate başlatma
echo -e "${GREEN}Crypt Gate servisi başlatılıyor...${NC}"
(cd services && cd crypt-gate && RUST_LOG=info cargo run) &
CRYPT_GATE_PID=$!

# Key Gate başlatma
echo -e "${GREEN}Key Gate servisi başlatılıyor...${NC}"
(cd services && cd key-gate && RUST_LOG=info cargo run) &
KEY_GATE_PID=$!

# PID'leri kaydet
echo $FRONTEND_PID > /tmp/dev-frontend.pid
echo $CRYPT_GATE_PID > /tmp/dev-crypt-gate.pid
echo $KEY_GATE_PID > /tmp/dev-key-gate.pid

echo -e "\n${BLUE}Tüm servisler başlatıldı!${NC}"
echo -e "Frontend: http://localhost:5173"
echo -e "Crypt Gate: http://localhost:8081"
echo -e "Key Gate: http://localhost:8082"

# CTRL+C yakalanması
trap 'kill $FRONTEND_PID $CRYPT_GATE_PID $KEY_GATE_PID; rm /tmp/dev-*.pid; echo -e "\n${BLUE}Tüm servisler durduruldu${NC}"; exit' INT

# Servislerin çalışmasını bekle
wait 