import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('workspaces switch without mixing projects or issues', async ({ page }) => {
  test.setTimeout(60_000);
  await page.addInitScript(() => {
    document.addEventListener('DOMContentLoaded', () => {
      if (document.documentElement.classList.contains('workspace-switch-pending'))
        sessionStorage.setItem('workspace-switch-cover-observed', 'yes');
    });
  });
  await page.goto('/#/issues');
  await expect(page.getByRole('button', { name: 'Switch workspace' })).toContainText(
    'Personal workspace',
  );
  const createIssue = page.getByRole('button', { name: 'Create issue' }).first();
  await expect(createIssue).toBeVisible();
  await expect(createIssue).toHaveClass(/ipex-execution/);
  await expect(createIssue).toHaveCSS('font-size', '12px');
  await expect(createIssue).toHaveCSS('min-height', '24px');

  const trigger = page.getByRole('button', { name: 'Switch workspace' });
  const workspaceName = trigger.getByText('Personal workspace');
  const chevron = trigger.locator('svg');
  const nameBox = await workspaceName.boundingBox();
  const chevronBox = await chevron.boundingBox();
  expect(nameBox).toBeTruthy();
  expect(chevronBox).toBeTruthy();
  expect(chevronBox!.x - (nameBox!.x + nameBox!.width)).toBeLessThan(14);
  const triggerBox = await trigger.boundingBox();
  expect(triggerBox).toBeTruthy();
  expect(triggerBox!.x + triggerBox!.width - (chevronBox!.x + chevronBox!.width)).toBeLessThan(12);
  await trigger.click();
  const workspaceOption = page.getByRole('menuitem', { name: 'Personal workspace' });
  await expect(workspaceOption).toHaveCSS('font-size', '12px');
  await expect(workspaceOption).toHaveCSS('min-height', '28px');
  const menuBox = await page.locator('.akaza-menu-content').first().boundingBox();
  const sidebarBox = await page
    .getByRole('complementary', { name: 'Workspace navigation' })
    .boundingBox();
  expect(menuBox).toBeTruthy();
  expect(sidebarBox).toBeTruthy();
  expect(menuBox!.width).toBeGreaterThan(sidebarBox!.width - 24);
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: 'Add workspace' }).click();
  await page.getByRole('textbox', { name: 'Workspace name' }).fill('Design team');
  const switchStartedAt = Date.now();
  await page.getByRole('button', { name: 'Create workspace' }).click();
  await expect(page.locator('#workspace-transition')).toContainText(
    /Creating workspace\s+Design team/,
  );
  await expect(page.locator('#workspace-transition')).toBeHidden({ timeout: 10_000 });
  expect(Date.now() - switchStartedAt).toBeGreaterThanOrEqual(1900);
  await expect(page.getByRole('button', { name: 'Switch workspace' })).toContainText('Design team');
  expect(await page.evaluate(() => sessionStorage.getItem('workspace-switch-cover-observed'))).toBe(
    'yes',
  );
  await expect(page.getByRole('link', { name: 'Desktop app' })).toHaveCount(0);
  await expect(page.getByRole('link', { name: /All issues/ })).toContainText('0');

  await page.getByRole('button', { name: 'Create project' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Design library');
  await dialog.getByRole('button', { name: 'Create project' }).click();
  await expect(dialog).toBeHidden();
  await expect(page.locator('.akaza-toast').filter({ hasText: 'Project created' })).toBeVisible();
  await expect(page.getByRole('link', { name: /Design library/ })).toBeVisible();

  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('Personal workspace');
  await page.getByRole('option', { name: 'Switch to Personal workspace' }).click();
  await expect(page.locator('#workspace-transition')).toContainText('Personal workspace');
  await expect(page.locator('#workspace-transition')).toBeHidden({ timeout: 10_000 });
  await expect(
    page.locator('.akaza-toast').filter({ hasText: 'Workspace switched' }),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Switch workspace' })).toContainText(
    'Personal workspace',
  );
  await expect(page.getByRole('link', { name: /Desktop app/ })).toBeVisible();
  await expect(page.getByRole('link', { name: /Design library/ })).toHaveCount(0);

  await page.getByRole('button', { name: 'Switch workspace' }).click();
  await page.getByRole('menuitem', { name: 'Design team' }).click();
  await expect(page.locator('#workspace-transition')).toBeHidden();
  await expect(page.getByRole('link', { name: /Design library/ })).toBeVisible();

  await page.getByRole('button', { name: 'Create issue' }).first().click();
  const issueDialog = page.getByRole('dialog', { name: 'Create issue' });
  await issueDialog.getByRole('textbox', { name: 'Title' }).fill('Check workspace notifications');
  await issueDialog.getByRole('button', { name: 'Create issue' }).click();
  await expect(page.locator('.akaza-toast').filter({ hasText: 'Issue created' })).toBeVisible();
});
