import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('Help opens the About dialog outside Settings', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Help and settings' }).click();
  await page.getByRole('menuitem', { name: 'Help' }).click();
  const submenu = page.locator('.akaza-menu-submenu-content');
  const parent = page.locator('.akaza-menu-content').first();
  const [submenuBounds, parentBounds] = await Promise.all([
    submenu.boundingBox(),
    parent.boundingBox(),
  ]);
  expect(submenuBounds!.x - (parentBounds!.x + parentBounds!.width)).toBeGreaterThanOrEqual(7);
  await expect(page.getByRole('menuitem', { name: 'Submit Issue & Feedback' })).toHaveCSS(
    'white-space',
    'nowrap',
  );
  await page.getByRole('menuitem', { name: 'About' }).click();

  const about = page.getByRole('dialog', { name: 'About Initera' });
  await expect(about).toBeVisible();
  await expect(about).toContainText('Rahul Vashishtha');
  await expect(about).toContainText('0.1.0');

  await about.getByRole('button', { name: 'Close', exact: true }).click();
  await page.goto('/#/settings');
  await expect(page.getByRole('heading', { name: 'About Initera' })).toHaveCount(0);
});

test('help menu opens settings, changes appearance, and launches commands', async ({ page }) => {
  await page.goto('/#/issues');
  const account = page.getByRole('button', { name: 'Help and settings' });
  await account.click();
  await expect(account).toHaveAttribute('aria-expanded', 'true');
  const triggerBounds = await account.boundingBox();
  const menu = page.locator('.akaza-menu-content').first();
  await expect
    .poll(async () => Math.abs((await menu.boundingBox())!.width - triggerBounds!.width))
    .toBeLessThan(0.5);
  const menuBounds = await menu.boundingBox();
  expect(menuBounds!.x).toBeCloseTo(triggerBounds!.x, 0);
  const settings = page.getByRole('menuitem', { name: 'Settings' });
  await expect(settings).toHaveCSS('font-size', '12px');
  const restingBackground = await settings.evaluate(
    (element) => getComputedStyle(element).backgroundColor,
  );
  await settings.hover();
  await expect
    .poll(() => settings.evaluate((element) => getComputedStyle(element).backgroundColor))
    .not.toBe(restingBackground);
  await settings.click();
  await expect(page).toHaveURL(/#\/settings$/);
  await expect(page.getByRole('heading', { name: 'Settings' }).last()).toBeVisible();
  await page
    .getByRole('radiogroup', { name: 'Appearance' })
    .getByText('Light', { exact: true })
    .click();
  await expect(page.getByRole('radio', { name: /Light/ })).toBeChecked();
  await expect(page.locator('html')).not.toHaveClass(/dark/);

  await page.getByRole('button', { name: 'Help and settings' }).click();
  await page.getByRole('menuitem', { name: 'Theme' }).click();
  const themeMenu = page.locator('.akaza-menu-submenu-content');
  await expect(themeMenu).toBeVisible();
  await expect(themeMenu).not.toHaveCSS('background-color', 'rgba(0, 0, 0, 0)');
  const darkTheme = page.getByRole('menuitem', { name: 'Dark' });
  const darkBounds = await darkTheme.boundingBox();
  expect(darkBounds).toBeTruthy();
  expect(darkBounds!.y + darkBounds!.height).toBeLessThanOrEqual(page.viewportSize()!.height);
  await darkTheme.click();
  await expect(page.locator('html')).toHaveClass(/dark/);
  await page.getByRole('button', { name: 'Help and settings' }).click();
  await page.getByRole('menuitem', { name: 'Command bar' }).click();
  const search = page.getByRole('combobox', { name: 'Search commands' });
  await expect(search).toBeFocused();
  await search.fill('Triage');
  await search.press('Enter');
  await expect(page).toHaveURL(/#\/triage$/);
  await page.keyboard.press('ControlOrMeta+k');
  await expect(search).toBeFocused();
});

test('theme submenu stays above the board', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await page.keyboard.press('Escape');
  await expect(page.getByLabel('Issue board')).toBeVisible();

  await page.getByRole('button', { name: 'Help and settings' }).click();
  await page.getByRole('menuitem', { name: 'Theme' }).click();
  const dark = page.getByRole('menuitem', { name: 'Dark' });
  await expect(dark).toBeVisible();
  const bounds = await dark.boundingBox();
  expect(bounds).toBeTruthy();
  const hit = await page.evaluate(
    ({ x, y }) => document.elementFromPoint(x, y)?.closest('[role="menuitem"]')?.textContent,
    { x: bounds!.x + bounds!.width / 2, y: bounds!.y + bounds!.height / 2 },
  );
  expect(hit).toContain('Dark');
});

test('select trigger and options respond to hover in both themes', async ({ page }) => {
  await page.goto('/#/issues');
  for (const theme of ['light', 'dark']) {
    await page.evaluate(
      (value) => document.documentElement.classList.toggle('dark', value === 'dark'),
      theme,
    );
    await page.getByRole('button', { name: 'Create issue' }).first().click();
    const status = page.getByRole('combobox', { name: 'Status' });
    const restingTrigger = await status.evaluate(
      (element) => getComputedStyle(element).backgroundColor,
    );
    await status.hover();
    await expect
      .poll(() => status.evaluate((element) => getComputedStyle(element).backgroundColor))
      .not.toBe(restingTrigger);
    await status.click();
    const todo = page.getByRole('option', { name: /Todo/ });
    const restingOption = await todo.evaluate(
      (element) => getComputedStyle(element).backgroundColor,
    );
    await todo.hover();
    await expect
      .poll(() => todo.evaluate((element) => getComputedStyle(element).backgroundColor))
      .not.toBe(restingOption);
    await page.keyboard.press('Escape');
    await page.keyboard.press('Escape');
  }
});
