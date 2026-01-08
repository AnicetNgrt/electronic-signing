#!/bin/bash
set -e

echo "================================================"
echo "SignVault - Full-Stack Development Server"
echo "================================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found${NC}"
    echo "Run ./initial-setup.sh first"
    exit 1
fi

# Load environment variables
set -a
source .env
set +a

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down services...${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
    docker-compose stop postgres 2>/dev/null || true
    echo -e "${GREEN}Services stopped${NC}"
}

trap cleanup EXIT

# Start PostgreSQL with Docker
echo ""
echo -e "${CYAN}Starting PostgreSQL...${NC}"

# Check if postgres container exists and is running
if docker-compose ps postgres 2>/dev/null | grep -q "Up"; then
    echo -e "${GREEN}PostgreSQL already running${NC}"
else
    docker-compose up -d postgres
    echo "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if docker-compose exec -T postgres pg_isready -U "$POSTGRES_USER" > /dev/null 2>&1; then
            echo -e "${GREEN}PostgreSQL ready${NC}"
            break
        fi
        sleep 1
    done
fi

# Run database migrations
echo ""
echo -e "${CYAN}Running database migrations...${NC}"
cd backend
sqlx migrate run || echo -e "${YELLOW}Migrations may already be applied${NC}"
cd ..

# Create storage directory
mkdir -p data/storage

# Start backend with cargo watch
echo ""
echo -e "${CYAN}Starting backend (with hot reload)...${NC}"
cd backend

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

cargo watch -x run &
BACKEND_PID=$!
cd ..

# Wait for backend to start
echo "Waiting for backend to start..."
for i in {1..60}; do
    if curl -s http://localhost:8080/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}Backend ready at http://localhost:8080${NC}"
        break
    fi
    sleep 1
done

# Start frontend
echo ""
echo -e "${CYAN}Starting frontend (with hot reload)...${NC}"
cd frontend
npm run dev &
FRONTEND_PID=$!
cd ..

# Wait for frontend to start
sleep 3
echo -e "${GREEN}Frontend ready at http://localhost:5173${NC}"

echo ""
echo "================================================"
echo -e "${GREEN}Development servers running!${NC}"
echo "================================================"
echo ""
echo "  Frontend:  http://localhost:5173"
echo "  Backend:   http://localhost:8080"
echo "  API:       http://localhost:8080/api"
echo ""
echo "  Default admin credentials:"
echo "    Email:    ${ADMIN_EMAIL:-admin@example.com}"
echo "    Password: ${ADMIN_PASSWORD:-change-this-secure-password}"
echo ""
echo "Press Ctrl+C to stop all services"
echo ""

# Wait for any process to exit
wait
