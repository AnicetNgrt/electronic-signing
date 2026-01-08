import { useRef, useEffect, useCallback, useState } from 'react';
import SignatureCanvas from 'react-signature-canvas';

interface SignaturePadProps {
  onSave: (signatureData: string) => void;
  onCancel: () => void;
  width?: number;
  height?: number;
}

export default function SignaturePad({
  onSave,
  onCancel,
  width = 500,
  height = 200,
}: SignaturePadProps) {
  const sigCanvasRef = useRef<SignatureCanvas>(null);
  const [isEmpty, setIsEmpty] = useState(true);

  const handleClear = useCallback(() => {
    sigCanvasRef.current?.clear();
    setIsEmpty(true);
  }, []);

  const handleSave = useCallback(() => {
    if (sigCanvasRef.current && !sigCanvasRef.current.isEmpty()) {
      const dataUrl = sigCanvasRef.current.getTrimmedCanvas().toDataURL('image/png');
      onSave(dataUrl);
    }
  }, [onSave]);

  const handleEnd = useCallback(() => {
    setIsEmpty(sigCanvasRef.current?.isEmpty() ?? true);
  }, []);

  useEffect(() => {
    const canvas = sigCanvasRef.current?.getCanvas();
    if (canvas) {
      canvas.style.touchAction = 'none';
    }
  }, []);

  return (
    <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg p-6 max-w-lg w-full">
        <h3 className="text-lg font-medium text-gray-900 mb-4">
          Draw Your Signature
        </h3>
        <p className="text-sm text-gray-500 mb-4">
          Use your mouse or finger to sign in the box below.
        </p>

        <div className="border-2 border-gray-300 rounded-lg overflow-hidden bg-white">
          <SignatureCanvas
            ref={sigCanvasRef}
            canvasProps={{
              width,
              height,
              className: 'signature-pad w-full',
            }}
            backgroundColor="white"
            penColor="black"
            onEnd={handleEnd}
          />
        </div>

        <div className="mt-4 flex justify-between">
          <button
            type="button"
            onClick={handleClear}
            className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
          >
            Clear
          </button>
          <div className="flex gap-3">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleSave}
              disabled={isEmpty}
              className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Save Signature
            </button>
          </div>
        </div>

        <p className="mt-4 text-xs text-gray-500 text-center">
          By signing, you agree that your electronic signature is the legal equivalent of your manual signature.
        </p>
      </div>
    </div>
  );
}
