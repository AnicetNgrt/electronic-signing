#!/bin/bash
set -e

echo "================================================"
echo "SignVault - Code Quality Checks"
echo "================================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

FAILED=0
SKIPPED=0

# Function to run a check
run_check() {
    local name="$1"
    local cmd="$2"
    local dir="$3"

    echo ""
    echo -e "${YELLOW}Running: $name${NC}"

    if [ -n "$dir" ]; then
        pushd "$dir" > /dev/null
    fi

    if eval "$cmd"; then
        echo -e "${GREEN}$name passed${NC}"
    else
        echo -e "${RED}$name failed${NC}"
        FAILED=1
    fi

    if [ -n "$dir" ]; then
        popd > /dev/null
    fi
}

echo ""
echo "=== Backend Checks (Rust) ==="

# Check if Rust toolchain is available
if command -v cargo &> /dev/null; then
    # Rust formatting check
    run_check "Rust Format Check" "cargo fmt -- --check" "backend"

    # Rust clippy (linting)
    run_check "Rust Clippy" "cargo clippy --all-targets --all-features -- -D warnings" "backend"

    # Rust type check / build
    run_check "Rust Build" "cargo build" "backend"

    # Rust unit tests
    run_check "Rust Unit Tests" "cargo test --lib" "backend"
else
    echo ""
    echo -e "${CYAN}Skipping Rust checks (cargo not found in PATH)${NC}"
    echo -e "${CYAN}Install Rust with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    SKIPPED=1
fi

echo ""
echo "=== Frontend Checks (TypeScript/React) ==="

# Install deps if needed
if [ ! -d "frontend/node_modules" ]; then
    echo "Installing frontend dependencies..."
    (cd frontend && npm install)
fi

# TypeScript type check
run_check "TypeScript Type Check" "npm run typecheck" "frontend"

# ESLint
run_check "ESLint" "npm run lint" "frontend"

# Frontend build
run_check "Frontend Build" "npm run build" "frontend"

# Frontend unit tests
run_check "Frontend Unit Tests" "npm run test -- --run" "frontend"

echo ""
echo "================================================"
if [ $FAILED -eq 0 ]; then
    if [ $SKIPPED -eq 1 ]; then
        echo -e "${GREEN}All available checks passed!${NC}"
        echo -e "${CYAN}(Some checks were skipped due to missing toolchain)${NC}"
    else
        echo -e "${GREEN}All checks passed!${NC}"
    fi
    exit 0
else
    echo -e "${RED}Some checks failed!${NC}"
    exit 1
fi
