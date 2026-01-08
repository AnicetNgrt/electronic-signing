export interface User {
  id: string;
  email: string;
  name: string;
  is_admin: boolean;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export type DocumentStatus = 'draft' | 'pending' | 'completed' | 'voided' | 'expired';
export type FieldType = 'signature' | 'date' | 'text' | 'initial';
export type SignerStatus = 'pending' | 'sent' | 'viewed' | 'signed' | 'declined';

export interface Document {
  id: string;
  owner_id: string;
  title: string;
  original_filename: string;
  file_path: string;
  file_hash: string;
  status: DocumentStatus;
  self_sign_only: boolean;
  total_signers: number;
  completed_signers: number;
  expires_at: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface DocumentField {
  id: string;
  document_id: string;
  field_type: FieldType;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  signer_id: string | null;
  value: string | null;
  font_size: number | null;
  font_family: string | null;
  date_format: string | null;
  created_at: string;
  updated_at: string;
}

export interface Signer {
  id: string;
  document_id: string;
  email: string;
  name: string;
  order_index: number;
  status: SignerStatus;
  access_token: string;
  ip_address: string | null;
  user_agent: string | null;
  viewed_at: string | null;
  signed_at: string | null;
  declined_at: string | null;
  decline_reason: string | null;
  email_sent_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface DocumentWithFields extends Document {
  fields: DocumentField[];
  signers: Signer[];
}

export interface DocumentListResponse {
  documents: Document[];
  total: number;
}

export interface AddFieldRequest {
  field_type: FieldType;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  signer_id?: string;
  value?: string;
  font_size?: number;
  font_family?: string;
  date_format?: string;
}

export interface UpdateFieldRequest {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  value?: string;
  font_size?: number;
  font_family?: string;
  date_format?: string;
}

export interface AddSignerRequest {
  email: string;
  name: string;
  order_index?: number;
}

export type AuditAction =
  | 'document_created'
  | 'document_uploaded'
  | 'document_viewed'
  | 'document_sent'
  | 'document_completed'
  | 'document_voided'
  | 'document_downloaded'
  | 'field_added'
  | 'field_updated'
  | 'field_deleted'
  | 'signer_added'
  | 'signer_removed'
  | 'signer_email_sent'
  | 'signer_viewed'
  | 'signer_signed'
  | 'signer_declined'
  | 'signature_applied'
  | 'certificate_generated';

export interface AuditLog {
  id: string;
  document_id: string;
  signer_id: string | null;
  user_id: string | null;
  action: AuditAction;
  ip_address: string | null;
  user_agent: string | null;
  details: Record<string, unknown> | null;
  entry_hash: string;
  previous_hash: string | null;
  created_at: string;
}

export interface CertificateSigner {
  name: string;
  email: string;
  signed_at: string;
  ip_address: string;
  signature_hash: string;
}

export interface CertificateAuditEntry {
  action: string;
  actor: string | null;
  timestamp: string;
  ip_address: string | null;
  details: string | null;
}

export interface Certificate {
  document_id: string;
  document_title: string;
  document_hash: string;
  created_at: string;
  completed_at: string;
  signers: CertificateSigner[];
  audit_trail: CertificateAuditEntry[];
  certificate_hash: string;
  generated_at: string;
}

export interface SigningSession {
  document_id: string;
  document_title: string;
  signer: {
    id: string;
    name: string;
    email: string;
    status: SignerStatus;
  };
  fields: DocumentField[];
  page_count: number;
}

export interface SubmitSignatureRequest {
  field_id: string;
  signature_data: string;
}

export interface SubmitFieldValueRequest {
  field_id: string;
  value: string;
}

export interface CompleteSigningRequest {
  signatures: SubmitSignatureRequest[];
  field_values: SubmitFieldValueRequest[];
}

export interface ApiError {
  error: string;
  message: string;
}
