import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('workspace settings create a status with the shared dialog', async ({ page }) => {
  await page.goto('/#/settings#workflow');
  await page.getByRole('button', { name: 'New status' }).click();
  const editor = page.getByRole('dialog', { name: 'Create status' });
  await editor.getByRole('textbox', { name: 'Group name' }).fill('Verification');
  await editor.getByRole('button', { name: 'Create status' }).click();
  await expect(editor).toBeHidden();
  await expect(page.getByRole('button', { name: 'Edit Verification' })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('button', { name: 'Edit Verification' })).toBeVisible();
});

test('custom workflow category, description, and default status persist in the preview', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Add group' }).click();
  await page.getByRole('textbox', { name: 'Group name' }).fill('Ready');
  await page.getByRole('combobox', { name: 'New group category' }).click();
  await page.getByRole('option', { name: 'Unstarted' }).click();
  await page.getByRole('textbox', { name: 'New group description' }).fill('Prepared for work');
  await page.getByRole('button', { name: 'Add group' }).click();
  const lane = page.getByRole('region', { name: 'Ready lane' });
  await lane.getByRole('button', { name: 'Edit Ready group' }).click();
  await expect(page.getByRole('combobox', { name: 'Group category' })).toContainText('Unstarted');
  await expect(page.getByRole('textbox', { name: 'Group description' })).toHaveValue(
    'Prepared for work',
  );
  await page.getByRole('button', { name: 'Make default status' }).click();
  await page.reload();
  await page
    .getByRole('region', { name: 'Ready lane' })
    .getByRole('button', { name: 'Edit Ready group' })
    .click();
  await expect(page.getByRole('button', { name: 'Archive group' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Delete empty group' })).toBeDisabled();
  await page.getByRole('button', { name: 'Create issue' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  await expect(dialog.getByRole('combobox', { name: 'Status' })).toContainText('Ready');
});

test('a custom completed status follows completed issue visibility', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Add group' }).click();
  await page.getByRole('textbox', { name: 'Group name' }).fill('Accepted');
  await page.getByRole('combobox', { name: 'New group category' }).click();
  await page.getByRole('option', { name: 'Completed' }).click();
  await page.getByRole('button', { name: 'Add group', exact: true }).click();
  await page
    .getByRole('region', { name: 'Accepted lane' })
    .getByRole('button', { name: 'New issue in Accepted' })
    .click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  await dialog.getByRole('textbox', { name: 'Title' }).fill('Accepted work');
  await dialog.getByRole('button', { name: 'Create issue' }).click();
  await expect(
    page
      .getByRole('region', { name: 'Accepted lane' })
      .getByRole('button', { name: 'Accepted work' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Active', exact: true }).click();
  await expect(
    page
      .getByRole('region', { name: 'Accepted lane' })
      .getByRole('button', { name: 'Accepted work' }),
  ).toHaveCount(0);
  await page.getByRole('button', { name: 'All', exact: true }).click();
  await expect(
    page
      .getByRole('region', { name: 'Accepted lane' })
      .getByRole('button', { name: 'Accepted work' }),
  ).toBeVisible();
});

test('renaming a workflow group keeps archived issues in that group', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: 'Add group' }).click();
  await page.getByRole('textbox', { name: 'Group name' }).fill('Review');
  await page.getByRole('button', { name: 'Add group', exact: true }).click();
  await page.getByRole('button', { name: 'New issue in Review' }).click();
  const composer = page.getByRole('dialog', { name: 'Create issue' });
  await composer.getByRole('textbox', { name: 'Title' }).fill('Review archive');
  await composer.getByRole('button', { name: 'Create issue' }).click();

  await page.getByRole('button', { name: 'Review archive' }).click({ button: 'right' });
  await page.getByRole('menuitem', { name: 'Archive issue' }).click();
  await page
    .getByRole('region', { name: 'Review lane' })
    .getByRole('button', { name: 'Edit Review group' })
    .click();
  await page.getByRole('textbox', { name: 'Edit group name' }).fill('QA review');
  await page.getByRole('button', { name: 'Save group' }).click();

  await page.getByRole('link', { name: /Archives/ }).click();
  const archived = page.getByRole('main', { name: 'Archived issues' });
  await expect(archived.getByText('Review archive')).toBeVisible();
  await archived
    .getByRole('listitem')
    .filter({ hasText: 'Review archive' })
    .getByRole('button', { name: /Restore/ })
    .click();
  await page.getByRole('link', { name: /All issues/ }).click();
  await expect(
    page.getByRole('region', { name: 'QA review lane' }).getByRole('button', {
      name: 'Review archive',
    }),
  ).toBeVisible();
});
