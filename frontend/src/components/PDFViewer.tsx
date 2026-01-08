import { useEffect, useRef, useState, useCallback } from 'react';
import * as pdfjs from 'pdfjs-dist';
import type { PDFDocumentProxy, PDFPageProxy } from 'pdfjs-dist';

// Configure PDF.js worker
pdfjs.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjs.version}/pdf.worker.min.mjs`;

interface PDFViewerProps {
  url: string;
  scale?: number;
  onPageRender?: (pageNum: number, canvas: HTMLCanvasElement) => void;
  renderOverlay?: (pageNum: number, dimensions: { width: number; height: number }) => React.ReactNode;
}

interface PageInfo {
  pageNum: number;
  width: number;
  height: number;
}

export default function PDFViewer({
  url,
  scale = 1.5,
  onPageRender,
  renderOverlay,
}: PDFViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [pdf, setPdf] = useState<PDFDocumentProxy | null>(null);
  const [pages, setPages] = useState<PageInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const canvasRefs = useRef<Map<number, HTMLCanvasElement>>(new Map());

  const renderPage = useCallback(
    async (page: PDFPageProxy, pageNum: number) => {
      const canvas = canvasRefs.current.get(pageNum);
      if (!canvas) return;

      const viewport = page.getViewport({ scale });
      canvas.height = viewport.height;
      canvas.width = viewport.width;

      const context = canvas.getContext('2d');
      if (!context) return;

      await page.render({
        canvasContext: context,
        viewport,
      }).promise;

      onPageRender?.(pageNum, canvas);
    },
    [scale, onPageRender]
  );

  useEffect(() => {
    let cancelled = false;

    const loadPdf = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const loadingTask = pdfjs.getDocument(url);
        const pdfDoc = await loadingTask.promise;

        if (cancelled) return;

        setPdf(pdfDoc);

        const pageInfos: PageInfo[] = [];
        for (let i = 1; i <= pdfDoc.numPages; i++) {
          const page = await pdfDoc.getPage(i);
          const viewport = page.getViewport({ scale });
          pageInfos.push({
            pageNum: i,
            width: viewport.width,
            height: viewport.height,
          });
        }

        if (!cancelled) {
          setPages(pageInfos);
          setIsLoading(false);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to load PDF');
          setIsLoading(false);
        }
      }
    };

    void loadPdf();

    return () => {
      cancelled = true;
    };
  }, [url, scale]);

  useEffect(() => {
    if (!pdf || pages.length === 0) return;

    const renderAllPages = async () => {
      for (const pageInfo of pages) {
        const page = await pdf.getPage(pageInfo.pageNum);
        await renderPage(page, pageInfo.pageNum);
      }
    };

    void renderAllPages();
  }, [pdf, pages, renderPage]);

  if (isLoading) {
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

  return (
    <div ref={containerRef} className="pdf-container">
      {pages.map((pageInfo) => (
        <div
          key={pageInfo.pageNum}
          className="pdf-page relative mb-4"
          style={{ width: pageInfo.width, height: pageInfo.height }}
        >
          <canvas
            ref={(el) => {
              if (el) canvasRefs.current.set(pageInfo.pageNum, el);
            }}
          />
          {renderOverlay && (
            <div className="absolute inset-0">
              {renderOverlay(pageInfo.pageNum, {
                width: pageInfo.width,
                height: pageInfo.height,
              })}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
