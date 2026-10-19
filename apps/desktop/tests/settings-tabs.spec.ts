import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('settings sections and select labels follow the active tab', async ({ page }) => {
  await page.goto('/#/settings');

  const tabs = page.getByRole('tablist', { name: 'Settings sections' });
  await expect(tabs).toHaveAttribute('aria-orientation', 'vertical');
  await expect(tabs.getByRole('tab', { name: 'Workspace' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(page.getByRole('button', { name: 'Create issue' })).toHaveCount(0);
  const save = await page.getByRole('button', { name: 'Save names' }).boundingBox();
  const name = await page.getByRole('textbox', { name: 'Workspace name' }).boundingBox();
  expect(Math.abs(save!.x + save!.width - name!.x - name!.width)).toBeLessThanOrEqual(1);

  await tabs.getByRole('tab', { name: 'Issues' }).click();
  await expect(page).toHaveURL(/#\/settings#issues$/);
  await expect(
    page.getByRole('combobox', { name: 'Issue identifier move preference' }),
  ).toContainText('Ask every time');
  await page.getByRole('combobox', { name: 'Issue identifier move preference' }).click();
  await page.getByRole('option', { name: 'Keep current identifier' }).click();
  await expect(
    page.getByRole('combobox', { name: 'Issue identifier move preference' }),
  ).toContainText('Keep current identifier');
  await expect(page.getByRole('heading', { name: 'Issue estimates' })).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Workspace name' })).toHaveCount(0);

  await page.goto('/#/settings#labels');
  await expect(tabs.getByRole('tab', { name: 'Labels' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByRole('button', { name: 'New label' })).toBeVisible();

  await page.setViewportSize({ width: 560, height: 720 });
  await expect(tabs).toHaveAttribute('aria-orientation', 'horizontal');
  await expect(page.getByRole('button', { name: 'New label' })).toBeVisible();
});
