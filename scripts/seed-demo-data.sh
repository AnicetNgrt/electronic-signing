#!/bin/bash
# Script to seed demo data for development and testing
# Creates sample documents and signers to demonstrate the application

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "================================================"
echo "SignVault - Seed Demo Data"
echo "================================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Load environment variables
if [ -f .env ]; then
    set -a
    source .env
    set +a
fi

API_URL="${API_URL:-http://localhost:8080/api}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-change-this-secure-password}"

# Check if API is available
echo ""
echo -e "${CYAN}Checking API availability...${NC}"
if ! curl -s "${API_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}Error: API not available at ${API_URL}${NC}"
    echo "Make sure the backend is running first."
    exit 1
fi
echo -e "${GREEN}API is available${NC}"

# Login to get auth token
echo ""
echo -e "${CYAN}Logging in as admin...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\": \"${ADMIN_EMAIL}\", \"password\": \"${ADMIN_PASSWORD}\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo -e "${RED}Error: Failed to login${NC}"
    echo "Response: $LOGIN_RESPONSE"
    exit 1
fi
echo -e "${GREEN}Logged in successfully${NC}"

# Create a sample PDF file for uploads
echo ""
echo -e "${CYAN}Creating sample PDF files...${NC}"
mkdir -p /tmp/signvault-demo

# Employment Agreement PDF
cat > /tmp/signvault-demo/employment-agreement.pdf << 'PDFEOF'
%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>
endobj
4 0 obj
<< /Length 256 >>
stream
BT
/F1 18 Tf
72 700 Td
(Employment Agreement) Tj
0 -30 Td
/F1 12 Tf
(This Employment Agreement is entered into between) Tj
0 -20 Td
(the Company and the Employee on the date signed below.) Tj
0 -40 Td
(Terms and conditions apply as discussed.) Tj
0 -60 Td
(Employee Signature: ____________________) Tj
0 -30 Td
(Date: ____________________) Tj
0 -60 Td
(Employer Signature: ____________________) Tj
0 -30 Td
(Date: ____________________) Tj
ET
endstream
endobj
5 0 obj
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000266 00000 n
0000000574 00000 n
trailer
<< /Size 6 /Root 1 0 R >>
startxref
651
%%EOF
PDFEOF

# NDA PDF
cat > /tmp/signvault-demo/nda.pdf << 'PDFEOF'
%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>
endobj
4 0 obj
<< /Length 300 >>
stream
BT
/F1 18 Tf
72 700 Td
(Non-Disclosure Agreement) Tj
0 -30 Td
/F1 12 Tf
(This Non-Disclosure Agreement protects confidential) Tj
0 -20 Td
(information shared between the parties.) Tj
0 -40 Td
(The receiving party agrees to maintain confidentiality) Tj
0 -20 Td
(of all proprietary information disclosed.) Tj
0 -60 Td
(Disclosing Party: ____________________) Tj
0 -30 Td
(Receiving Party: ____________________) Tj
0 -30 Td
(Date: ____________________) Tj
ET
endstream
endobj
5 0 obj
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000266 00000 n
0000000618 00000 n
trailer
<< /Size 6 /Root 1 0 R >>
startxref
695
%%EOF
PDFEOF

# Service Contract PDF
cat > /tmp/signvault-demo/service-contract.pdf << 'PDFEOF'
%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>
endobj
4 0 obj
<< /Length 280 >>
stream
BT
/F1 18 Tf
72 700 Td
(Service Contract) Tj
0 -30 Td
/F1 12 Tf
(This Service Contract outlines the terms of service) Tj
0 -20 Td
(to be provided by the Service Provider to the Client.) Tj
0 -40 Td
(Payment terms: Net 30 days from invoice date.) Tj
0 -60 Td
(Client Signature: ____________________) Tj
0 -30 Td
(Date: ____________________) Tj
0 -60 Td
(Provider Signature: ____________________) Tj
0 -30 Td
(Date: ____________________) Tj
ET
endstream
endobj
5 0 obj
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000266 00000 n
0000000598 00000 n
trailer
<< /Size 6 /Root 1 0 R >>
startxref
675
%%EOF
PDFEOF

echo -e "${GREEN}Sample PDF files created${NC}"

# Function to upload document and get ID
upload_document() {
    local title="$1"
    local file="$2"
    local self_sign="$3"

    RESPONSE=$(curl -s -X POST "${API_URL}/documents" \
        -H "Authorization: Bearer ${TOKEN}" \
        -F "title=${title}" \
        -F "self_sign_only=${self_sign}" \
        -F "file=@${file}")

    echo "$RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4
}

# Function to add signer
add_signer() {
    local doc_id="$1"
    local email="$2"
    local name="$3"

    RESPONSE=$(curl -s -X POST "${API_URL}/documents/${doc_id}/signers" \
        -H "Authorization: Bearer ${TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"email\": \"${email}\", \"name\": \"${name}\"}")

    echo "$RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4
}

# Function to add field
add_field() {
    local doc_id="$1"
    local field_type="$2"
    local page="$3"
    local x="$4"
    local y="$5"
    local width="$6"
    local height="$7"
    local signer_id="$8"

    local body="{\"field_type\": \"${field_type}\", \"page\": ${page}, \"x\": ${x}, \"y\": ${y}, \"width\": ${width}, \"height\": ${height}"

    if [ -n "$signer_id" ]; then
        body="${body}, \"signer_id\": \"${signer_id}\""
    fi

    body="${body}}"

    curl -s -X POST "${API_URL}/documents/${doc_id}/fields" \
        -H "Authorization: Bearer ${TOKEN}" \
        -H "Content-Type: application/json" \
        -d "$body" > /dev/null
}

echo ""
echo -e "${CYAN}Creating demo documents...${NC}"

# Document 1: Employment Agreement (multi-signer)
echo -e "${YELLOW}Creating Employment Agreement...${NC}"
DOC1_ID=$(upload_document "Employment Agreement - John Smith" "/tmp/signvault-demo/employment-agreement.pdf" "false")
if [ -n "$DOC1_ID" ]; then
    SIGNER1_ID=$(add_signer "$DOC1_ID" "john.smith@example.com" "John Smith")
    SIGNER2_ID=$(add_signer "$DOC1_ID" "hr@company.com" "HR Department")

    # Employee signature field
    add_field "$DOC1_ID" "signature" 1 100 420 200 50 "$SIGNER1_ID"
    add_field "$DOC1_ID" "date" 1 100 380 120 25 "$SIGNER1_ID"

    # Employer signature field
    add_field "$DOC1_ID" "signature" 1 100 300 200 50 "$SIGNER2_ID"
    add_field "$DOC1_ID" "date" 1 100 260 120 25 "$SIGNER2_ID"

    echo -e "${GREEN}  Created with 2 signers (ID: ${DOC1_ID})${NC}"
else
    echo -e "${RED}  Failed to create${NC}"
fi

# Document 2: NDA (multi-signer)
echo -e "${YELLOW}Creating NDA...${NC}"
DOC2_ID=$(upload_document "NDA - Acme Corp Partnership" "/tmp/signvault-demo/nda.pdf" "false")
if [ -n "$DOC2_ID" ]; then
    SIGNER1_ID=$(add_signer "$DOC2_ID" "legal@acme.com" "Acme Corp Legal")
    SIGNER2_ID=$(add_signer "$DOC2_ID" "partner@startup.io" "Partner Company")

    add_field "$DOC2_ID" "signature" 1 100 350 200 50 "$SIGNER1_ID"
    add_field "$DOC2_ID" "signature" 1 100 290 200 50 "$SIGNER2_ID"
    add_field "$DOC2_ID" "date" 1 100 230 120 25 "$SIGNER2_ID"

    echo -e "${GREEN}  Created with 2 signers (ID: ${DOC2_ID})${NC}"
else
    echo -e "${RED}  Failed to create${NC}"
fi

# Document 3: Service Contract (self-sign)
echo -e "${YELLOW}Creating Service Contract (self-sign)...${NC}"
DOC3_ID=$(upload_document "Service Contract - Web Development" "/tmp/signvault-demo/service-contract.pdf" "true")
if [ -n "$DOC3_ID" ]; then
    # Self-sign fields don't have signer_id
    add_field "$DOC3_ID" "signature" 1 100 380 200 50 ""
    add_field "$DOC3_ID" "date" 1 100 340 120 25 ""
    add_field "$DOC3_ID" "signature" 1 100 260 200 50 ""
    add_field "$DOC3_ID" "date" 1 100 220 120 25 ""

    echo -e "${GREEN}  Created for self-signing (ID: ${DOC3_ID})${NC}"
else
    echo -e "${RED}  Failed to create${NC}"
fi

# Document 4: Another employment doc (single external signer)
echo -e "${YELLOW}Creating Contractor Agreement...${NC}"
DOC4_ID=$(upload_document "Contractor Agreement - Jane Doe" "/tmp/signvault-demo/employment-agreement.pdf" "false")
if [ -n "$DOC4_ID" ]; then
    SIGNER_ID=$(add_signer "$DOC4_ID" "jane.doe@freelancer.com" "Jane Doe")

    add_field "$DOC4_ID" "signature" 1 100 420 200 50 "$SIGNER_ID"
    add_field "$DOC4_ID" "date" 1 100 380 120 25 "$SIGNER_ID"
    add_field "$DOC4_ID" "text" 1 100 340 200 25 "$SIGNER_ID"

    echo -e "${GREEN}  Created with 1 signer (ID: ${DOC4_ID})${NC}"
else
    echo -e "${RED}  Failed to create${NC}"
fi

# Cleanup temp files
rm -rf /tmp/signvault-demo

echo ""
echo "================================================"
echo -e "${GREEN}Demo data seeded successfully!${NC}"
echo "================================================"
echo ""
echo "Created documents:"
echo "  - Employment Agreement (2 signers, draft)"
echo "  - NDA (2 signers, draft)"
echo "  - Service Contract (self-sign, draft)"
echo "  - Contractor Agreement (1 signer, draft)"
echo ""
echo "Login to the app at http://localhost:5173 to see them."
echo "You can send documents for signing and test the workflow."
echo ""
