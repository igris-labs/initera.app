import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('blocker relation is visible from both issues and survives restart', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const detail = page.getByRole('region', { name: 'Full issue view' });
  await detail.getByRole('button', { name: 'Blocks', exact: true }).click();
  await page
    .getByRole('dialog', { name: 'Link issue' })
    .getByRole('option', { name: /APP-121/ })
    .click();
  await expect(
    detail.getByRole('list', { name: 'Issue relations' }).getByRole('listitem'),
  ).toContainText('Blocks');
  await expect(detail.getByRole('button', { name: /APP-121 ·/ })).toBeVisible();

  await page.reload();
  await expect(
    page
      .getByRole('region', { name: 'Full issue view' })
      .getByRole('button', { name: /APP-121 ·/ }),
  ).toBeVisible();
  await page.goto('/#/issues/APP-121/full');
  await expect(
    page
      .getByRole('region', { name: 'Full issue view' })
      .getByRole('list', { name: 'Issue relations' })
      .getByRole('listitem'),
  ).toContainText('Blocked by');
  await expect(
    page
      .getByRole('region', { name: 'Full issue view' })
      .getByRole('button', { name: /APP-123 ·/ }),
  ).toBeVisible();
});

test('a blocker cycle is rejected and the existing link can be removed', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const detail = page.getByRole('region', { name: 'Full issue view' });
  await detail.getByRole('button', { name: 'Blocks', exact: true }).click();
  await page
    .getByRole('dialog', { name: 'Link issue' })
    .getByRole('option', { name: /APP-121/ })
    .click();
  await page.goto('/#/issues/APP-121/full');
  const other = page.getByRole('region', { name: 'Full issue view' });
  await other.getByRole('button', { name: 'Blocks', exact: true }).click();
  const picker = page.getByRole('dialog', { name: 'Link issue' });
  await picker.getByRole('option', { name: /APP-123/ }).click();
  await expect(picker.getByRole('alert')).toContainText('dependency cycle');
  await page.keyboard.press('Escape');
  await other.getByRole('button', { name: 'Remove relation to APP-123' }).click();
  await expect(other.getByRole('button', { name: /APP-123 ·/ })).toHaveCount(0);
});

test('a sub-issue shows blocking links inherited from its parent', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const parent = page.getByRole('region', { name: 'Full issue view' });
  await parent.getByRole('button', { name: 'Blocks', exact: true }).click();
  const picker = page.getByRole('dialog', { name: 'Link issue' });
  await picker.getByRole('combobox', { name: 'Search issues for relation' }).fill('APP-121');
  await picker.getByRole('option', { name: /APP-121/ }).click();

  await parent.getByRole('button', { name: 'Add sub-issue' }).click();
  const composer = page.getByRole('dialog', { name: 'Create sub-issue' });
  await composer.getByRole('textbox', { name: 'Title' }).fill('Inherited blocker');
  await composer.getByRole('button', { name: 'Create sub-issue' }).click();
  await parent.getByRole('button', { name: /Inherited blocker/ }).click();
  const child = page.getByRole('region', { name: 'Full issue view' });
  await expect(child.getByRole('list', { name: 'Issue relations' })).toContainText('via APP-123');
});
