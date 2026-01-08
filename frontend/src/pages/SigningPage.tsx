import { useEffect, useState, useCallback, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { api } from '@/api/client';
import PDFViewer from '@/components/PDFViewer';
import SignaturePad from '@/components/SignaturePad';
import type { SigningSession, DocumentField, FieldType } from '@/types';
import { format } from 'date-fns';

interface FieldValue {
  fieldId: string;
  value: string;
}

interface SignatureValue {
  fieldId: string;
  signatureData: string;
}

const dateFormats: Record<string, string> = {
  'YYYY-MM-DD': 'yyyy-MM-dd',
  'MM/DD/YYYY': 'MM/dd/yyyy',
  'DD/MM/YYYY': 'dd/MM/yyyy',
  'MMMM d, yyyy': 'MMMM d, yyyy',
  'MMM d, yyyy': 'MMM d, yyyy',
  'd MMMM yyyy': 'd MMMM yyyy',
};

export default function SigningPage() {
  const { token } = useParams<{ token: string }>();
  const navigate = useNavigate();
  const [session, setSession] = useState<SigningSession | null>(null);
  const [pdfUrl, setPdfUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [signatures, setSignatures] = useState<SignatureValue[]>([]);
  const [fieldValues, setFieldValues] = useState<FieldValue[]>([]);
  const [activeSignatureField, setActiveSignatureField] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [completed, setCompleted] = useState(false);
  const [declineReason, setDeclineReason] = useState('');
  const [showDeclineModal, setShowDeclineModal] = useState(false);

  useEffect(() => {
    if (!token) return;

    const loadSession = async () => {
      setIsLoading(true);
      try {
        const [sessionData, pdfBlob] = await Promise.all([
          api.getSigningSession(token),
          api.getSigningPdf(token),
        ]);

        setSession(sessionData);
        setPdfUrl(URL.createObjectURL(pdfBlob));

        // Initialize field values for date and text fields
        const initialValues: FieldValue[] = sessionData.fields
          .filter((f) => f.field_type === 'date' || f.field_type === 'text')
          .map((f) => ({
            fieldId: f.id,
            value:
              f.field_type === 'date'
                ? format(new Date(), dateFormats[f.date_format ?? 'YYYY-MM-DD'] ?? 'yyyy-MM-dd')
                : f.value ?? '',
          }));
        setFieldValues(initialValues);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load document');
      } finally {
        setIsLoading(false);
      }
    };

    void loadSession();

    return () => {
      if (pdfUrl) {
        URL.revokeObjectURL(pdfUrl);
      }
    };
  }, [token]);

  const handleSignatureSave = useCallback(
    (signatureData: string) => {
      if (!activeSignatureField) return;

      setSignatures((prev) => {
        const existing = prev.findIndex((s) => s.fieldId === activeSignatureField);
        if (existing >= 0) {
          const updated = [...prev];
          updated[existing] = { fieldId: activeSignatureField, signatureData };
          return updated;
        }
        return [...prev, { fieldId: activeSignatureField, signatureData }];
      });
      setActiveSignatureField(null);
    },
    [activeSignatureField]
  );

  const handleFieldValueChange = useCallback((fieldId: string, value: string) => {
    setFieldValues((prev) => {
      const existing = prev.findIndex((f) => f.fieldId === fieldId);
      if (existing >= 0) {
        const updated = [...prev];
        updated[existing] = { fieldId, value };
        return updated;
      }
      return [...prev, { fieldId, value }];
    });
  }, []);

  const requiredSignatureFields = useMemo(() => {
    return session?.fields.filter((f) => f.field_type === 'signature' || f.field_type === 'initial') ?? [];
  }, [session?.fields]);

  const allSignaturesComplete = useMemo(() => {
    return requiredSignatureFields.every((f) =>
      signatures.some((s) => s.fieldId === f.id)
    );
  }, [requiredSignatureFields, signatures]);

  const handleSubmit = useCallback(async () => {
    if (!token || !allSignaturesComplete) return;

    setIsSubmitting(true);
    try {
      const result = await api.submitSigning(token, {
        signatures: signatures.map((s) => ({
          field_id: s.fieldId,
          signature_data: s.signatureData,
        })),
        field_values: fieldValues.map((f) => ({
          field_id: f.fieldId,
          value: f.value,
        })),
      });

      setCompleted(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit signature');
    } finally {
      setIsSubmitting(false);
    }
  }, [token, signatures, fieldValues, allSignaturesComplete]);

  const handleDecline = useCallback(async () => {
    if (!token) return;

    try {
      await api.declineSigning(token, declineReason || undefined);
      setError('You have declined to sign this document.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to decline');
    }
    setShowDeclineModal(false);
  }, [token, declineReason]);

  const getSignatureForField = useCallback(
    (fieldId: string) => {
      return signatures.find((s) => s.fieldId === fieldId)?.signatureData;
    },
    [signatures]
  );

  const getValueForField = useCallback(
    (fieldId: string) => {
      return fieldValues.find((f) => f.fieldId === fieldId)?.value ?? '';
    },
    [fieldValues]
  );

  const renderOverlay = useCallback(
    (pageNum: number) => {
      if (!session) return null;

      const pageFields = session.fields.filter((f) => f.page === pageNum);

      return (
        <>
          {pageFields.map((field) => (
            <SigningField
              key={field.id}
              field={field}
              signatureData={getSignatureForField(field.id)}
              value={getValueForField(field.id)}
              onSignatureClick={() => setActiveSignatureField(field.id)}
              onValueChange={(value) => handleFieldValueChange(field.id, value)}
            />
          ))}
        </>
      );
    },
    [session, getSignatureForField, getValueForField, handleFieldValueChange]
  );

  if (completed) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
        <div className="max-w-md w-full text-center">
          <div className="bg-white rounded-lg shadow-lg p-8">
            <svg
              className="mx-auto h-16 w-16 text-green-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <h2 className="mt-4 text-2xl font-bold text-gray-900">
              Document Signed Successfully
            </h2>
            <p className="mt-2 text-gray-600">
              Your signature has been recorded. You will receive a copy of the signed document via email.
            </p>
            <p className="mt-4 text-sm text-gray-500">
              You can close this window now.
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
        <div className="max-w-md w-full">
          <div className="bg-white rounded-lg shadow-lg p-8 text-center">
            <svg
              className="mx-auto h-16 w-16 text-red-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
            <h2 className="mt-4 text-xl font-bold text-gray-900">{error}</h2>
          </div>
        </div>
      </div>
    );
  }

  if (!session) return null;

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-gray-900">{session.document_title}</h1>
              <p className="text-sm text-gray-500">
                Signing as {session.signer.name} ({session.signer.email})
              </p>
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => setShowDeclineModal(true)}
                className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
              >
                Decline to Sign
              </button>
              <button
                onClick={() => void handleSubmit()}
                disabled={!allSignaturesComplete || isSubmitting}
                className="px-4 py-2 bg-primary-600 text-white rounded-md text-sm font-medium hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSubmitting ? 'Submitting...' : 'Complete Signing'}
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Progress indicator */}
      <div className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4 py-2">
          <p className="text-sm text-gray-600">
            {signatures.length} of {requiredSignatureFields.length} signature{requiredSignatureFields.length !== 1 ? 's' : ''} completed
          </p>
        </div>
      </div>

      {/* PDF Viewer */}
      <div className="max-w-5xl mx-auto px-4 py-8">
        <div className="bg-gray-200 rounded-lg p-4 overflow-auto">
          {pdfUrl && <PDFViewer url={pdfUrl} scale={1.5} renderOverlay={renderOverlay} />}
        </div>
      </div>

      {/* Signature Pad Modal */}
      {activeSignatureField && (
        <SignaturePad
          onSave={handleSignatureSave}
          onCancel={() => setActiveSignatureField(null)}
        />
      )}

      {/* Decline Modal */}
      {showDeclineModal && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-md w-full p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              Decline to Sign
            </h3>
            <p className="text-sm text-gray-500 mb-4">
              Are you sure you want to decline signing this document? You can optionally provide a reason.
            </p>
            <textarea
              value={declineReason}
              onChange={(e) => setDeclineReason(e.target.value)}
              placeholder="Reason for declining (optional)"
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm mb-4"
              rows={3}
            />
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowDeclineModal(false)}
                className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={() => void handleDecline()}
                className="px-4 py-2 bg-red-600 text-white rounded-md text-sm font-medium hover:bg-red-700"
              >
                Decline
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Legal notice */}
      <footer className="bg-white border-t mt-8">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <p className="text-xs text-gray-500 text-center">
            By clicking "Complete Signing", you agree that your electronic signature is legally binding
            under the ESIGN Act (USA) and eIDAS Regulation (EU). Your signature, IP address, and timestamp
            will be recorded for legal compliance.
          </p>
        </div>
      </footer>
    </div>
  );
}

interface SigningFieldProps {
  field: DocumentField;
  signatureData?: string;
  value: string;
  onSignatureClick: () => void;
  onValueChange: (value: string) => void;
}

function SigningField({
  field,
  signatureData,
  value,
  onSignatureClick,
  onValueChange,
}: SigningFieldProps) {
  const isSignatureField = field.field_type === 'signature' || field.field_type === 'initial';

  return (
    <div
      className={`absolute border-2 rounded ${
        isSignatureField
          ? signatureData
            ? 'border-green-500 bg-green-50'
            : 'border-purple-500 bg-purple-50 cursor-pointer hover:bg-purple-100'
          : 'border-amber-500 bg-amber-50'
      }`}
      style={{
        left: field.x,
        top: field.y,
        width: field.width,
        height: field.height,
      }}
      onClick={isSignatureField && !signatureData ? onSignatureClick : undefined}
    >
      {isSignatureField ? (
        signatureData ? (
          <img
            src={signatureData}
            alt="Signature"
            className="w-full h-full object-contain p-1"
          />
        ) : (
          <div className="flex items-center justify-center h-full text-purple-600 text-sm">
            Click to sign
          </div>
        )
      ) : field.field_type === 'date' ? (
        <input
          type="text"
          value={value}
          onChange={(e) => onValueChange(e.target.value)}
          className="w-full h-full px-2 bg-transparent text-gray-800 text-sm border-none outline-none"
          style={{ fontSize: field.font_size ?? 12 }}
        />
      ) : (
        <input
          type="text"
          value={value}
          onChange={(e) => onValueChange(e.target.value)}
          placeholder="Enter text"
          className="w-full h-full px-2 bg-transparent text-gray-800 text-sm border-none outline-none"
          style={{ fontSize: field.font_size ?? 12 }}
        />
      )}
    </div>
  );
}
