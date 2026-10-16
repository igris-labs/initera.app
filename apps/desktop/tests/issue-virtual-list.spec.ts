import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('large grouped issue list keeps a bounded row count while scrolling', async ({ page }) => {
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

  const list = page.getByLabel('Issue list');
  await expect(list).toBeVisible();
  await expect(page.getByRole('button', { name: /Todo 10000/ })).toBeVisible();
  expect(await page.locator('[data-issue-id]').count()).toBeLessThan(80);

  await list.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });

  await expect(page.locator('[data-issue-id="LOAD-10000"]')).toBeVisible();
  expect(await page.locator('[data-issue-id]').count()).toBeLessThan(80);
});

test('large list retains pointer reordering', async ({ page }) => {
  await page.addInitScript(() => {
    const items = Array.from({ length: 250 }, (_, index) => ({
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
  const first = page.locator('[data-issue-id="LOAD-1"]');
  const second = page.locator('[data-issue-id="LOAD-2"]');
  await expect(first).toBeVisible();
  await first.dragTo(second, { targetPosition: { x: 40, y: 30 } });

  await expect(page.locator('[data-issue-id]').first()).toHaveAttribute('data-issue-id', 'LOAD-2');
  await expect(page.locator('[data-issue-id]').nth(1)).toHaveAttribute('data-issue-id', 'LOAD-1');
});

test('large list accepts drops into an empty status group', async ({ page }) => {
  await page.addInitScript(() => {
    const items = Array.from({ length: 250 }, (_, index) => ({
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

  const first = page.locator('[data-issue-id="LOAD-250"]');
  const target = page.getByRole('button', { name: 'In Progress 0' });

  await page.getByLabel('Issue list').evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });

  await expect(first).toBeVisible();
  await expect(target).toBeVisible();
  await first.dragTo(target);

  await expect(page.getByRole('button', { name: 'In Progress 1' })).toBeVisible();
  await expect(first).toBeVisible();
});

test('large board cards keep pointer reordering without per-card layout observers', async ({
  page,
}) => {
  await page.addInitScript(() => {
    const items = Array.from({ length: 250 }, (_, index) => ({
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
  const first = todo.locator('[data-issue-id="LOAD-1"]');
  const second = todo.locator('[data-issue-id="LOAD-2"]');

  expect(await todo.locator('[data-issue-id]').count()).toBeLessThan(25);

  const from = await first.boundingBox();
  const to = await second.boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();

  await page.mouse.move(from!.x + 40, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(180);
  await page.mouse.move(to!.x + 40, to!.y + to!.height * 0.8, { steps: 12 });
  await page.mouse.up();

  await expect(todo.locator('[data-issue-id]').first()).toHaveAttribute('data-issue-id', 'LOAD-2');
});
