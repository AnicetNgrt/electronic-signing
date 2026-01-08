#!/bin/bash
set -e

echo "================================================"
echo "SignVault - Initial Setup"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check for required tools
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        exit 1
    fi
    echo -e "${GREEN}Found $1${NC}"
}

echo ""
echo "Checking required tools..."
check_tool "cargo"
check_tool "rustc"
check_tool "node"
check_tool "npm"
check_tool "docker"
check_tool "docker-compose"

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo -e "${GREEN}Rust version: $RUST_VERSION${NC}"

# Check Node version
NODE_VERSION=$(node --version)
echo -e "${GREEN}Node version: $NODE_VERSION${NC}"

echo ""
echo "Setting up environment..."

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env

    # Generate a random JWT secret
    JWT_SECRET=$(openssl rand -base64 32 | tr -d '\n')
    sed -i "s/change-this-to-a-secure-random-string-min-32-chars/$JWT_SECRET/" .env

    echo -e "${GREEN}.env file created with a secure JWT secret${NC}"
    echo -e "${YELLOW}Please review and update .env with your configuration${NC}"
else
    echo -e "${YELLOW}.env file already exists, skipping...${NC}"
fi

# Create data directories
echo ""
echo "Creating data directories..."
mkdir -p data/storage
mkdir -p data/postgres
echo -e "${GREEN}Data directories created${NC}"

# Install backend dependencies
echo ""
echo "Setting up Rust backend..."
cd backend
cargo fetch
echo -e "${GREEN}Rust dependencies fetched${NC}"
cd ..

# Install frontend dependencies
echo ""
echo "Setting up React frontend..."
cd frontend
npm install
echo -e "${GREEN}Frontend dependencies installed${NC}"
cd ..

# Create sample PDF for testing
echo ""
echo "Creating test fixtures..."
mkdir -p backend/tests/fixtures
if [ ! -f backend/tests/fixtures/sample.pdf ]; then
    # Create a minimal PDF file for testing
    cat > backend/tests/fixtures/sample.pdf << 'EOF'
%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << >> >>
endobj
4 0 obj
<< /Length 44 >>
stream
BT
/F1 12 Tf
100 700 Td
(Test Document) Tj
ET
endstream
endobj
xref
0 5
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000214 00000 n
trailer
<< /Size 5 /Root 1 0 R >>
startxref
306
%%EOF
EOF
    echo -e "${GREEN}Sample PDF created${NC}"
fi

# Install sqlx-cli for migrations
echo ""
echo "Installing sqlx-cli..."
cargo install sqlx-cli --no-default-features --features postgres || echo -e "${YELLOW}sqlx-cli may already be installed${NC}"

echo ""
echo "================================================"
echo -e "${GREEN}Setup complete!${NC}"
echo "================================================"
echo ""
echo "Next steps:"
echo "1. Review and configure .env file"
echo "2. Start services with: docker-compose up -d"
echo "3. Run database migrations with: cd backend && sqlx migrate run"
echo "4. Start development with: ./start-fullstack-dev-watch.sh"
echo ""
echo "Or use Docker Compose for everything:"
echo "  docker-compose up --build"
echo ""
