# SignVault - Electronic Signature Platform

A legally-compliant electronic signature platform built with Rust (backend) and React TypeScript (frontend).

## Features

- **Legally Compliant**: Meets ESIGN Act (USA) and eIDAS (EU) requirements
- **PDF Document Handling**: Upload, view, and annotate PDF documents
- **Drag-and-Drop Fields**: Add signature, date, text, and initial fields anywhere on documents
- **Multi-Party Signing**: Send documents to N signers via email
- **Self-Signing**: Option to sign documents yourself only
- **Cryptographic Audit Trail**: Tamper-evident blockchain-style audit logs
- **Certificate of Completion**: Generates legally-valid certificates for signed documents
- **Email Notifications**: Automated emails for signature requests and completions
- **Document Tracking**: Track document status and signer progress
- **Self-Hostable**: Full Docker support for easy deployment

## Tech Stack

- **Backend**: Rust with Axum web framework, SQLx for PostgreSQL
- **Frontend**: React 18 with TypeScript, Tailwind CSS
- **Database**: PostgreSQL 16
- **PDF Rendering**: PDF.js
- **Email**: Lettre (Rust SMTP client)

## Quick Start

### Prerequisites

- Docker and Docker Compose
- (For development) Rust 1.75+, Node.js 20+

### Production Deployment

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/signvault.git
   cd signvault
   ```

2. Copy and configure environment:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. Start with Docker Compose:
   ```bash
   docker-compose up -d
   ```

4. Access the application at `http://localhost`

### Development Setup

1. Run the initial setup:
   ```bash
   ./initial-setup.sh
   ```

2. Start the development servers:
   ```bash
   ./start-fullstack-dev-watch.sh
   ```

3. Access:
   - Frontend: http://localhost:5173
   - Backend API: http://localhost:8080/api

### Default Credentials

- Email: `admin@example.com`
- Password: `change-this-secure-password`

**Important**: Change these in your `.env` file before production deployment.

## Scripts

| Script | Description |
|--------|-------------|
| `./initial-setup.sh` | First-time setup: installs dependencies, creates `.env` |
| `./start-fullstack-dev-watch.sh` | Starts all services with hot-reload |
| `./checks.sh` | Runs linting and type checking |
| `./tests.sh` | Runs all tests (unit, integration, e2e) |

## Configuration

See `.env.example` for all configuration options:

- **Database**: PostgreSQL connection settings
- **Authentication**: JWT secret and expiration
- **Email**: SMTP server configuration
- **Storage**: File upload limits and paths

## API Endpoints

### Authentication
- `POST /api/auth/login` - Login with email/password
- `GET /api/auth/me` - Get current user

### Documents
- `GET /api/documents` - List documents
- `POST /api/documents` - Create new document (multipart)
- `GET /api/documents/:id` - Get document with fields and signers
- `DELETE /api/documents/:id` - Delete document
- `POST /api/documents/:id/send` - Send for signing
- `POST /api/documents/:id/void` - Void document
- `GET /api/documents/:id/audit` - Get audit trail
- `GET /api/documents/:id/certificate` - Get completion certificate
- `GET /api/documents/:id/download` - Download PDF

### Fields
- `POST /api/documents/:id/fields` - Add field
- `PUT /api/documents/:id/fields/:fieldId` - Update field
- `DELETE /api/documents/:id/fields/:fieldId` - Delete field

### Signers
- `POST /api/documents/:id/signers` - Add signer
- `DELETE /api/documents/:id/signers/:signerId` - Remove signer

### Signing (Public)
- `GET /api/sign/:token` - Get signing session
- `GET /api/sign/:token/pdf` - Get PDF for signing
- `POST /api/sign/:token/submit` - Submit signatures
- `POST /api/sign/:token/decline` - Decline to sign

## Legal Compliance

SignVault is designed to meet electronic signature requirements:

### ESIGN Act (USA)
- Electronic records attributable to signers
- Consumer consent tracking
- Detailed audit trails with timestamps
- IP address and device logging

### eIDAS (EU)
- Simple Electronic Signatures (SES) compliance
- Tamper-evident audit logs
- Cryptographic integrity verification
- Certificate of completion generation

## Security Features

- JWT-based authentication
- Bcrypt password hashing
- Cryptographically-linked audit chain
- Document hash verification
- CORS protection
- Rate limiting support

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run `./checks.sh` to ensure code quality
4. Run `./tests.sh` to run all tests
5. Submit a pull request
