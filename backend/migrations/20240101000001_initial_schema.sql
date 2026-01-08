-- SignVault Initial Schema Migration
-- This creates all required tables for the electronic signature platform

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create custom enum types
CREATE TYPE document_status AS ENUM ('draft', 'pending', 'completed', 'voided', 'expired');
CREATE TYPE field_type AS ENUM ('signature', 'date', 'text', 'initial');
CREATE TYPE signer_status AS ENUM ('pending', 'sent', 'viewed', 'signed', 'declined');
CREATE TYPE audit_action AS ENUM (
    'document_created',
    'document_uploaded',
    'document_viewed',
    'document_sent',
    'document_completed',
    'document_voided',
    'document_downloaded',
    'field_added',
    'field_updated',
    'field_deleted',
    'signer_added',
    'signer_removed',
    'signer_email_sent',
    'signer_viewed',
    'signer_signed',
    'signer_declined',
    'signature_applied',
    'certificate_generated'
);

-- Users table (admin accounts)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Documents table
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    file_hash VARCHAR(128) NOT NULL,
    status document_status NOT NULL DEFAULT 'draft',
    self_sign_only BOOLEAN NOT NULL DEFAULT false,
    total_signers INTEGER NOT NULL DEFAULT 0,
    completed_signers INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_owner_id ON documents(owner_id);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_created_at ON documents(created_at DESC);

-- Signers table
CREATE TABLE signers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    status signer_status NOT NULL DEFAULT 'pending',
    access_token VARCHAR(128) NOT NULL UNIQUE,
    ip_address VARCHAR(45),
    user_agent TEXT,
    viewed_at TIMESTAMPTZ,
    signed_at TIMESTAMPTZ,
    declined_at TIMESTAMPTZ,
    decline_reason TEXT,
    email_sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_signers_document_id ON signers(document_id);
CREATE INDEX idx_signers_access_token ON signers(access_token);
CREATE INDEX idx_signers_email ON signers(email);

-- Document fields table (signature boxes, date fields, text fields)
CREATE TABLE document_fields (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    field_type field_type NOT NULL,
    page INTEGER NOT NULL,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    width DOUBLE PRECISION NOT NULL,
    height DOUBLE PRECISION NOT NULL,
    signer_id UUID REFERENCES signers(id) ON DELETE SET NULL,
    value TEXT,
    font_size INTEGER DEFAULT 12,
    font_family VARCHAR(100) DEFAULT 'Arial',
    date_format VARCHAR(50) DEFAULT 'YYYY-MM-DD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_fields_document_id ON document_fields(document_id);
CREATE INDEX idx_document_fields_signer_id ON document_fields(signer_id);

-- Signatures table (stores actual signature data)
CREATE TABLE signatures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    signer_id UUID NOT NULL REFERENCES signers(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    field_id UUID NOT NULL REFERENCES document_fields(id) ON DELETE CASCADE,
    signature_data TEXT NOT NULL,
    signature_hash VARCHAR(128) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_signatures_signer_id ON signatures(signer_id);
CREATE INDEX idx_signatures_document_id ON signatures(document_id);

-- Audit log table (immutable, cryptographically linked)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    signer_id UUID REFERENCES signers(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action audit_action NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    entry_hash VARCHAR(128) NOT NULL,
    previous_hash VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_document_id ON audit_logs(document_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply triggers
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_signers_updated_at
    BEFORE UPDATE ON signers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_document_fields_updated_at
    BEFORE UPDATE ON document_fields
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
