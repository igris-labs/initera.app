import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('nodes reorder, archive, dissolve, and restore without losing issues', async ({ page }) => {
  await page.goto('/#/projects/Desktop%20app/structure');
  await page.getByRole('button', { name: 'Add node' }).click();
  const nodeDialog = page.getByRole('dialog', { name: 'Add project node' });
  await nodeDialog.getByRole('textbox', { name: 'Node name' }).fill('Desktop shell');
  await nodeDialog.getByRole('button', { name: 'Add node' }).click();

  await page.getByRole('button', { name: 'Add child to Desktop shell' }).click();
  await nodeDialog.getByRole('textbox', { name: 'Node name' }).fill('Quick capture');
  await nodeDialog.getByRole('combobox', { name: 'Node kind' }).click();
  await page.getByRole('option', { name: 'Feature' }).click();
  await nodeDialog.getByRole('button', { name: 'Add node' }).click();

  const featureRow = page.getByText('Quick capture', { exact: true }).locator('..').locator('..');
  await featureRow.getByRole('button', { name: 'Issues' }).click();
  const issuesDialog = page.getByRole('dialog', { name: 'Issues in Quick capture' });
  await issuesDialog.getByRole('textbox', { name: 'Find project issue' }).fill('APP-121');
  await issuesDialog.getByRole('button', { name: /APP-121/ }).click();
  await issuesDialog.getByRole('button', { name: 'Done' }).click();
  await expect(featureRow).toContainText('0/1 done');

  await page.reload();
  await expect(page.getByText('Quick capture', { exact: true })).toBeVisible();
  await expect(featureRow).toContainText('0/1 done');
  await page.getByRole('button', { name: 'Remove Desktop shell' }).click();
  await page
    .getByRole('dialog', { name: 'Remove Desktop shell?' })
    .getByRole('button', { name: 'Archive subtree', exact: true })
    .click();
  await expect(page.getByRole('button', { name: 'Add child to Quick capture' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Restore Desktop shell' }).click();
  await expect(featureRow).toContainText('0/1 done');
  await page.getByRole('button', { name: 'Remove Desktop shell' }).click();
  await page
    .getByRole('dialog', { name: 'Remove Desktop shell?' })
    .getByRole('button', { name: 'Dissolve this node' })
    .click();
  await page
    .getByRole('dialog', { name: 'Remove Desktop shell?' })
    .getByRole('button', { name: 'Dissolve node' })
    .click();
  await expect(page.getByText('Quick capture', { exact: true })).toBeVisible();
  await expect(featureRow).toContainText('0/1 done');
  await page.getByRole('button', { name: 'Restore Desktop shell memberships' }).click();
  await expect(page.getByText('Desktop shell', { exact: true })).toBeVisible();
  await page.getByRole('link', { name: 'Issues', exact: true }).click();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
});
