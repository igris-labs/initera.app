import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('workspace labels can be created, archived, trashed, and restored', async ({ page }) => {
  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'New label' }).click();
  const editor = page.getByRole('dialog', { name: 'Create label' });
  await editor.getByRole('textbox', { name: 'Label name' }).fill('Release');
  await editor.getByRole('button', { name: 'Create label' }).click();
  await expect(page.getByRole('button', { name: 'Archive Release' })).toBeVisible();

  await page.getByRole('button', { name: 'Archive Release' }).click();
  await expect(page.getByRole('button', { name: 'Restore Release' })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('button', { name: 'Restore Release' })).toBeVisible();
  await page.getByRole('button', { name: 'Restore Release' }).click();
  await page.getByRole('button', { name: 'Trash Release' }).click();
  await page
    .getByRole('dialog', { name: 'Trash Release?' })
    .getByRole('button', { name: 'Trash label' })
    .click();
  await expect(page.getByText('Trashed', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Restore Release' }).click();
  await expect(page.getByRole('button', { name: 'Archive Release' })).toBeVisible();
});

test('trashed labels keep saved filters unavailable until restored', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Without UI');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  await page.getByRole('button', { name: 'Exclude', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'UI', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Clear excluded UI label' })).toBeVisible();

  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Trash UI' }).click();
  await page
    .getByRole('dialog', { name: 'Trash UI?' })
    .getByRole('button', { name: 'Trash label' })
    .click();
  await page.goto('/#/views');
  await expect(page.getByRole('link', { name: /Without UI/ })).toContainText('Unavailable label');
  await page.getByRole('link', { name: /Without UI/ }).click();
  await expect(page.getByRole('alert')).toContainText('Unavailable label filter: UI');
  await expect(page.locator('[data-issue-id]')).toHaveCount(0);

  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Restore UI' }).click();
  await page.goto('/#/views');
  await page.getByRole('link', { name: /Without UI/ }).click();
  await expect(page.getByRole('alert')).toHaveCount(0);
});
