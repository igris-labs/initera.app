import { expect, test } from '@playwright/test';

test('first-run intro waits for Get Started before showing the form', async ({ page }) => {
  await page.goto('/#/issues');
  const start = page.getByRole('button', { name: 'Get Started' });
  await expect(start).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Set up your workspace' })).toBeHidden();
  await start.click();
  await expect(page.getByRole('heading', { name: 'Set up your workspace' })).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Your name' })).toBeFocused();
});

test('reduced motion presents the form immediately', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/#/issues');
  await expect(page.getByRole('heading', { name: 'Set up your workspace' })).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Your name' })).toBeFocused();
  await expect(page.getByRole('button', { name: 'Get Started' })).toBeHidden();
});

test('first launch collects identity and keeps navigation linked', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Get Started' }).click();
  await expect(page.getByRole('heading', { name: 'Set up your workspace' })).toBeVisible();
  await expect(page.getByText('Welcome', { exact: true })).toBeVisible();
  const nameInput = page.getByRole('textbox', { name: 'Your name' });
  await nameInput.focus();
  const focusShadow = await nameInput.evaluate((input) => getComputedStyle(input).boxShadow);
  expect(focusShadow).not.toMatch(/\b[1-9]\d*px\b/);
  await nameInput.fill('Rahul Vashishtha');
  await page.getByRole('textbox', { name: 'Workspace name' }).fill('Igris Labs');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByRole('navigation', { name: 'Primary' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Help and settings' })).toContainText(
    'Rahul Vashishtha',
  );
  await page.getByRole('link', { name: 'Triage' }).click();
  await page.getByRole('link', { name: 'Igris Labs' }).click();
  await expect(page).toHaveURL(/#\/issues$/);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Help and settings' })).toContainText(
    'Rahul Vashishtha',
  );
  await page.goto('/#/settings');
  await page.getByRole('textbox', { name: 'Your name' }).fill('Rahul');
  await page.getByRole('textbox', { name: 'Workspace name' }).fill('Product Studio');
  await page.getByRole('button', { name: 'Save names' }).click();
  await expect(page.getByRole('button', { name: 'Help and settings' })).toContainText('Rahul');
  await expect(page.getByRole('link', { name: 'Product Studio' })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('link', { name: 'Product Studio' })).toBeVisible();
});
