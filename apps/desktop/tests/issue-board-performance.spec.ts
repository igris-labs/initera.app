import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('large board keeps its last issue reachable', async ({ page }) => {
  test.setTimeout(60_000);

  await page.addInitScript(() => {
    const items = Array.from({ length: 10_000 }, (_, index) => ({
      id: `LOAD-${index + 1}`,
      title: `Performance issue ${index + 1}`,
      description: '',
      status: 'Todo',
      priority: 'Medium',
      project: 'Unassigned',
      labels: [],
      updated: 'Today',
    }));

    localStorage.setItem('issues', JSON.stringify({ items }));
  });

  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();

  const todo = page.getByRole('region', { name: 'Todo lane' });
  const last = todo.locator('[data-issue-id="LOAD-10000"]');

  await expect(todo.locator('[data-issue-id]')).toHaveCount(12);
  await todo.locator('.overflow-y-auto').evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  await expect(last).toBeVisible();
  expect(await todo.locator('[data-issue-id]').count()).toBeLessThan(25);
});
