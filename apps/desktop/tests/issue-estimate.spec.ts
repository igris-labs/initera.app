import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('estimate can be set in the composer and edited from issue details', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  await dialog.getByRole('textbox', { name: 'Title' }).fill('Estimate this work');
  await dialog.getByRole('combobox', { name: 'Estimate' }).click();
  await page.getByRole('option', { name: '5 points' }).click();
  await dialog.getByRole('button', { name: 'Create issue' }).click();
  await page.goto('/#/triage');
  await page.getByRole('button', { name: /Estimate this work/ }).click();
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  await expect(detail.getByRole('combobox', { name: 'Estimate' })).toContainText('5 points');
  await detail.getByRole('combobox', { name: 'Estimate' }).click();
  await page.getByRole('option', { name: '8 points' }).click();
  await page.reload();
  await expect(
    page
      .getByRole('complementary', { name: 'Issue details' })
      .getByRole('combobox', { name: 'Estimate' }),
  ).toContainText('8 points');
});

test('estimate and relation shortcuts open their issue pickers', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  await expect(page.getByRole('combobox', { name: 'Estimate' })).toBeVisible();
  await page.keyboard.press('Shift+E');
  await expect(page.getByRole('option', { name: '5 points' })).toBeVisible();
  await page.keyboard.press('Escape');
  await page.keyboard.press('Shift+R');
  await expect(
    page.getByRole('dialog', { name: 'Link issue' }).getByRole('option', { name: /APP-121/ }),
  ).toBeVisible();
});

test('workspace estimate scale changes issue picker labels without rewriting old estimates', async ({
  page,
}) => {
  await page.goto('/#/settings');
  await page.getByRole('radio', { name: /T-shirt sizes/ }).click();
  await page.goto('/#/issues/APP-123/full');
  await page.getByRole('combobox', { name: 'Estimate' }).click();
  await expect(page.getByRole('option', { name: 'XXXL' })).toBeVisible();
  await page.getByRole('option', { name: /^M\b/ }).click();
  await expect(page.getByRole('combobox', { name: 'Estimate' })).toContainText('M');
  await page.reload();
  await expect(page.getByRole('combobox', { name: 'Estimate' })).toContainText('M');
});
