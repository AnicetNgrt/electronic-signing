import { create } from 'zustand';
import type {
  Document,
  DocumentWithFields,
  DocumentField,
  Signer,
  AddFieldRequest,
  UpdateFieldRequest,
  AddSignerRequest,
} from '@/types';
import { api } from '@/api/client';

interface DocumentState {
  documents: Document[];
  currentDocument: DocumentWithFields | null;
  total: number;
  isLoading: boolean;
  error: string | null;

  fetchDocuments: (limit?: number, offset?: number) => Promise<void>;
  fetchDocument: (id: string) => Promise<void>;
  createDocument: (title: string, file: File, selfSignOnly: boolean) => Promise<Document>;
  deleteDocument: (id: string) => Promise<void>;
  sendDocument: (id: string) => Promise<void>;
  voidDocument: (id: string) => Promise<void>;

  addField: (documentId: string, field: AddFieldRequest) => Promise<DocumentField>;
  updateField: (documentId: string, fieldId: string, updates: UpdateFieldRequest) => Promise<void>;
  deleteField: (documentId: string, fieldId: string) => Promise<void>;

  addSigner: (documentId: string, signer: AddSignerRequest) => Promise<Signer>;
  removeSigner: (documentId: string, signerId: string) => Promise<void>;

  clearError: () => void;
  clearCurrentDocument: () => void;
}

export const useDocumentStore = create<DocumentState>((set) => ({
  documents: [],
  currentDocument: null,
  total: 0,
  isLoading: false,
  error: null,

  fetchDocuments: async (limit = 20, offset = 0) => {
    set({ isLoading: true, error: null });
    try {
      const response = await api.listDocuments(limit, offset);
      set({
        documents: response.documents,
        total: response.total,
        isLoading: false,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to fetch documents',
        isLoading: false,
      });
    }
  },

  fetchDocument: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const document = await api.getDocument(id);
      set({ currentDocument: document, isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to fetch document',
        isLoading: false,
      });
    }
  },

  createDocument: async (title: string, file: File, selfSignOnly: boolean) => {
    set({ isLoading: true, error: null });
    try {
      const document = await api.createDocument(title, file, selfSignOnly);
      set((state) => ({
        documents: [document, ...state.documents],
        total: state.total + 1,
        isLoading: false,
      }));
      return document;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to create document',
        isLoading: false,
      });
      throw err;
    }
  },

  deleteDocument: async (id: string) => {
    try {
      await api.deleteDocument(id);
      set((state) => ({
        documents: state.documents.filter((d) => d.id !== id),
        total: state.total - 1,
        currentDocument:
          state.currentDocument?.id === id ? null : state.currentDocument,
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to delete document',
      });
      throw err;
    }
  },

  sendDocument: async (id: string) => {
    try {
      const updated = await api.sendDocument(id);
      set((state) => ({
        documents: state.documents.map((d) => (d.id === id ? updated : d)),
        currentDocument:
          state.currentDocument?.id === id
            ? { ...state.currentDocument, ...updated }
            : state.currentDocument,
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to send document',
      });
      throw err;
    }
  },

  voidDocument: async (id: string) => {
    try {
      const updated = await api.voidDocument(id);
      set((state) => ({
        documents: state.documents.map((d) => (d.id === id ? updated : d)),
        currentDocument:
          state.currentDocument?.id === id
            ? { ...state.currentDocument, ...updated }
            : state.currentDocument,
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to void document',
      });
      throw err;
    }
  },

  addField: async (documentId: string, field: AddFieldRequest) => {
    const newField = await api.addField(documentId, field);
    set((state) => {
      if (state.currentDocument?.id === documentId) {
        return {
          currentDocument: {
            ...state.currentDocument,
            fields: [...state.currentDocument.fields, newField],
          },
        };
      }
      return state;
    });
    return newField;
  },

  updateField: async (documentId: string, fieldId: string, updates: UpdateFieldRequest) => {
    const updated = await api.updateField(documentId, fieldId, updates);
    set((state) => {
      if (state.currentDocument?.id === documentId) {
        return {
          currentDocument: {
            ...state.currentDocument,
            fields: state.currentDocument.fields.map((f) =>
              f.id === fieldId ? updated : f
            ),
          },
        };
      }
      return state;
    });
  },

  deleteField: async (documentId: string, fieldId: string) => {
    await api.deleteField(documentId, fieldId);
    set((state) => {
      if (state.currentDocument?.id === documentId) {
        return {
          currentDocument: {
            ...state.currentDocument,
            fields: state.currentDocument.fields.filter((f) => f.id !== fieldId),
          },
        };
      }
      return state;
    });
  },

  addSigner: async (documentId: string, signer: AddSignerRequest) => {
    const newSigner = await api.addSigner(documentId, signer);
    set((state) => {
      if (state.currentDocument?.id === documentId) {
        return {
          currentDocument: {
            ...state.currentDocument,
            signers: [...state.currentDocument.signers, newSigner],
            total_signers: state.currentDocument.total_signers + 1,
          },
        };
      }
      return state;
    });
    return newSigner;
  },

  removeSigner: async (documentId: string, signerId: string) => {
    await api.removeSigner(documentId, signerId);
    set((state) => {
      if (state.currentDocument?.id === documentId) {
        return {
          currentDocument: {
            ...state.currentDocument,
            signers: state.currentDocument.signers.filter((s) => s.id !== signerId),
            total_signers: state.currentDocument.total_signers - 1,
          },
        };
      }
      return state;
    });
  },

  clearError: () => set({ error: null }),
  clearCurrentDocument: () => set({ currentDocument: null }),
}));
