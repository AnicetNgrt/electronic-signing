import { useEffect, useState, useCallback, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useDocumentStore } from '@/stores/document';
import { api } from '@/api/client';
import PDFViewer from '@/components/PDFViewer';
import DraggableField from '@/components/DraggableField';
import type { FieldType, DocumentField, AuditLog, Certificate } from '@/types';
import { format } from 'date-fns';

const fieldTypes: { type: FieldType; label: string; icon: string }[] = [
  {
    type: 'signature',
    label: 'Signature',
    icon: 'M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z',
  },
  {
    type: 'date',
    label: 'Date',
    icon: 'M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
  },
  {
    type: 'text',
    label: 'Text',
    icon: 'M4 6h16M4 12h16M4 18h7',
  },
  {
    type: 'initial',
    label: 'Initial',
    icon: 'M13 10V3L4 14h7v7l9-11h-7z',
  },
];

export default function DocumentEditorPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const {
    currentDocument,
    isLoading,
    error,
    fetchDocument,
    clearCurrentDocument,
    addField,
    updateField,
    deleteField,
    addSigner,
    removeSigner,
    sendDocument,
    voidDocument,
    deleteDocument,
  } = useDocumentStore();

  const [selectedFieldId, setSelectedFieldId] = useState<string | null>(null);
  const [pdfUrl, setPdfUrl] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'fields' | 'signers' | 'audit'>('fields');
  const [newSignerEmail, setNewSignerEmail] = useState('');
  const [newSignerName, setNewSignerName] = useState('');
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);
  const [certificate, setCertificate] = useState<Certificate | null>(null);
  const [isSending, setIsSending] = useState(false);

  useEffect(() => {
    if (id) {
      void fetchDocument(id);
    }
    return () => {
      clearCurrentDocument();
    };
  }, [id, fetchDocument, clearCurrentDocument]);

  useEffect(() => {
    if (currentDocument && id) {
      api
        .downloadDocument(id)
        .then((blob) => {
          const url = URL.createObjectURL(blob);
          setPdfUrl(url);
        })
        .catch(console.error);
    }
    return () => {
      if (pdfUrl) {
        URL.revokeObjectURL(pdfUrl);
      }
    };
  }, [currentDocument, id]);

  const loadAuditLogs = useCallback(async () => {
    if (!id) return;
    try {
      const logs = await api.getAuditLogs(id);
      setAuditLogs(logs);
    } catch (err) {
      console.error('Failed to load audit logs:', err);
    }
  }, [id]);

  const loadCertificate = useCallback(async () => {
    if (!id || currentDocument?.status !== 'completed') return;
    try {
      const cert = await api.getCertificate(id);
      setCertificate(cert);
    } catch (err) {
      console.error('Failed to load certificate:', err);
    }
  }, [id, currentDocument?.status]);

  useEffect(() => {
    if (activeTab === 'audit') {
      void loadAuditLogs();
      void loadCertificate();
    }
  }, [activeTab, loadAuditLogs, loadCertificate]);

  const handleAddField = useCallback(
    async (type: FieldType, pageNum: number) => {
      if (!id) return;
      await addField(id, {
        field_type: type,
        page: pageNum,
        x: 100,
        y: 100,
        width: type === 'signature' ? 200 : type === 'date' ? 150 : 180,
        height: type === 'signature' ? 60 : 30,
        font_size: 12,
        date_format: type === 'date' ? 'MMMM d, yyyy' : undefined,
      });
    },
    [id, addField]
  );

  const handleUpdateField = useCallback(
    async (fieldId: string, updates: { x?: number; y?: number; width?: number; height?: number }) => {
      if (!id) return;
      await updateField(id, fieldId, updates);
    },
    [id, updateField]
  );

  const handleDeleteField = useCallback(
    async (fieldId: string) => {
      if (!id) return;
      await deleteField(id, fieldId);
      setSelectedFieldId(null);
    },
    [id, deleteField]
  );

  const handleAddSigner = useCallback(async () => {
    if (!id || !newSignerEmail || !newSignerName) return;
    await addSigner(id, { email: newSignerEmail, name: newSignerName });
    setNewSignerEmail('');
    setNewSignerName('');
  }, [id, newSignerEmail, newSignerName, addSigner]);

  const handleRemoveSigner = useCallback(
    async (signerId: string) => {
      if (!id) return;
      await removeSigner(id, signerId);
    },
    [id, removeSigner]
  );

  const handleSend = useCallback(async () => {
    if (!id) return;
    setIsSending(true);
    try {
      await sendDocument(id);
    } finally {
      setIsSending(false);
    }
  }, [id, sendDocument]);

  const handleVoid = useCallback(async () => {
    if (!id) return;
    if (window.confirm('Are you sure you want to void this document?')) {
      await voidDocument(id);
    }
  }, [id, voidDocument]);

  const handleDelete = useCallback(async () => {
    if (!id) return;
    if (window.confirm('Are you sure you want to delete this document?')) {
      await deleteDocument(id);
      navigate('/');
    }
  }, [id, deleteDocument, navigate]);

  const signerMap = useMemo(() => {
    const map = new Map<string, string>();
    currentDocument?.signers.forEach((s) => map.set(s.id, s.name));
    return map;
  }, [currentDocument?.signers]);

  const renderOverlay = useCallback(
    (pageNum: number) => {
      if (!currentDocument) return null;

      const pageFields = currentDocument.fields.filter((f) => f.page === pageNum);

      return (
        <>
          {pageFields.map((field) => (
            <DraggableField
              key={field.id}
              field={field}
              isSelected={selectedFieldId === field.id}
              onSelect={() => setSelectedFieldId(field.id)}
              onUpdate={(updates) => void handleUpdateField(field.id, updates)}
              onDelete={() => void handleDeleteField(field.id)}
              readOnly={currentDocument.status !== 'draft'}
              signerName={field.signer_id ? signerMap.get(field.signer_id) : undefined}
            />
          ))}
        </>
      );
    },
    [currentDocument, selectedFieldId, handleUpdateField, handleDeleteField, signerMap]
  );

  if (isLoading || !currentDocument) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <p className="text-red-600">{error}</p>
      </div>
    );
  }

  const isDraft = currentDocument.status === 'draft';

  return (
    <div className="flex gap-6">
      {/* PDF Viewer */}
      <div className="flex-1 min-w-0">
        <div className="bg-white rounded-lg shadow p-4 mb-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-gray-900">{currentDocument.title}</h1>
              <p className="text-sm text-gray-500">
                Status:{' '}
                <span className="capitalize">{currentDocument.status}</span>
                {currentDocument.completed_at && (
                  <> - Completed {format(new Date(currentDocument.completed_at), 'MMM d, yyyy')}</>
                )}
              </p>
            </div>
            <div className="flex gap-2">
              {isDraft && (
                <>
                  <button
                    onClick={handleSend}
                    disabled={isSending || (!currentDocument.self_sign_only && currentDocument.signers.length === 0)}
                    className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {isSending ? 'Sending...' : 'Send for Signing'}
                  </button>
                  <button
                    onClick={handleDelete}
                    className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700"
                  >
                    Delete
                  </button>
                </>
              )}
              {currentDocument.status === 'pending' && (
                <button
                  onClick={handleVoid}
                  className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700"
                >
                  Void Document
                </button>
              )}
            </div>
          </div>
        </div>

        <div
          className="bg-gray-200 rounded-lg p-4 overflow-auto"
          style={{ maxHeight: 'calc(100vh - 250px)' }}
          onClick={() => setSelectedFieldId(null)}
        >
          {pdfUrl && (
            <PDFViewer url={pdfUrl} scale={1.5} renderOverlay={renderOverlay} />
          )}
        </div>
      </div>

      {/* Sidebar */}
      <div className="w-80 flex-shrink-0">
        <div className="bg-white rounded-lg shadow">
          <div className="border-b border-gray-200">
            <nav className="flex -mb-px">
              <button
                onClick={() => setActiveTab('fields')}
                className={`flex-1 py-3 px-4 text-center text-sm font-medium border-b-2 ${
                  activeTab === 'fields'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                Fields
              </button>
              <button
                onClick={() => setActiveTab('signers')}
                className={`flex-1 py-3 px-4 text-center text-sm font-medium border-b-2 ${
                  activeTab === 'signers'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                Signers
              </button>
              <button
                onClick={() => setActiveTab('audit')}
                className={`flex-1 py-3 px-4 text-center text-sm font-medium border-b-2 ${
                  activeTab === 'audit'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                Audit
              </button>
            </nav>
          </div>

          <div className="p-4">
            {activeTab === 'fields' && (
              <div>
                <h3 className="text-sm font-medium text-gray-900 mb-3">Add Fields</h3>
                {isDraft ? (
                  <div className="grid grid-cols-2 gap-2">
                    {fieldTypes.map(({ type, label, icon }) => (
                      <button
                        key={type}
                        onClick={() => void handleAddField(type, 1)}
                        className="flex flex-col items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50"
                      >
                        <svg className="w-6 h-6 text-gray-600 mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={icon} />
                        </svg>
                        <span className="text-xs text-gray-700">{label}</span>
                      </button>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-gray-500">Fields cannot be modified after sending.</p>
                )}

                {currentDocument.fields.length > 0 && (
                  <div className="mt-4">
                    <h4 className="text-sm font-medium text-gray-900 mb-2">
                      Fields ({currentDocument.fields.length})
                    </h4>
                    <ul className="space-y-1">
                      {currentDocument.fields.map((field) => (
                        <li
                          key={field.id}
                          className={`text-sm p-2 rounded cursor-pointer ${
                            selectedFieldId === field.id
                              ? 'bg-primary-100 text-primary-700'
                              : 'hover:bg-gray-50'
                          }`}
                          onClick={() => setSelectedFieldId(field.id)}
                        >
                          <span className="capitalize">{field.field_type}</span>
                          <span className="text-gray-500"> - Page {field.page}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'signers' && (
              <div>
                {currentDocument.self_sign_only ? (
                  <p className="text-sm text-gray-500">This document is set for self-signing only.</p>
                ) : (
                  <>
                    {isDraft && (
                      <div className="mb-4">
                        <h3 className="text-sm font-medium text-gray-900 mb-2">Add Signer</h3>
                        <div className="space-y-2">
                          <input
                            type="text"
                            placeholder="Name"
                            value={newSignerName}
                            onChange={(e) => setNewSignerName(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                          <input
                            type="email"
                            placeholder="Email"
                            value={newSignerEmail}
                            onChange={(e) => setNewSignerEmail(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                          />
                          <button
                            onClick={() => void handleAddSigner()}
                            disabled={!newSignerEmail || !newSignerName}
                            className="w-full px-3 py-2 bg-primary-600 text-white rounded-md text-sm hover:bg-primary-700 disabled:opacity-50"
                          >
                            Add Signer
                          </button>
                        </div>
                      </div>
                    )}

                    <h3 className="text-sm font-medium text-gray-900 mb-2">
                      Signers ({currentDocument.signers.length})
                    </h3>
                    {currentDocument.signers.length === 0 ? (
                      <p className="text-sm text-gray-500">No signers added yet.</p>
                    ) : (
                      <ul className="space-y-2">
                        {currentDocument.signers.map((signer) => (
                          <li key={signer.id} className="flex items-center justify-between p-2 bg-gray-50 rounded">
                            <div>
                              <p className="text-sm font-medium">{signer.name}</p>
                              <p className="text-xs text-gray-500">{signer.email}</p>
                              <p className="text-xs text-gray-400 capitalize">{signer.status}</p>
                            </div>
                            {isDraft && (
                              <button
                                onClick={() => void handleRemoveSigner(signer.id)}
                                className="text-red-600 hover:text-red-800"
                              >
                                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                              </button>
                            )}
                          </li>
                        ))}
                      </ul>
                    )}
                  </>
                )}
              </div>
            )}

            {activeTab === 'audit' && (
              <div>
                <h3 className="text-sm font-medium text-gray-900 mb-2">Audit Trail</h3>
                {auditLogs.length === 0 ? (
                  <p className="text-sm text-gray-500">No audit entries yet.</p>
                ) : (
                  <ul className="space-y-2 max-h-96 overflow-auto">
                    {auditLogs.map((log) => (
                      <li key={log.id} className="text-xs p-2 bg-gray-50 rounded">
                        <p className="font-medium capitalize">
                          {log.action.replace(/_/g, ' ')}
                        </p>
                        <p className="text-gray-500">
                          {format(new Date(log.created_at), 'MMM d, yyyy h:mm a')}
                        </p>
                        {log.ip_address && (
                          <p className="text-gray-400">IP: {log.ip_address}</p>
                        )}
                      </li>
                    ))}
                  </ul>
                )}

                {certificate && (
                  <div className="mt-4 p-3 bg-green-50 rounded-lg">
                    <h4 className="text-sm font-medium text-green-800">Certificate of Completion</h4>
                    <p className="text-xs text-green-600 mt-1">
                      Hash: {certificate.certificate_hash.slice(0, 16)}...
                    </p>
                    <p className="text-xs text-green-600">
                      Generated: {format(new Date(certificate.generated_at), 'MMM d, yyyy h:mm a')}
                    </p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
