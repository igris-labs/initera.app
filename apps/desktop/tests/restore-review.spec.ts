import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('restored workspace keeps cycle catch-up review open', async ({ page }) => {
  await page.goto('/#/issues');
  await page.evaluate(async () => {
    const { useWorkspaceStore } = await import('/src/stores/workspace.ts');
    useWorkspaceStore().restoredBackup = true;
  });

  const review = page.getByRole('dialog', { name: 'Review restored workspace' });
  await expect(review).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(review).toBeVisible();
  await page.mouse.click(5, 5);
  await expect(review).toBeVisible();
});
