import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('should display login page', async ({ page }) => {
    await page.goto('/login');
    await expect(page.getByText('SignVault')).toBeVisible();
    await expect(page.getByPlaceholder('Email address')).toBeVisible();
    await expect(page.getByPlaceholder('Password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
  });

  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/login');
    await page.getByPlaceholder('Email address').fill('wrong@example.com');
    await page.getByPlaceholder('Password').fill('wrongpassword');
    await page.getByRole('button', { name: 'Sign in' }).click();

    // Should show error message
    await expect(page.getByText(/unauthorized|invalid/i)).toBeVisible({ timeout: 10000 });
  });

  test('should redirect to login when accessing protected route', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveURL(/.*login/);
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    await page.goto('/login');
    await page.getByPlaceholder('Email address').fill('admin@example.com');
    await page.getByPlaceholder('Password').fill('change-this-secure-password');
    await page.getByRole('button', { name: 'Sign in' }).click();

    // Should redirect to dashboard
    await expect(page).toHaveURL('/');
    await expect(page.getByText('Documents')).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.getByPlaceholder('Email address').fill('admin@example.com');
    await page.getByPlaceholder('Password').fill('change-this-secure-password');
    await page.getByRole('button', { name: 'Sign in' }).click();
    await expect(page).toHaveURL('/');
  });

  test('should display dashboard after login', async ({ page }) => {
    await expect(page.getByText('Documents')).toBeVisible();
    await expect(page.getByRole('button', { name: 'New Document' })).toBeVisible();
  });

  test('should open upload modal when clicking New Document', async ({ page }) => {
    await page.getByRole('button', { name: 'New Document' }).click();
    await expect(page.getByText('Upload New Document')).toBeVisible();
    await expect(page.getByPlaceholder('Enter document title')).toBeVisible();
  });

  test('should logout successfully', async ({ page }) => {
    await page.getByText('Logout').click();
    await expect(page).toHaveURL(/.*login/);
  });
});
