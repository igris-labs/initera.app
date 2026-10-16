import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('context menu stays scoped to an issue', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('main', { name: 'All issues' }).click({
    button: 'right',
    position: { x: 700, y: 650 },
  });
  await expect(page.locator('.akaza-context-menu-content')).toHaveCount(0);
});

test('list issue context menu updates status and labels', async ({ page }) => {
  await page.goto('/#/issues');
  const row = page.getByRole('button', { name: /Open APP-123:/ });
  await row.click({ button: 'right' });
  const menu = page.locator('.akaza-context-menu-content');
  await expect(menu).toBeVisible();
  expect(await menu.evaluate((element) => getComputedStyle(element).padding)).toBe('4px');
  expect(await menu.evaluate((element) => getComputedStyle(element).width)).toBe('224px');
  expect(
    await menu
      .locator('.akaza-menu-item')
      .first()
      .evaluate((element) => getComputedStyle(element).paddingLeft),
  ).toBe('8px');
  expect(
    await menu
      .locator('.akaza-menu-item')
      .first()
      .evaluate((element) => getComputedStyle(element).fontSize),
  ).toBe('12px');
  await expect(menu.locator('.akaza-menu-item svg').first()).toBeVisible();
  await menu.getByRole('menuitem', { name: /Status · Todo/ }).hover();
  await menu.getByRole('menuitem', { name: 'On Hold', exact: true }).click();
  await expect(menu).toHaveCount(0);
  await expect(row).toHaveAttribute('aria-label', /On Hold/);

  await row.click({ button: 'right' });
  await menu.getByRole('menuitem', { name: /Labels · 1/ }).hover();
  await menu.getByRole('menuitemcheckbox', { name: 'Bug' }).click();
  await expect(menu).toBeVisible();
  await expect(menu.getByRole('menuitemcheckbox', { name: 'Bug' })).toHaveAttribute(
    'aria-checked',
    'true',
  );
  await page.keyboard.press('Escape');
  await page.reload();
  await row.click({ button: 'right' });
  await menu.getByRole('menuitem', { name: /Labels · 2/ }).hover();
  await expect(menu.getByRole('menuitemcheckbox', { name: 'Bug' })).toHaveAttribute(
    'aria-checked',
    'true',
  );
});

test('board issue context menu opens from keyboard and updates project', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  const card = page.locator('[data-issue-id="APP-123"]');
  await card.focus();
  await page.keyboard.press('Shift+F10');
  const menu = page.locator('.akaza-context-menu-content');
  await expect(menu).toBeVisible();
  await menu.getByRole('menuitem', { name: /Project · Desktop app/ }).hover();
  await menu.getByRole('menuitem', { name: 'Design system', exact: true }).press('Enter');
  const moveDialog = page.getByRole('dialog', { name: 'Move APP-123 to Design system' });
  await expect(moveDialog).toBeVisible();
  await moveDialog.getByRole('button', { name: 'Move issue' }).click();
  await card.click({ button: 'right' });
  await expect(menu.getByRole('menuitem', { name: /Project · Design system/ })).toBeVisible();
  await menu.getByRole('menuitem', { name: /Priority · Urgent/ }).hover();
  await menu.getByRole('menuitem', { name: 'High', exact: true }).press('Enter');
  await card.click({ button: 'right' });
  await menu.getByRole('menuitem', { name: /Due date/ }).hover();
  await menu.getByRole('menuitem', { name: 'Tomorrow', exact: true }).press('Enter');
  await card.click({ button: 'right' });
  await expect(menu.getByRole('menuitem', { name: /Priority · High/ })).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: /Due date · \d{4}-\d{2}-\d{2}/ })).toBeVisible();
  await page.keyboard.press('Escape');
  await page.reload();
  await card.click({ button: 'right' });
  await expect(menu.getByRole('menuitem', { name: /Project · Design system/ })).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: /Due date · \d{4}-\d{2}-\d{2}/ })).toBeVisible();
});

test('reviewed project move can regenerate an issue ID while preserving its work', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open APP-123:/ }).click({ button: 'right' });
  const menu = page.locator('.akaza-context-menu-content');
  await menu.getByRole('menuitem', { name: /Project · Desktop app/ }).hover();
  await menu.getByRole('menuitem', { name: 'Design system', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Move APP-123 to Design system' });
  await expect(dialog.getByText(/1 issue will move/)).toBeVisible();
  const regenerate = dialog.getByRole('radio', { name: /Use DSN-/ });
  const nextId = (await regenerate.textContent())?.match(/DSN-\d+/)?.[0];
  expect(nextId).toBeTruthy();
  await regenerate.click();
  await dialog.getByRole('button', { name: 'Move issue' }).click();
  await expect(page.locator(`[data-issue-id="${nextId}"]`)).toBeVisible();
  await expect(page.getByRole('button', { name: /Open APP-123:/ })).toHaveCount(0);
  await expect(dialog).toBeHidden();
  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('APP-123');
  await expect(page.getByRole('option', { name: new RegExp(nextId!) })).toBeVisible();
});
