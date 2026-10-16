import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('an archived issue leaves active views and restores after a restart', async ({ page }) => {
  await page.goto('/#/issues');
  const row = page.getByRole('button', { name: /Open APP-123:/ });
  await row.click({ button: 'right' });
  await page.getByRole('menuitem', { name: 'Archive issue' }).click();
  await expect(row).toHaveCount(0);

  await page.getByRole('link', { name: /Archives/ }).click();
  const archive = page.getByRole('main', { name: 'Archived issues' });
  await expect(archive.getByText('Persist issues to local SQLite')).toBeVisible();
  await page.reload();
  await expect(archive.getByText('Persist issues to local SQLite')).toBeVisible();
  await archive.getByRole('button', { name: 'Restore APP-123' }).click();
  await expect(archive.getByText('Persist issues to local SQLite')).toHaveCount(0);
  await page.getByRole('link', { name: /All issues/ }).click();
  await expect(row).toBeVisible();
});

test('archiving a parent also archives its sub-issues', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  await page.getByRole('button', { name: 'Add sub-issue' }).click();
  const composer = page.getByRole('dialog', { name: 'Create sub-issue' });
  await composer.getByRole('textbox', { name: 'Title' }).fill('Archive child');
  await composer.getByRole('button', { name: 'Create sub-issue' }).click();
  await expect(page.getByRole('button', { name: /Archive child/ })).toBeVisible();
  await page.getByRole('button', { name: 'Archive issue' }).click();
  await page
    .getByRole('dialog', { name: 'Archive APP-123' })
    .getByRole('button', { name: 'Archive issue' })
    .click();
  await expect(page).toHaveURL(/#\/issues$/);
  await page.goto('/#/archives');
  const archive = page.getByRole('main', { name: 'Archived issues' });
  await expect(archive.getByText('Archive child')).toBeVisible();
  await expect(archive.getByRole('listitem').filter({ hasText: 'Archive child' })).toContainText(
    'Restore as top-level',
  );
  await archive
    .getByRole('listitem')
    .filter({ hasText: 'Archive child' })
    .getByRole('button', { name: /^Restore/ })
    .click();
  await expect(archive.getByText('Archive child')).toHaveCount(0);
  await expect(archive.getByText('Persist issues to local SQLite')).toBeVisible();
  await archive.getByRole('button', { name: 'Restore APP-123' }).click();
  await expect(archive.getByText('Archive child')).toHaveCount(0);
});

test('archiving a parent can keep its sub-issues open and archived issues can be deleted', async ({
  page,
}) => {
  await page.goto('/#/issues/APP-123/full');
  await page.getByRole('button', { name: 'Add sub-issue' }).click();
  const composer = page.getByRole('dialog', { name: 'Create sub-issue' });
  await composer.getByRole('textbox', { name: 'Title' }).fill('Keep this child');
  await composer.getByRole('button', { name: 'Create sub-issue' }).click();

  await page.getByRole('button', { name: 'Archive issue' }).click();
  const confirm = page.getByRole('dialog', { name: 'Archive APP-123' });
  await confirm.getByRole('radio', { name: /Make sub-issues top-level/ }).click();
  await confirm.getByRole('button', { name: 'Archive issue' }).click();
  await page.goto('/#/issues');
  await expect(page.getByRole('button', { name: /Keep this child/ })).toBeVisible();

  await page.goto('/#/archives');
  const archive = page.getByRole('main', { name: 'Archived issues' });
  await archive.getByRole('button', { name: 'Delete APP-123' }).click();
  await page
    .getByRole('dialog', { name: 'Delete APP-123' })
    .getByRole('button', { name: 'Delete permanently' })
    .click();
  await expect(archive.getByText('Persist issues to local SQLite')).toHaveCount(0);
});
