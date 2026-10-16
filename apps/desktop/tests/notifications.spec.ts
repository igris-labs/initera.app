import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('success notification has a clear type icon and aligned dismiss control', async ({ page }) => {
  await page.goto('/#/projects');
  await page.getByRole('button', { name: 'New project' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Toast audit');
  await dialog.getByRole('button', { name: 'Create project' }).click();
  const toast = page
    .locator('.akaza-toast[data-akaza-type="success"]')
    .filter({ hasText: 'Project created' });
  await expect(toast).toBeVisible();
  await expect(toast.locator('svg.tabler-icon-circle-check')).toBeVisible();
  const [title, dismiss] = await Promise.all([
    toast.getByText('Project created').boundingBox(),
    toast.getByRole('button', { name: 'Dismiss notification' }).boundingBox(),
  ]);
  expect(title).toBeTruthy();
  expect(dismiss).toBeTruthy();
  expect(dismiss!.x).toBeGreaterThan(title!.x + title!.width);
  await expect(toast).toHaveCSS(
    'border-left-color',
    await toast
      .locator('svg.tabler-icon-circle-check')
      .evaluate((icon) => getComputedStyle(icon).color),
  );
  await toast.getByRole('button', { name: 'Dismiss notification' }).click();
  await expect(toast).toHaveCount(0);
});

test('info, warning, and error notifications use distinct colors', async ({ page }) => {
  await page.goto('/#/issues');
  await page.evaluate(async () => {
    const { notifications } = await import('/src/lib/notifications.ts');
    notifications.add({ title: 'Information example', type: 'info' });
    notifications.add({ title: 'Warning example', type: 'warning' });
    notifications.add({ title: 'Error example', type: 'error' });
  });
  const colors: string[] = [];
  for (const type of ['info', 'warning', 'error']) {
    const toast = page.locator(`.akaza-toast[data-akaza-type="${type}"]`);
    await expect(toast).toBeVisible();
    colors.push(await toast.evaluate((element) => getComputedStyle(element).borderLeftColor));
    await expect(toast.locator('svg').first()).toBeVisible();
  }

  expect(new Set(colors).size).toBe(3);
});
