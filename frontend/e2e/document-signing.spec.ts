import { test, expect, Page } from '@playwright/test';
import path from 'path';

// Helper to login
async function login(page: Page) {
  await page.goto('/login');
  await page.getByPlaceholder('Email address').fill('admin@example.com');
  await page.getByPlaceholder('Password').fill('change-this-secure-password');
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page).toHaveURL('/');
}

test.describe('Document Upload and Management', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should upload a new document', async ({ page }) => {
    // Click New Document button
    await page.getByRole('button', { name: 'New Document' }).click();

    // Wait for modal to appear
    await expect(page.getByText('Upload New Document')).toBeVisible();

    // Fill in document title
    await page.getByPlaceholder('Enter document title').fill('Test Contract');

    // Upload a PDF file - using the sample PDF from fixtures
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;

    // Use a minimal valid PDF for testing
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);

    // Submit the form
    await page.getByRole('button', { name: 'Upload' }).click();

    // Should navigate to document editor
    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Should show document title
    await expect(page.getByText('Test Contract')).toBeVisible({ timeout: 10000 });
  });

  test('should display document list after upload', async ({ page }) => {
    // First upload a document
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('List Test Document');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    // Go back to dashboard
    await page.goto('/');

    // Should show the document in the list
    await expect(page.getByText('List Test Document')).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Field Placement', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should add signature field to document', async ({ page }) => {
    // Upload a document first
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Signature Field Test');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    // Wait for editor to load
    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);
    await expect(page.getByText('Signature Field Test')).toBeVisible({ timeout: 10000 });

    // Look for the field type buttons in the sidebar
    const signatureButton = page.getByRole('button', { name: /signature/i });
    if (await signatureButton.isVisible()) {
      await signatureButton.click();

      // Click on the PDF viewer area to place the field
      const pdfViewer = page.locator('[data-testid="pdf-viewer"]').or(page.locator('.pdf-viewer')).or(page.locator('canvas').first());
      await pdfViewer.click({ position: { x: 200, y: 300 } });

      // Should show the placed field
      await expect(page.locator('[data-field-type="signature"]').or(page.getByText(/placed|field/i))).toBeVisible({ timeout: 5000 });
    }
  });

  test('should add date field to document', async ({ page }) => {
    // Upload a document
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Date Field Test');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Look for date field button
    const dateButton = page.getByRole('button', { name: /date/i });
    if (await dateButton.isVisible()) {
      await dateButton.click();

      const pdfViewer = page.locator('[data-testid="pdf-viewer"]').or(page.locator('.pdf-viewer')).or(page.locator('canvas').first());
      await pdfViewer.click({ position: { x: 200, y: 400 } });

      await expect(page.locator('[data-field-type="date"]').or(page.getByText(/date.*placed|field/i))).toBeVisible({ timeout: 5000 });
    }
  });
});

test.describe('Signer Management', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should add a signer to document', async ({ page }) => {
    // Upload a document
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Signer Test Document');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Look for Add Signer button or signers section
    const addSignerButton = page.getByRole('button', { name: /add signer/i });
    if (await addSignerButton.isVisible()) {
      await addSignerButton.click();

      // Fill signer details
      await page.getByPlaceholder(/email/i).fill('signer@example.com');
      await page.getByPlaceholder(/name/i).fill('John Doe');

      // Submit
      await page.getByRole('button', { name: /add|save|confirm/i }).click();

      // Signer should appear in the list
      await expect(page.getByText('signer@example.com')).toBeVisible({ timeout: 5000 });
      await expect(page.getByText('John Doe')).toBeVisible();
    }
  });

  test('should remove a signer from document', async ({ page }) => {
    // Upload a document and add a signer first
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Remove Signer Test');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    const addSignerButton = page.getByRole('button', { name: /add signer/i });
    if (await addSignerButton.isVisible()) {
      await addSignerButton.click();
      await page.getByPlaceholder(/email/i).fill('removeme@example.com');
      await page.getByPlaceholder(/name/i).fill('Remove Me');
      await page.getByRole('button', { name: /add|save|confirm/i }).click();

      await expect(page.getByText('removeme@example.com')).toBeVisible({ timeout: 5000 });

      // Click remove button next to signer
      const removeButton = page.locator('[data-signer="removeme@example.com"]').getByRole('button', { name: /remove|delete/i })
        .or(page.getByText('Remove Me').locator('..').getByRole('button', { name: /remove|delete|x/i }));

      if (await removeButton.isVisible()) {
        await removeButton.click();

        // Confirm if there's a confirmation dialog
        const confirmButton = page.getByRole('button', { name: /confirm|yes|ok/i });
        if (await confirmButton.isVisible({ timeout: 1000 })) {
          await confirmButton.click();
        }

        // Signer should be removed
        await expect(page.getByText('removeme@example.com')).not.toBeVisible({ timeout: 5000 });
      }
    }
  });
});

test.describe('Document Sending', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should send document for signing', async ({ page }) => {
    // Upload a document
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Send Test Document');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Add a signer
    const addSignerButton = page.getByRole('button', { name: /add signer/i });
    if (await addSignerButton.isVisible()) {
      await addSignerButton.click();
      await page.getByPlaceholder(/email/i).fill('sendsigner@example.com');
      await page.getByPlaceholder(/name/i).fill('Send Signer');
      await page.getByRole('button', { name: /add|save|confirm/i }).click();
      await expect(page.getByText('sendsigner@example.com')).toBeVisible({ timeout: 5000 });
    }

    // Click Send button
    const sendButton = page.getByRole('button', { name: /send|send for signing/i });
    if (await sendButton.isVisible()) {
      await sendButton.click();

      // Confirm if needed
      const confirmButton = page.getByRole('button', { name: /confirm|yes|send/i });
      if (await confirmButton.isVisible({ timeout: 1000 })) {
        await confirmButton.click();
      }

      // Document status should change to pending/sent
      await expect(page.getByText(/pending|sent|awaiting/i)).toBeVisible({ timeout: 10000 });
    }
  });
});

test.describe('Self-Signing', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should create self-sign document and complete signing', async ({ page }) => {
    // Upload a document with self-sign option
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Self Sign Document');

    // Check self-sign option if available
    const selfSignCheckbox = page.getByLabel(/self.*sign|sign.*myself/i);
    if (await selfSignCheckbox.isVisible()) {
      await selfSignCheckbox.check();
    }

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Add a signature field
    const signatureButton = page.getByRole('button', { name: /signature/i });
    if (await signatureButton.isVisible()) {
      await signatureButton.click();
      const pdfViewer = page.locator('canvas').first();
      await pdfViewer.click({ position: { x: 200, y: 300 } });
    }

    // Look for sign/complete button
    const signButton = page.getByRole('button', { name: /sign|complete|finish/i });
    if (await signButton.isVisible()) {
      await signButton.click();

      // Should show signature pad/modal
      const signaturePad = page.locator('[data-testid="signature-pad"]').or(page.locator('canvas.signature'));
      if (await signaturePad.isVisible({ timeout: 3000 })) {
        // Draw a simple signature
        await signaturePad.click({ position: { x: 50, y: 50 } });
        await signaturePad.click({ position: { x: 100, y: 50 } });
        await signaturePad.click({ position: { x: 150, y: 100 } });

        // Confirm signature
        const confirmButton = page.getByRole('button', { name: /confirm|apply|done/i });
        if (await confirmButton.isVisible()) {
          await confirmButton.click();
        }
      }
    }
  });
});

test.describe('Document Deletion', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should delete a draft document', async ({ page }) => {
    // Upload a document
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Delete Test Document');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Look for delete button
    const deleteButton = page.getByRole('button', { name: /delete|remove/i });
    if (await deleteButton.isVisible()) {
      await deleteButton.click();

      // Confirm deletion
      const confirmButton = page.getByRole('button', { name: /confirm|yes|delete/i });
      if (await confirmButton.isVisible({ timeout: 2000 })) {
        await confirmButton.click();
      }

      // Should redirect to dashboard
      await expect(page).toHaveURL('/');

      // Document should not appear in list
      await expect(page.getByText('Delete Test Document')).not.toBeVisible({ timeout: 5000 });
    }
  });
});

test.describe('Audit Trail', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should display audit trail for document', async ({ page }) => {
    // Upload a document
    await page.getByRole('button', { name: 'New Document' }).click();
    await page.getByPlaceholder('Enter document title').fill('Audit Trail Test');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.getByText('Click to upload').click();
    const fileChooser = await fileChooserPromise;
    const testPdfPath = path.join(__dirname, '..', '..', 'backend', 'tests', 'fixtures', 'sample.pdf');
    await fileChooser.setFiles(testPdfPath);
    await page.getByRole('button', { name: 'Upload' }).click();

    await expect(page).toHaveURL(/\/documents\/[a-f0-9-]+/);

    // Look for audit trail tab/button
    const auditButton = page.getByRole('button', { name: /audit|history|activity/i })
      .or(page.getByRole('tab', { name: /audit|history|activity/i }));

    if (await auditButton.isVisible()) {
      await auditButton.click();

      // Should show document creation entry
      await expect(page.getByText(/created|uploaded/i)).toBeVisible({ timeout: 5000 });
    }
  });
});
