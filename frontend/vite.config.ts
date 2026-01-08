import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'pdf-worker': ['pdfjs-dist/build/pdf.worker.mjs'],
          'pdf-lib': ['pdfjs-dist'],
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'dnd-kit': ['@dnd-kit/core', '@dnd-kit/sortable', '@dnd-kit/utilities'],
          'utils': ['date-fns', 'zustand'],
        },
      },
    },
  },
});
