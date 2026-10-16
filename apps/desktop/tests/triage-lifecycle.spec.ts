import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('accepting a triaged issue moves it into active work', async ({ page }) => {
  await page.goto('/#/triage');
  await page.getByRole('button', { name: /Open LOCAL-3:/ }).click();
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  await detail.getByRole('button', { name: 'Accept from triage' }).click();
  await page.getByRole('button', { name: 'Accept issues' }).click();
  await expect(page).toHaveURL(/#\/issues\/LOCAL-3/);
  await page.goto('/#/issues');
  await expect(page.getByRole('button', { name: /Open LOCAL-3:/ })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('button', { name: /Open LOCAL-3:/ })).toBeVisible();
});

test('declining a triaged issue keeps it in completed history', async ({ page }) => {
  await page.goto('/#/triage');
  await page.getByRole('button', { name: /Open LOCAL-3:/ }).click();
  await page.getByRole('button', { name: 'Decline from triage' }).click();
  await page.getByRole('button', { name: 'Decline issues' }).click();
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'All', exact: true }).click();
  await expect(page.getByRole('button', { name: /Open LOCAL-3:.*Canceled/ })).toBeVisible();
});
