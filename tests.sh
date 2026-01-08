#!/bin/bash
set -e

echo "================================================"
echo "SignVault - Test Suite"
echo "================================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Parse arguments
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_E2E=true
KEEP_SERVICES=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            RUN_INTEGRATION=false
            RUN_E2E=false
            shift
            ;;
        --integration-only)
            RUN_UNIT=false
            RUN_E2E=false
            shift
            ;;
        --e2e-only)
            RUN_UNIT=false
            RUN_INTEGRATION=false
            shift
            ;;
        --keep-services)
            KEEP_SERVICES=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Cleanup function
cleanup() {
    if [ "$KEEP_SERVICES" = false ]; then
        echo ""
        echo -e "${YELLOW}Cleaning up test services...${NC}"
        docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true
    fi
}

trap cleanup EXIT

FAILED=0

# Unit Tests
if [ "$RUN_UNIT" = true ]; then
    echo ""
    echo -e "${CYAN}=== Running Unit Tests ===${NC}"

    echo ""
    echo -e "${YELLOW}Backend unit tests...${NC}"
    cd backend
    if cargo test --lib; then
        echo -e "${GREEN}Backend unit tests passed${NC}"
    else
        echo -e "${RED}Backend unit tests failed${NC}"
        FAILED=1
    fi
    cd ..

    echo ""
    echo -e "${YELLOW}Frontend unit tests...${NC}"
    cd frontend
    if npm run test -- --run; then
        echo -e "${GREEN}Frontend unit tests passed${NC}"
    else
        echo -e "${RED}Frontend unit tests failed${NC}"
        FAILED=1
    fi
    cd ..
fi

# Integration Tests
if [ "$RUN_INTEGRATION" = true ]; then
    echo ""
    echo -e "${CYAN}=== Running Integration Tests ===${NC}"

    # Start test database
    echo -e "${YELLOW}Starting test database...${NC}"

    # Create test docker-compose if it doesn't exist
    if [ ! -f docker-compose.test.yml ]; then
        cat > docker-compose.test.yml << 'EOF'
version: '3.8'

services:
  test-db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: signvault_test
      POSTGRES_PASSWORD: test_secret
      POSTGRES_DB: signvault_test
    ports:
      - "5433:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U signvault_test"]
      interval: 5s
      timeout: 5s
      retries: 5
EOF
    fi

    docker-compose -f docker-compose.test.yml up -d

    # Wait for database
    echo "Waiting for test database..."
    for i in {1..30}; do
        if docker-compose -f docker-compose.test.yml exec -T test-db pg_isready -U signvault_test > /dev/null 2>&1; then
            echo -e "${GREEN}Test database ready${NC}"
            break
        fi
        sleep 1
    done

    # Run migrations
    echo "Running migrations..."
    export DATABASE_URL="postgres://signvault_test:test_secret@localhost:5433/signvault_test"
    cd backend
    sqlx migrate run || echo -e "${YELLOW}Migrations may already be applied${NC}"
    cd ..

    # Start backend in background for integration tests
    echo "Starting backend for integration tests..."
    export JWT_SECRET="test-jwt-secret-for-integration-tests"
    export ADMIN_EMAIL="admin@example.com"
    export ADMIN_PASSWORD="change-this-secure-password"
    export STORAGE_PATH="./data/test-storage"
    export PUBLIC_URL="http://localhost:5173"

    mkdir -p data/test-storage

    cd backend
    cargo build --release

    # Start the server in background
    ./target/release/signvault &
    BACKEND_PID=$!
    cd ..

    # Wait for backend
    echo "Waiting for backend to start..."
    for i in {1..30}; do
        if curl -s http://localhost:8080/api/health > /dev/null 2>&1; then
            echo -e "${GREEN}Backend ready${NC}"
            break
        fi
        sleep 1
    done

    # Run integration tests
    echo ""
    echo -e "${YELLOW}Running backend integration tests...${NC}"
    cd backend
    if cargo test --test integration_tests; then
        echo -e "${GREEN}Integration tests passed${NC}"
    else
        echo -e "${RED}Integration tests failed${NC}"
        FAILED=1
    fi
    cd ..

    # Stop backend
    kill $BACKEND_PID 2>/dev/null || true
fi

# E2E Tests
if [ "$RUN_E2E" = true ]; then
    echo ""
    echo -e "${CYAN}=== Running E2E Tests ===${NC}"

    cd frontend

    # Install Playwright browsers if needed
    if [ ! -d "node_modules/.cache/ms-playwright" ]; then
        echo "Installing Playwright browsers..."
        npx playwright install --with-deps chromium
    fi

    # Run E2E tests
    echo -e "${YELLOW}Running Playwright E2E tests...${NC}"
    if npm run test:e2e; then
        echo -e "${GREEN}E2E tests passed${NC}"
    else
        echo -e "${RED}E2E tests failed${NC}"
        FAILED=1
    fi
    cd ..
fi

echo ""
echo "================================================"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
