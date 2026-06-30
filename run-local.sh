    #!/usr/bin/env bash

    # Cab Logger - Bare Metal Local Development Script
    # High-contrast visual CLI to compile and run both Fastify (DuckDB) and Leptos (WASM) locally.

    set -euo pipefail

    # Text Colors
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color

    # Print banner
    echo -e "${YELLOW}${BOLD}"
    echo "=========================================================="
    echo "    ██████╗ █████╗ ██████╗  ██╗      ██████╗  ██████╗      "
    echo "   ██╔════╝██╔══██╗██╔══██╗ ██║     ██╔═══██╗██╔════╝      "
    echo "   ██║     ███████║██████╔╝ ██║     ██║   ██║██║  ███╗     "
    echo "   ██║     ██╔══██║██╔══██╗ ██║     ██║   ██║██║   ██║     "
    echo "   ╚██████╗██║  ██║██████╔╝ ███████╗╚██████╔╝╚██████╔╝     "
    echo "    ╚══════╝╚═╝  ╚═╝╚═════╝  ╚══════╝ ╚═════╝  ╚═════╝      "
    echo "              D I S P A T C H    D E S K                  "
    echo "=========================================================="
    echo -e "${NC}"

    echo -e "${BLUE}[*] Initializing bare-metal execution script...${NC}"

    # Check for required tools
    echo -e "\n${BOLD}Checking system prerequisites...${NC}"

    # 1. Node.js & NPM (for backend)
    if ! command -v node &> /dev/null; then
        echo -e "${RED}[ERROR] Node.js is not installed. Please install Node.js (v18+) to run the backend.${NC}"
        exit 1
    else
        echo -e "${GREEN}[✔] Node.js: $(node -v)${NC}"
    fi

    if ! command -v npm &> /dev/null; then
        echo -e "${RED}[ERROR] npm is not installed. Please install npm to resolve dependencies.${NC}"
        exit 1
    else
        echo -e "${GREEN}[✔] npm: $(npm -v)${NC}"
    fi

    # 2. Rust Toolchain (for frontend)
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}[ERROR] Cargo / Rust toolchain is not found. Please install Rust from https://rustup.rs/${NC}"
        exit 1
    else
        echo -e "${GREEN}[✔] Cargo / Rust: $(cargo --version)${NC}"
    fi

    # 3. WASM Target
    echo -e "${BLUE}[*] Verifying wasm32-unknown-unknown target...${NC}"
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        echo -e "${YELLOW}[!] wasm32-unknown-unknown target not found. Installing...${NC}"
        rustup target add wasm32-unknown-unknown
        echo -e "${GREEN}[✔] Target wasm32-unknown-unknown successfully added.${NC}"
    else
        echo -e "${GREEN}[✔] Target wasm32-unknown-unknown is active.${NC}"
    fi

    # 4. Trunk WASM Bundler
    if ! command -v trunk &> /dev/null; then
        echo -e "${YELLOW}[!] Trunk WASM builder is not installed.${NC}"
        echo -e "${CYAN}Would you like to install trunk now? (using cargo install trunk) [y/N]${NC}"
        read -r response
        if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
            echo -e "${BLUE}[*] Installing Trunk (this might take a few minutes)...${NC}"
            # Do not pass --locked: trunk 0.21.x locks libdeflate-sys 1.23.1, which fails on GCC 16+.
            cargo install trunk
            echo -e "${GREEN}[✔] Trunk installed successfully.${NC}"
        else
            echo -e "${RED}[ERROR] Trunk is required to build/serve the Leptos frontend. Run 'cargo install trunk' manually.${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}[✔] Trunk bundler is available: $(trunk --version)${NC}"
    fi

    # Apply M1/Apple Silicon safety environment flag
    # This resolves "failed to find intrinsics to enable clone_ref" errors during WebAssembly compilation
    ARCH=$(uname -m)
    OS=$(uname -s)
    if [[ "$ARCH" == "arm64" || "$OS" == "Darwin" ]]; then
        echo -e "${CYAN}[*] Apple Silicon / macOS detected. Injecting reference-types workaround for wasm-bindgen...${NC}"
        export RUSTFLAGS="-C target-feature=-reference-types"
        echo -e "${GREEN}[✔] RUSTFLAGS is set to: $RUSTFLAGS${NC}"
    fi

    # Setup backend dependencies
    echo -e "\n${BOLD}Setting up backend workspace...${NC}"
    if [ ! -d "backend/node_modules" ]; then
        echo -e "${YELLOW}[!] Node modules not found for backend. Installing...${NC}"
        (cd backend && npm install)
        echo -e "${GREEN}[✔] Backend dependencies installed.${NC}"
    else
        echo -e "${GREEN}[✔] Backend dependencies already configured.${NC}"
    fi

    # Keep background processes tracking
    BACKEND_PID=""
    TAIL_PID=""

    cleanup() {
        echo -e "\n\n${YELLOW}[*] Gracefully shutting down local processes...${NC}"
        if [ -n "$BACKEND_PID" ]; then
            echo -e "${BLUE}[*] Stopping backend API server (PID: $BACKEND_PID)...${NC}"
            kill "$BACKEND_PID" 2>/dev/null || true
        fi
        if [ -n "$TAIL_PID" ]; then
            echo -e "${BLUE}[*] Stopping log tailer...${NC}"
            kill "$TAIL_PID" 2>/dev/null || true
        fi
        echo -e "${GREEN}[✔] Done! Dispatch Desk closed.${NC}"
        exit 0
    }

    # Trap terminal exits (Ctrl+C, SIGHUP, SIGTERM)
    trap cleanup SIGINT SIGTERM SIGHUP EXIT

    # Create data directory for DuckDB if not present
    mkdir -p backend/data

    # Clean up any orphaned processes on target ports to prevent port clashes and DuckDB file lock errors
    echo -e "${BLUE}[*] Cleaning up any orphaned processes on port 8095 (backend) and 8090 (frontend)...${NC}"
    if command -v lsof >/dev/null 2>&1; then
        lsof -t -i:8095 | xargs kill -9 2>/dev/null || true
        lsof -t -i:8090 | xargs kill -9 2>/dev/null || true
    elif command -v fuser >/dev/null 2>&1; then
        fuser -k 8095/tcp >/dev/null 2>&1 || true
        fuser -k 8090/tcp >/dev/null 2>&1 || true
    fi

    # Run backend Fastify service
    echo -e "\n${BOLD}Starting Fastify API backend with DuckDB...${NC}"
    cd backend
    # Redirect logs to backend.log so they do not clutter trunk's output, but are preserved for diagnosis
    touch backend.log
    npm run dev > backend.log 2>&1 &
    BACKEND_PID=$!
    # Tail backend logs in the background with a prefix
    tail -f backend.log | sed 's/^/[BACKEND] /' &
    TAIL_PID=$!
    cd ..

    # Verify backend starts up successfully and does not crash immediately
    echo -e "${BLUE}[*] Verifying backend startup stability...${NC}"
    sleep 4
    if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
        echo -e "${RED}[ERROR] Backend service crashed immediately upon startup!${NC}"
        echo -e "${YELLOW}--- Last 25 lines of backend/backend.log ---${NC}"
        if [ -f "backend/backend.log" ]; then
            tail -n 25 backend/backend.log
        else
            echo "No log file found."
        fi
        echo -e "${YELLOW}--------------------------------------------${NC}"
        echo -e "${RED}Please resolve the database or node issue above and try again.${NC}"
        exit 1
    else
        echo -e "${GREEN}[✔] Backend service is running (PID: $BACKEND_PID). Logs are saved to backend/backend.log${NC}"
    fi

    # Run frontend Trunk service
    echo -e "\n${BOLD}Starting Leptos WASM Frontend via Trunk...${NC}"
    echo -e "${BLUE}[*] Serves UI on http://localhost:8090${NC}"
    echo -e "${BLUE}[*] Frontend calls Fastify directly at http://localhost:8095${NC}"
    echo -e "${YELLOW}[!] Press Ctrl+C at any time to gracefully terminate both servers.${NC}\n"

    cd frontend
    trunk serve
