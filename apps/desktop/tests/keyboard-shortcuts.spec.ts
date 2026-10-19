import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('shortcuts work immediately after closing the command bar', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('ControlOrMeta+k');
  await expect(page.getByRole('dialog', { name: 'Command bar' })).toBeVisible();
  await page.keyboard.press('ControlOrMeta+k');
  await expect(page.getByRole('dialog', { name: 'Command bar' })).toBeHidden();

  await page.keyboard.press('g');
  await page.keyboard.press('w');
  await expect(page).toHaveURL(/#\/settings#workflow$/);

  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('Create issue');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog', { name: 'Create issue' })).toBeVisible();
});

test('Escape from the command bar leaves the next shortcut ready', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('ControlOrMeta+k');
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog', { name: 'Command bar' })).toBeHidden();
  await page.keyboard.press('c');
  await expect(page.getByRole('dialog', { name: 'Create issue' })).toBeVisible();
});

test('command bar searches issues and projects with keyboard navigation', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('/');
  const search = page.getByRole('combobox', { name: 'Search commands' });
  await expect(search).toBeFocused();
  await search.fill('Desktop app');
  await expect(page.getByRole('option', { name: /Desktop app/ })).toBeVisible();
  await search.press('Enter');
  await expect(page).toHaveURL(/#\/projects\/Desktop%20app$/);
  await expect(page.getByRole('dialog', { name: 'Command bar' })).toBeHidden();
  await page.keyboard.press('o');
  await page.keyboard.press('i');
  await expect(search).toBeFocused();
  await expect(search).toHaveAttribute('placeholder', 'Search issues...');
});

test('command bar finds issues by ID and projects by name from another screen', async ({
  page,
}) => {
  await page.goto('/#/settings');
  await page.keyboard.press('ControlOrMeta+k');
  const search = page.getByRole('combobox', { name: 'Search commands' });
  await search.fill('APP-123');
  await expect(
    page.getByRole('option', { name: /APP-123.*Persist issues to local SQLite/ }),
  ).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#\/issues\/APP-123\/full$/);

  await page.keyboard.press('ControlOrMeta+k');
  await search.fill('Desktop app');
  await expect(page.getByRole('option', { name: /Desktop app/ })).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#\/projects\/Desktop%20app$/);
});

test('command bar opens a matched issue comment', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  await page.getByRole('textbox', { name: 'Write a comment' }).fill('Review the cobalt handoff');
  await page.getByRole('button', { name: 'Post comment' }).click();
  await page.goto('/#/settings');

  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('cobalt');
  await page.getByRole('option', { name: /APP-123.*Review the cobalt handoff/ }).click();

  await expect(page).toHaveURL(/#\/issues\/APP-123\/full\?comment=/);
  await expect(page.getByLabel('Comment content').getByText('cobalt handoff')).toBeVisible();
});

test('command bar exposes creation and appearance actions across screens', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('ControlOrMeta+k');
  const search = page.getByRole('combobox', { name: 'Search commands' });
  await search.fill('create project');
  await page.getByRole('option', { name: 'Create project' }).click();
  await expect(page.getByRole('dialog', { name: 'Create project' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog', { name: 'Create project' })).toBeHidden();

  await page.keyboard.press('ControlOrMeta+k');
  await search.fill('create view');
  await page.getByRole('option', { name: 'Create view' }).click();
  await expect(page.getByRole('textbox', { name: 'View name' })).toBeVisible();

  await page.keyboard.press('ControlOrMeta+k');
  await search.fill('dark theme');
  await page.getByRole('option', { name: 'Use dark theme' }).click();
  await expect(page.locator('html')).toHaveClass(/dark/);
});

test('command bar can start workspace creation and search saved views', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('ControlOrMeta+k');
  const search = page.getByRole('combobox', { name: 'Search commands' });
  await search.fill('create workspace');
  await page.getByRole('option', { name: 'Create workspace' }).click();
  await expect(page.getByRole('dialog', { name: 'Add workspace' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog', { name: 'Add workspace' })).toBeHidden();

  await page.keyboard.press('ControlOrMeta+k');
  await search.fill('create view');
  await page.getByRole('option', { name: 'Create view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Release watch');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.keyboard.press('ControlOrMeta+k');
  await search.fill('Release watch');
  await expect(page.getByRole('option', { name: 'Release watch' })).toBeVisible();
});

test('command bar can start a sub-issue from issue detail', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('sub-issue');
  await page.getByRole('option', { name: 'Add sub-issue' }).click();
  await expect(page.getByRole('dialog', { name: 'Create sub-issue' })).toBeVisible();
});

test('command bar opens the issue detail property picker', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('Set parent issue');
  await page.getByRole('option', { name: 'Set parent issue' }).click();

  await expect(page.getByRole('combobox', { name: 'Parent issue' })).toHaveAttribute(
    'aria-expanded',
    'true',
  );
});

test('command bar changes issue scope and display preferences', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('ControlOrMeta+k');
  const search = page.getByRole('combobox', { name: 'Search commands' });
  await search.fill('including completed');
  await page.getByRole('option', { name: 'Show all issues including completed' }).click();
  await expect(page.getByRole('button', { name: 'All', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true',
  );

  await page.keyboard.press('ControlOrMeta+k');
  await search.fill('hide issue priority');
  await page.getByRole('option', { name: 'Hide issue priority' }).click();
  await page.goto('/#/settings');
  await page.getByRole('tab', { name: 'Issues' }).click();
  await expect(page.getByRole('switch', { name: 'Show priority' })).toHaveAttribute(
    'aria-checked',
    'false',
  );
});

test('new and open sequences reach project, view, and workspace actions', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('n');
  await page.keyboard.press('p');
  const projectDialog = page.getByRole('dialog', { name: 'Create project' });
  await expect(projectDialog).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(projectDialog).toBeHidden();

  await page.keyboard.press('n');
  await page.keyboard.press('w');
  const workspaceDialog = page.getByRole('dialog', { name: 'Add workspace' });
  await expect(workspaceDialog).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(workspaceDialog).toBeHidden();

  await page.keyboard.press('n');
  await page.keyboard.press('v');
  await expect(page.getByRole('textbox', { name: 'View name' })).toBeVisible();
  await page.getByRole('button', { name: 'Cancel' }).click();
  await page.keyboard.press('o');
  await page.keyboard.press('w');
  await expect(page.getByRole('combobox', { name: 'Search commands' })).toHaveAttribute(
    'placeholder',
    'Search workspaces...',
  );
});

test('navigation and detail shortcuts reach contextual actions', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('g');
  await page.keyboard.press('t');
  await expect(page).toHaveURL(/#\/triage$/);
  await page.keyboard.press('g');
  await page.keyboard.press('s');
  await expect(page).toHaveURL(/#\/settings$/);

  await page.goto('/#/issues/APP-123/full');
  await expect(page.getByRole('button', { name: 'Add sub-issue' })).toBeVisible();
  await page.locator('body').click({ position: { x: 2, y: 2 } });
  await page.keyboard.press('Shift+c');
  await expect(page.getByRole('dialog', { name: 'Create sub-issue' })).toBeVisible();
  await page.getByRole('button', { name: 'Close dialog' }).click();
  await expect(page.getByRole('dialog', { name: 'Create sub-issue' })).toBeHidden();
  await page.locator('body').click({ position: { x: 2, y: 2 } });
  await page.keyboard.press('e');
  await expect(page.getByRole('button', { name: 'Save changes' })).toBeVisible();

  await page.goto('/#/projects/Desktop%20app');
  await expect(page.getByRole('combobox', { name: 'Project status' })).toBeVisible();
  await page.keyboard.press('p');
  await page.keyboard.press('u');
  await expect(page.getByRole('textbox', { name: 'Project update' })).toBeVisible();
});

test('issue view shortcuts control filters, display and board layout', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('f');
  await expect(page.getByRole('textbox', { name: 'Search issues' })).toBeFocused();
  await page.keyboard.press('Escape');
  await page.keyboard.press('Shift+v');
  await expect(page.getByRole('button', { name: 'Board', exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  await page.keyboard.press('ControlOrMeta+b');
  await expect(page.getByLabel('Issue board')).toBeVisible();
  await page.keyboard.press('v');
  await page.keyboard.press('l');
  await expect(page.getByLabel('Issue board')).toBeHidden();
  await page.keyboard.press('?');
  await expect(page.getByText('Keyboard shortcuts', { exact: true }).first()).toBeVisible();
});

test('project shortcut opens the matching property picker', async ({ page }) => {
  await page.goto('/#/projects/Desktop%20app');
  await expect(page.getByRole('combobox', { name: 'Project status' })).toBeVisible();
  await page.keyboard.press('p');
  await page.keyboard.press('s');
  await expect(page.getByRole('combobox', { name: 'Project status' })).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await page.keyboard.press('Escape');
  await page.keyboard.press('p');
  await page.keyboard.press('o');
  await expect(page.getByRole('combobox', { name: 'Project lead' })).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await page.keyboard.press('Escape');
  await page.keyboard.press('p');
  await page.keyboard.press('v');
  await expect(page).toHaveURL(/#\/projects\/Desktop%20app\/issues$/);
});

test('command bar suggestions follow the current screen', async ({ page }) => {
  await page.goto('/#/projects/Desktop%20app');
  await expect(page.getByRole('combobox', { name: 'Project status' })).toBeVisible();
  await page.keyboard.press('ControlOrMeta+k');
  const commands = page.getByRole('listbox', { name: 'Commands' });
  await expect(commands.getByRole('option', { name: 'Change project status' })).toBeVisible();
  await expect(commands.getByRole('option', { name: 'Set project lead' })).toBeVisible();
  await expect(commands.getByRole('option', { name: 'Set project target date' })).toBeVisible();
  await expect(commands.getByRole('option', { name: 'Filter issues' })).toHaveCount(0);
  await commands.getByRole('option', { name: 'Write project update' }).click();
  await expect(page.getByRole('textbox', { name: 'Project update' })).toBeVisible();
});

test('issue shortcuts open status and priority pickers', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  await expect(page.getByRole('combobox', { name: 'Status' })).toBeVisible();
  await page.keyboard.press('s');
  await expect(page.getByRole('combobox', { name: 'Status' })).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await page.keyboard.press('Escape');
  await page.keyboard.press('p');
  await expect(page.getByRole('combobox', { name: 'Priority' })).toHaveAttribute(
    'aria-expanded',
    'true',
  );
});
