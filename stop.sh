#!/bin/bash

# Stop local DFX replica and clean up lingering processes
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

info "Stopping dfx local replica (if running)..."
if command -v dfx >/dev/null 2>&1; then
  dfx stop || true
else
  error "dfx not found in PATH"
fi

# Give processes a moment to exit
sleep 1

# Kill any lingering processes on the default local port 4943
PORT=4943
PIDS=$(lsof -ti tcp:${PORT} || true)
if [ -n "${PIDS}" ]; then
  info "Killing lingering processes on port ${PORT}: ${PIDS}"
  kill ${PIDS} || true
  sleep 1
  # Force kill if still alive
  STILL=$(lsof -ti tcp:${PORT} || true)
  if [ -n "${STILL}" ]; then
    info "Force killing remaining processes: ${STILL}"
    kill -9 ${STILL} || true
  fi
else
  info "No processes listening on port ${PORT}"
fi

success "DFX local replica stopped and port ${PORT} is free." 
