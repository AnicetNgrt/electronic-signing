#!/bin/bash
# Script to generate SQLx query metadata for offline compilation
# This must be run before Docker builds

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "================================================"
echo "SignVault - Prepare SQLx Offline Metadata"
echo "================================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

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

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Check if PostgreSQL is running
echo ""
echo -e "${YELLOW}Ensuring PostgreSQL is running...${NC}"

if docker-compose ps postgres 2>/dev/null | grep -q "Up"; then
    echo -e "${GREEN}PostgreSQL already running${NC}"
else
    echo "Starting PostgreSQL..."
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

# Run migrations to ensure schema is up to date
echo ""
echo -e "${YELLOW}Running database migrations...${NC}"
cd backend
sqlx migrate run
cd ..

# Generate SQLx query metadata
echo ""
echo -e "${YELLOW}Generating SQLx query metadata...${NC}"
cd backend
cargo sqlx prepare

if [ -d ".sqlx" ]; then
    echo -e "${GREEN}SQLx metadata generated successfully in backend/.sqlx${NC}"
else
    echo -e "${RED}Failed to generate SQLx metadata${NC}"
    exit 1
fi

cd ..

echo ""
echo "================================================"
echo -e "${GREEN}SQLx preparation complete!${NC}"
echo "================================================"
echo ""
echo "You can now build the Docker image:"
echo "  docker-compose build backend"
echo ""
