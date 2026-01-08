import type {
  LoginResponse,
  User,
  DocumentListResponse,
  DocumentWithFields,
  Document,
  DocumentField,
  Signer,
  AuditLog,
  Certificate,
  SigningSession,
  AddFieldRequest,
  UpdateFieldRequest,
  AddSignerRequest,
  CompleteSigningRequest,
  ApiError,
} from '@/types';

const API_BASE = '/api';

class ApiClient {
  private token: string | null = null;

  setToken(token: string | null): void {
    this.token = token;
    if (token) {
      localStorage.setItem('auth_token', token);
    } else {
      localStorage.removeItem('auth_token');
    }
  }

  getToken(): string | null {
    if (!this.token) {
      this.token = localStorage.getItem('auth_token');
    }
    return this.token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: HeadersInit = {
      ...options.headers,
    };

    const token = this.getToken();
    if (token) {
      (headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
    }

    if (!(options.body instanceof FormData)) {
      (headers as Record<string, string>)['Content-Type'] = 'application/json';
    }

    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      let errorData: ApiError;
      try {
        errorData = await response.json() as ApiError;
      } catch {
        errorData = {
          error: 'unknown',
          message: `Request failed with status ${response.status}`,
        };
      }
      throw new ApiClientError(errorData.message, errorData.error, response.status);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    const contentType = response.headers.get('content-type');
    if (contentType?.includes('application/json')) {
      return response.json() as Promise<T>;
    }

    return response.blob() as unknown as T;
  }

  // Auth
  async login(email: string, password: string): Promise<LoginResponse> {
    const response = await this.request<LoginResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setToken(response.token);
    return response;
  }

  async getCurrentUser(): Promise<User> {
    return this.request<User>('/auth/me');
  }

  logout(): void {
    this.setToken(null);
  }

  // Documents
  async listDocuments(limit = 20, offset = 0): Promise<DocumentListResponse> {
    return this.request<DocumentListResponse>(
      `/documents?limit=${limit}&offset=${offset}`
    );
  }

  async getDocument(id: string): Promise<DocumentWithFields> {
    return this.request<DocumentWithFields>(`/documents/${id}`);
  }

  async createDocument(
    title: string,
    file: File,
    selfSignOnly: boolean
  ): Promise<Document> {
    const formData = new FormData();
    formData.append('title', title);
    formData.append('file', file);
    formData.append('self_sign_only', selfSignOnly.toString());

    return this.request<Document>('/documents', {
      method: 'POST',
      body: formData,
    });
  }

  async deleteDocument(id: string): Promise<void> {
    await this.request<{ success: boolean }>(`/documents/${id}`, {
      method: 'DELETE',
    });
  }

  async sendDocument(id: string): Promise<Document> {
    return this.request<Document>(`/documents/${id}/send`, {
      method: 'POST',
    });
  }

  async voidDocument(id: string): Promise<Document> {
    return this.request<Document>(`/documents/${id}/void`, {
      method: 'POST',
    });
  }

  async downloadDocument(id: string): Promise<Blob> {
    return this.request<Blob>(`/documents/${id}/download`);
  }

  // Fields
  async addField(documentId: string, field: AddFieldRequest): Promise<DocumentField> {
    return this.request<DocumentField>(`/documents/${documentId}/fields`, {
      method: 'POST',
      body: JSON.stringify(field),
    });
  }

  async updateField(
    documentId: string,
    fieldId: string,
    updates: UpdateFieldRequest
  ): Promise<DocumentField> {
    return this.request<DocumentField>(
      `/documents/${documentId}/fields/${fieldId}`,
      {
        method: 'PUT',
        body: JSON.stringify(updates),
      }
    );
  }

  async deleteField(documentId: string, fieldId: string): Promise<void> {
    await this.request<{ success: boolean }>(
      `/documents/${documentId}/fields/${fieldId}`,
      {
        method: 'DELETE',
      }
    );
  }

  // Signers
  async addSigner(documentId: string, signer: AddSignerRequest): Promise<Signer> {
    return this.request<Signer>(`/documents/${documentId}/signers`, {
      method: 'POST',
      body: JSON.stringify(signer),
    });
  }

  async removeSigner(documentId: string, signerId: string): Promise<void> {
    await this.request<{ success: boolean }>(
      `/documents/${documentId}/signers/${signerId}`,
      {
        method: 'DELETE',
      }
    );
  }

  // Audit
  async getAuditLogs(documentId: string): Promise<AuditLog[]> {
    return this.request<AuditLog[]>(`/documents/${documentId}/audit`);
  }

  async getCertificate(documentId: string): Promise<Certificate> {
    return this.request<Certificate>(`/documents/${documentId}/certificate`);
  }

  // Signing (public routes)
  async getSigningSession(token: string): Promise<SigningSession> {
    return this.request<SigningSession>(`/sign/${token}`);
  }

  async getSigningPdf(token: string): Promise<Blob> {
    const response = await fetch(`${API_BASE}/sign/${token}/pdf`);
    if (!response.ok) {
      throw new ApiClientError('Failed to load PDF', 'pdf_error', response.status);
    }
    return response.blob();
  }

  async submitSigning(
    token: string,
    request: CompleteSigningRequest
  ): Promise<{ success: boolean; document_completed: boolean }> {
    return this.request<{ success: boolean; document_completed: boolean }>(
      `/sign/${token}/submit`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  }

  async declineSigning(token: string, reason?: string): Promise<void> {
    await this.request<{ success: boolean }>(`/sign/${token}/decline`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  }
}

export class ApiClientError extends Error {
  constructor(
    message: string,
    public errorType: string,
    public statusCode: number
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

export const api = new ApiClient();
