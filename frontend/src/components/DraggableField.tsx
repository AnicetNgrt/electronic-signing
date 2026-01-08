import { useState, useRef, useCallback, type MouseEvent } from 'react';
import type { DocumentField, FieldType } from '@/types';

interface DraggableFieldProps {
  field: DocumentField;
  isSelected: boolean;
  onSelect: () => void;
  onUpdate: (updates: { x?: number; y?: number; width?: number; height?: number }) => void;
  onDelete: () => void;
  readOnly?: boolean;
  signerName?: string;
}

const fieldLabels: Record<FieldType, string> = {
  signature: 'Signature',
  date: 'Date',
  text: 'Text',
  initial: 'Initial',
};

const fieldIcons: Record<FieldType, string> = {
  signature: 'M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z',
  date: 'M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
  text: 'M4 6h16M4 12h16M4 18h7',
  initial: 'M13 10V3L4 14h7v7l9-11h-7z',
};

export default function DraggableField({
  field,
  isSelected,
  onSelect,
  onUpdate,
  onDelete,
  readOnly = false,
  signerName,
}: DraggableFieldProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const fieldRef = useRef<HTMLDivElement>(null);
  const startPos = useRef({ x: 0, y: 0, fieldX: 0, fieldY: 0, width: 0, height: 0 });

  const handleMouseDown = useCallback(
    (e: MouseEvent) => {
      if (readOnly) return;
      e.stopPropagation();
      onSelect();

      setIsDragging(true);
      startPos.current = {
        x: e.clientX,
        y: e.clientY,
        fieldX: field.x,
        fieldY: field.y,
        width: field.width,
        height: field.height,
      };

      const handleMouseMove = (moveEvent: globalThis.MouseEvent) => {
        const dx = moveEvent.clientX - startPos.current.x;
        const dy = moveEvent.clientY - startPos.current.y;

        onUpdate({
          x: Math.max(0, startPos.current.fieldX + dx),
          y: Math.max(0, startPos.current.fieldY + dy),
        });
      };

      const handleMouseUp = () => {
        setIsDragging(false);
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    },
    [field.x, field.y, field.width, field.height, onSelect, onUpdate, readOnly]
  );

  const handleResizeMouseDown = useCallback(
    (e: MouseEvent) => {
      if (readOnly) return;
      e.stopPropagation();
      e.preventDefault();

      setIsResizing(true);
      startPos.current = {
        x: e.clientX,
        y: e.clientY,
        fieldX: field.x,
        fieldY: field.y,
        width: field.width,
        height: field.height,
      };

      const handleMouseMove = (moveEvent: globalThis.MouseEvent) => {
        const dx = moveEvent.clientX - startPos.current.x;
        const dy = moveEvent.clientY - startPos.current.y;

        onUpdate({
          width: Math.max(50, startPos.current.width + dx),
          height: Math.max(30, startPos.current.height + dy),
        });
      };

      const handleMouseUp = () => {
        setIsResizing(false);
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    },
    [field.width, field.height, onUpdate, readOnly]
  );

  return (
    <div
      ref={fieldRef}
      className={`field-overlay ${field.field_type} ${isSelected ? 'selected' : ''} ${isDragging || isResizing ? 'cursor-grabbing' : ''}`}
      style={{
        left: field.x,
        top: field.y,
        width: field.width,
        height: field.height,
        fontSize: field.font_size ?? 12,
      }}
      onMouseDown={handleMouseDown}
      onClick={(e) => {
        e.stopPropagation();
        onSelect();
      }}
    >
      <div className="flex flex-col items-center justify-center w-full h-full p-1">
        <svg
          className="w-4 h-4 mb-1"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d={fieldIcons[field.field_type]}
          />
        </svg>
        <span className="text-xs truncate max-w-full">
          {signerName ?? fieldLabels[field.field_type]}
        </span>
        {field.value && (
          <span className="text-xs truncate max-w-full mt-1 opacity-75">
            {field.value}
          </span>
        )}
      </div>

      {isSelected && !readOnly && (
        <>
          <button
            className="absolute -top-2 -right-2 w-5 h-5 bg-red-500 text-white rounded-full flex items-center justify-center text-xs hover:bg-red-600"
            onClick={(e) => {
              e.stopPropagation();
              onDelete();
            }}
          >
            ×
          </button>
          <div
            className="resize-handle se"
            onMouseDown={handleResizeMouseDown}
          />
        </>
      )}
    </div>
  );
}
