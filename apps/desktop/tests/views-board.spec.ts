import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('issue table keeps column width and sort, with keyboard row navigation', async ({ page }) => {
  await page.goto('/#/issues');

  const resize = page.getByRole('separator', { name: 'Resize ID column' });
  const before = await page
    .locator('[data-issue-id="APP-123"]')
    .evaluate((row) => getComputedStyle(row).gridTemplateColumns);
  await resize.focus();
  await resize.press('ArrowRight');
  const after = await page
    .locator('[data-issue-id="APP-123"]')
    .evaluate((row) => getComputedStyle(row).gridTemplateColumns);
  expect(after).not.toBe(before);

  await page.getByRole('button', { name: 'Sort by Title' }).click();
  await expect(page.getByRole('columnheader', { name: /Title/ })).toHaveAttribute(
    'aria-sort',
    'ascending',
  );
  await page.reload();
  await expect(page.getByRole('columnheader', { name: /Title/ })).toHaveAttribute(
    'aria-sort',
    'ascending',
  );
  await expect(page.locator('[data-issue-id="APP-123"]')).toHaveCSS('grid-template-columns', after);

  const rows = page.locator('[data-issue-id]');
  await rows.first().focus();
  await rows.first().press('ArrowDown');
  await expect(rows.nth(1)).toBeFocused();
});

test('issue rows and board cards use the arrow cursor when idle', async ({ page }) => {
  await page.goto('/#/issues');
  const row = page.locator('[data-issue-id="APP-123"]');
  await row.hover();
  await expect(row).toHaveCSS('cursor', 'default');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  const card = page.locator('[data-issue-id="APP-123"]');
  await card.hover();
  await expect(card).toHaveCSS('cursor', 'default');
  const group = page.getByRole('button', { name: 'Move Todo group' });
  await group.hover();
  await expect(group).toHaveCSS('cursor', 'default');
});

test('board cards and view rows use the standard issue text size', async ({ page }) => {
  await page.goto('/#/issues');
  const rowSize = await page
    .locator('[data-issue-id="APP-123"]')
    .evaluate((element) => getComputedStyle(element).fontSize);
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  const boardSize = await page
    .locator('[data-issue-id="APP-123"]')
    .locator('span')
    .filter({ hasText: 'Persist issues to local SQLite' })
    .evaluate((element) => getComputedStyle(element).fontSize);
  expect(boardSize).toBe(rowSize);

  await page.goto('/#/views');
  const viewSize = await page
    .getByRole('link', { name: /Board view/ })
    .evaluate((element) => getComputedStyle(element).fontSize);
  expect(viewSize).toBe(rowSize);
});

test('saved view keeps its layout and opens issue detail', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Review queue');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Review queue' })).toBeVisible();
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Todo lane' })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('region', { name: 'Todo lane' })).toBeVisible();
  await page.getByRole('button', { name: 'Persist issues to local SQLite' }).click();
  await expect(page.getByRole('complementary', { name: 'Issue details' })).toBeVisible();
  await page.getByRole('button', { name: 'Close issue details' }).click();
  await expect(page.getByRole('complementary', { name: 'Issue details' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Edit view' }).click();
  const nameField = page
    .getByRole('dialog', { name: 'Edit view' })
    .getByRole('textbox', { name: 'View name' });
  await nameField.fill('Release review');
  await nameField.press('Enter');
  await expect(page.getByRole('heading', { name: 'Release review' })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Release review' })).toBeVisible();
});

test('saved view keeps nested filters and display settings', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Selected titles');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Edit view' }).click();

  const dialog = page.getByRole('dialog', { name: 'Edit view' });
  await dialog.getByRole('button', { name: 'Add group' }).click();
  await dialog.getByRole('button', { name: 'Add condition' }).first().click();
  await dialog.getByRole('button', { name: 'Add condition' }).first().click();
  await dialog.getByRole('textbox', { name: 'Filter value' }).nth(0).fill('Persist');
  await dialog.getByRole('textbox', { name: 'Filter value' }).nth(1).fill('Capture');
  await dialog.getByRole('combobox', { name: 'Row density' }).click();
  await page.getByRole('option', { name: 'compact' }).click();
  await dialog.getByRole('button', { name: 'Save', exact: true }).click();

  await expect(page.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="DSN-42"]')).toHaveCount(0);
  await page.reload();
  await expect(page.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
});

test('board drag updates status and list data', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  const card = page.locator('[data-issue-id="APP-123"]');
  const from = await card.boundingBox();
  const to = await page.getByRole('region', { name: 'In Progress lane' }).boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();
  await page.mouse.move(from!.x + from!.width / 2, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(180);
  await page.mouse.move(to!.x + to!.width / 2, to!.y + 100, { steps: 12 });
  await page.mouse.up();
  const settling = await card.evaluate(async (element) => {
    const positions: number[] = [];
    for (let frame = 0; frame < 18; frame++) {
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
      positions.push(element.getBoundingClientRect().y);
    }

    return positions;
  });
  expect(Math.max(...settling) - Math.min(...settling)).toBeLessThan(24);
  await expect(
    page.getByRole('region', { name: 'In Progress lane' }).locator('[data-issue-id="APP-123"]'),
  ).toBeVisible();
  await page.locator('[data-issue-id="APP-123"]').focus();
  await page.keyboard.press('Alt+ArrowRight');
  await expect(
    page.getByRole('region', { name: 'On Hold lane' }).locator('[data-issue-id="APP-123"]'),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole('region', { name: 'On Hold lane' }).locator('[data-issue-id="APP-123"]'),
  ).toBeVisible();
});

test('moving a board card to Done keeps it visible', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  const done = page.getByRole('region', { name: 'Done lane' });
  await done.getByRole('button', { name: 'Move Done group' }).focus();
  await page.keyboard.press('Alt+ArrowLeft');
  await page.keyboard.press('Alt+ArrowLeft');
  const card = page.getByRole('region', { name: 'Todo lane' }).locator('[data-issue-id="APP-123"]');
  await card.scrollIntoViewIfNeeded();
  const from = await card.boundingBox();
  const to = await done.locator('[data-board-lane]').boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();
  await page.mouse.move(from!.x + from!.width / 2, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(180);
  await page.mouse.move(to!.x + to!.width / 2, to!.y + 60, { steps: 12 });
  await page.mouse.up();
  await expect(done.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await expect(page.getByRole('button', { name: 'All', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
  await page.reload();
  await expect(done.locator('[data-issue-id="APP-123"]')).toBeVisible();
});

test('board cards make room before a drag is released', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  const todo = page.getByRole('region', { name: 'Todo lane' });
  const dragged = todo.locator('[data-issue-id="APP-123"]');
  const neighbor = todo.locator('[data-issue-id="APP-121"]');
  const from = await dragged.boundingBox();
  const to = await neighbor.boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();

  await page.mouse.move(from!.x + from!.width / 2, from!.y + from!.height / 2);
  await expect(dragged).toHaveCSS('cursor', 'default');
  await page.mouse.down();
  await page.waitForTimeout(180);
  await expect(dragged).toHaveCSS('cursor', 'default');
  await page.mouse.move(to!.x + to!.width / 2, to!.y + to!.height * 0.8, { steps: 16 });
  await expect(page.locator('body')).toHaveCSS('cursor', 'grabbing');
  await expect
    .poll(() =>
      todo
        .locator('[data-issue-id]')
        .evaluateAll((cards) => cards.map((card) => card.getAttribute('data-issue-id'))),
    )
    .toEqual(['APP-121', 'APP-123', 'DSN-39']);
  await expect.poll(async () => (await neighbor.boundingBox())?.y).toBeLessThan(to!.y - 20);
  await page.mouse.up();
  const settling = await dragged.evaluate(async (card) => {
    const positions: number[] = [];
    for (let frame = 0; frame < 18; frame++) {
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
      positions.push(card.getBoundingClientRect().y);
    }

    return positions;
  });
  expect(Math.max(...settling) - Math.min(...settling)).toBeLessThan(24);
  await expect(page.locator('body')).not.toHaveCSS('cursor', 'grabbing');
  await expect(todo.locator('[data-issue-id]').first()).toHaveAttribute('data-issue-id', 'APP-121');
  await page.reload();
  await expect(todo.locator('[data-issue-id]').first()).toHaveAttribute('data-issue-id', 'APP-121');
});

test('custom board group accepts issues and persists its place', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Add group' }).click();
  await page.getByRole('textbox', { name: 'Group name' }).fill('Review');
  await page.getByRole('button', { name: 'Add group', exact: true }).click();
  const review = page.getByRole('region', { name: 'Review lane' });
  await expect(review).toBeVisible();
  await expect(review.getByRole('button', { name: 'New issue in Review' })).toBeVisible();
  await review.getByRole('button', { name: 'New issue in Review' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  await expect(dialog.getByRole('combobox', { name: 'Status' })).toContainText('Review');
  await dialog.getByRole('textbox', { name: 'Title' }).fill('Check release notes');
  await dialog.getByRole('button', { name: 'Create issue' }).click();
  await expect(review.getByRole('button', { name: 'Check release notes' })).toBeVisible();

  const handle = review.getByRole('button', { name: 'Move Review group' });
  await handle.focus();
  for (let step = 0; step < 5; step++) await page.keyboard.press('Alt+ArrowLeft');
  await expect(
    page.locator('[aria-label="Issue board"] [data-board-group]').nth(1),
  ).toHaveAttribute('data-board-group', 'Review');
  const todoCard = page
    .getByRole('region', { name: 'Todo lane' })
    .locator('[data-issue-id="APP-123"]');
  await todoCard.scrollIntoViewIfNeeded();
  const from = await todoCard.boundingBox();
  const to = await review.locator('[data-board-lane]').boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();
  await page.mouse.move(from!.x + from!.width / 2, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(180);
  await page.mouse.move(to!.x + to!.width / 2, to!.y + 100, { steps: 14 });
  await page.mouse.up();
  await expect(review.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await page.reload();
  await expect(review.getByRole('button', { name: 'Check release notes' })).toBeVisible();
  await expect(review.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'List', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Review issues' })).toBeVisible();
});

test('custom status group can be styled, archived, restored, and deleted when empty', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('region', { name: 'On Hold lane' })).toBeVisible();

  await page.getByRole('button', { name: 'Add group' }).click();
  await page.getByRole('textbox', { name: 'Group name' }).fill('Review');
  await page.getByRole('textbox', { name: 'Search group icons' }).fill('scheduled');
  await page.getByRole('button', { name: 'Scheduled' }).click();
  await page.getByRole('button', { name: 'purple color' }).click();
  await page.getByRole('button', { name: 'Add group', exact: true }).click();
  await page.getByRole('button', { name: 'Edit Review group' }).click();
  await page.getByRole('textbox', { name: 'Edit group name' }).fill('In Review');
  await page.getByRole('button', { name: 'Save group' }).click();
  const review = page.getByRole('region', { name: 'In Review lane' });
  await expect(review).toBeVisible();
  expect(await review.locator('[data-board-group-handle] svg').first().innerHTML()).not.toBe(
    await page
      .getByRole('region', { name: 'Todo lane' })
      .locator('[data-board-group-handle] svg')
      .first()
      .innerHTML(),
  );
  await review.getByRole('button', { name: 'Edit In Review group' }).click();
  await page.getByRole('button', { name: 'Archive group' }).click();
  await expect(review).toHaveCount(0);
  await expect(page.getByText('Archived groups (1)')).toBeVisible();
  await page.getByText('Archived groups (1)').click();
  await page.getByRole('button', { name: 'Restore In Review' }).click();
  await expect(review).toBeVisible();
  await review.getByRole('button', { name: 'Edit In Review group' }).click();
  await page.getByRole('button', { name: 'Delete empty group' }).click();
  await expect(review).toHaveCount(0);
  await page.reload();
  await expect(page.getByRole('region', { name: 'In Review lane' })).toHaveCount(0);
});

test('archiving a populated group keeps its issues and prevents new assignments', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Add group' }).click();
  await page.getByRole('textbox', { name: 'Group name' }).fill('Waiting');
  await page.getByRole('button', { name: 'Add group', exact: true }).click();
  const waiting = page.getByRole('region', { name: 'Waiting lane' });
  await waiting.getByRole('button', { name: 'New issue in Waiting' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  await dialog.getByRole('textbox', { name: 'Title' }).fill('Awaiting reply');
  await dialog.getByRole('button', { name: 'Create issue' }).click();
  await waiting.getByRole('button', { name: 'Edit Waiting group' }).click();
  await page.getByRole('textbox', { name: 'Edit group name' }).fill('Awaiting');
  await page.getByRole('button', { name: 'Save group' }).click();
  const awaiting = page.getByRole('region', { name: 'Awaiting lane' });
  await expect(awaiting.getByRole('button', { name: 'Awaiting reply' })).toBeVisible();
  await awaiting.getByRole('button', { name: 'Edit Awaiting group' }).click();
  await page.getByRole('button', { name: 'Archive group' }).click();
  await expect(awaiting.getByRole('button', { name: 'Awaiting reply' })).toBeVisible();
  await expect(awaiting.getByRole('button', { name: 'New issue in Awaiting' })).toHaveCount(0);
  await page.reload();
  await expect(awaiting.getByRole('button', { name: 'Awaiting reply' })).toBeVisible();
  await awaiting.getByRole('button', { name: 'Edit Awaiting group' }).click();
  await expect(page.getByRole('button', { name: 'Delete empty group' })).toBeDisabled();
  await page.getByRole('button', { name: 'Restore group' }).click();
  await expect(awaiting.getByRole('button', { name: 'New issue in Awaiting' })).toBeVisible();
});

test('board groups move by dragging their headers', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  const board = page.locator('[aria-label="Issue board"]');
  const from = await board.getByRole('button', { name: 'Move Todo group' }).boundingBox();
  const to = await board.getByRole('button', { name: 'Move In Progress group' }).boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();
  await page.mouse.move(from!.x + from!.width / 2, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(180);
  await page.mouse.move(to!.x + to!.width * 0.8, to!.y + to!.height / 2, { steps: 18 });
  await expect(board.locator('[data-board-group]').first()).toHaveAttribute(
    'data-board-group',
    'In Progress',
  );
  await page.mouse.up();
  await page.reload();
  await expect(board.locator('[data-board-group]').first()).toHaveAttribute(
    'data-board-group',
    'In Progress',
  );
});

test('list row drags between groups and groups collapse smoothly', async ({ page }) => {
  await page.goto('/#/issues');
  const todo = page.getByRole('region', { name: 'Todo issues' });
  const progress = page.getByRole('region', { name: 'In Progress issues' });
  const row = todo.locator('[data-sort-id="APP-123"]');
  const from = await row.boundingBox();
  const to = await progress.locator('[data-sort-id="APP-124"]').boundingBox();
  expect(from).toBeTruthy();
  expect(to).toBeTruthy();
  await page.mouse.move(from!.x + 40, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(180);
  await page.mouse.move(to!.x + 40, to!.y + to!.height / 2, { steps: 12 });
  await page.waitForTimeout(100);
  await page.mouse.up();
  await expect(progress.locator('[data-sort-id="APP-123"]')).toBeVisible();
  await page.reload();
  await expect(progress.locator('[data-sort-id="APP-123"]')).toBeVisible();

  const content = progress.locator('.issue-group-body');
  const before = await content.boundingBox();
  const groupToggle = progress.getByRole('button', { name: /^In Progress \d+$/ });
  await groupToggle.click();
  await expect(groupToggle).toHaveAttribute('aria-expanded', 'false');
  await expect.poll(async () => (await content.boundingBox())?.height).toBeLessThan(before!.height);
  await expect(content).toHaveCSS('grid-template-rows', '0px');
  await groupToggle.click();
  await expect(progress.locator('[data-sort-id="APP-123"]')).toBeVisible();
  await progress.locator('[data-issue-id="APP-123"]').focus();
  await page.keyboard.press('Alt+ArrowLeft');
  await expect(todo.locator('[data-sort-id="APP-123"]')).toBeVisible();
});
