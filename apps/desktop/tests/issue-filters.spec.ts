import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('advanced date and relation groups persist in saved views', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Upcoming work');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Advanced filter' }).hover();
  await page.getByRole('button', { name: 'Add group' }).click();
  await expect(page.getByRole('button', { name: 'Clear advanced filter group 1' })).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="DSN-42"]')).toHaveCount(0);

  await page.getByRole('combobox', { name: 'Condition 1 operator' }).click();
  await page.getByRole('option', { name: 'Between' }).click();
  await page.getByLabel('Condition 1 date').fill('2026-10-01');
  await page.getByLabel('Condition 1 end date').fill('2026-10-31');
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-123"]')).toHaveCount(0);

  await page.getByRole('button', { name: 'Add condition' }).click();
  await page.getByRole('combobox', { name: 'Condition 2 field' }).click();
  await page.getByRole('option', { name: 'Parent issue' }).click();
  await page.getByRole('combobox', { name: 'Condition 2 operator' }).click();
  await page.getByRole('option', { name: 'No value' }).click();
  await page.reload();
  await expect(page.getByRole('button', { name: 'Clear advanced filter group 1' })).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-123"]')).toHaveCount(0);
  await expect(page.locator('[data-issue-id="DSN-42"]')).toHaveCount(0);
});

test('filters combine across fields and can be cleared from chips', async ({ page }) => {
  await page.goto('/#/issues');
  await page.keyboard.press('f');
  await expect(page.getByRole('textbox', { name: 'Search issues' })).toBeFocused();

  await page.getByRole('button', { name: 'Status', exact: true }).hover();
  const statusMenu = page.getByRole('menu', { name: 'Status options' });
  await expect(statusMenu).toBeVisible();
  const categoryBounds = await page
    .getByRole('button', { name: 'Status', exact: true })
    .boundingBox();
  const submenuBounds = await statusMenu.boundingBox();
  expect(submenuBounds!.x + submenuBounds!.width).toBeLessThan(categoryBounds!.x);
  await expect(page.getByRole('button', { name: 'Priority', exact: true })).toBeVisible();
  await page.getByRole('menuitemcheckbox', { name: 'Todo', exact: true }).click();
  await page.getByRole('button', { name: 'Priority', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'High', exact: true }).click();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'Workflow', exact: true }).click();
  await page.getByRole('button', { name: 'Project', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'Desktop app', exact: true }).click();
  await page.getByRole('button', { name: 'Due date', exact: true }).click();
  await page.getByRole('menuitemradio', { name: 'No due date', exact: true }).click();

  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-123"]')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Clear Workflow label filter' })).toBeVisible();
  await page.getByRole('button', { name: 'Clear all' }).click();
  await expect(page.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await expect(page.getByLabel('Active filters')).toHaveCount(0);
});

test('saved views retain priority and label filters after restart', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('High priority design');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Priority', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'High', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'Medium', exact: true }).click();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'UI', exact: true }).click();
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-118"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Clear High priority filter' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Clear Medium priority filter' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Clear UI label filter' })).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-118"]')).toBeVisible();
});

test('clearing a saved view removes its filters after reload', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Reset filters');
  await page.getByRole('button', { name: 'Create', exact: true }).click();

  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Priority', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'High', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Clear High priority filter' })).toBeVisible();
  await page.getByRole('button', { name: 'Clear all' }).click();

  await page.reload();
  await expect(page.getByLabel('Active filters')).toHaveCount(0);
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
});

test('project property submenus persist their choices in saved views', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Planned projects');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Project properties' }).hover();
  await page.getByRole('button', { name: 'Project status' }).hover();

  const statusOptions = page.getByRole('menu', { name: 'Project status options' });
  await statusOptions.getByRole('menuitemcheckbox', { name: 'Planned' }).click();
  await expect(
    page.getByRole('button', { name: 'Clear Planned project status filter' }),
  ).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);

  await page.reload();
  await expect(
    page.getByRole('button', { name: 'Clear Planned project status filter' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Clear Planned project status filter' }).click();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
});

test('project label filters include, exclude, and retain saved selections', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const projectDialog = page.getByRole('dialog', { name: 'Create project' });
  await projectDialog.getByRole('textbox', { name: 'Project name' }).fill('Mobile app');
  await projectDialog.getByRole('textbox', { name: 'Identifier (2–8 letters)' }).fill('MOB');
  await projectDialog.getByRole('button', { name: 'Add labels' }).click();
  await page.getByRole('checkbox', { name: 'Workflow' }).click();
  await projectDialog.getByRole('button', { name: 'Create project' }).click();
  await page.getByRole('link', { name: 'Issues', exact: true }).click();
  await page.getByRole('button', { name: 'Create issue' }).click();
  const issueDialog = page.getByRole('dialog', { name: 'Create issue' });
  await issueDialog.getByRole('textbox', { name: 'Title' }).fill('Ship mobile navigation');
  await issueDialog.getByRole('button', { name: 'Create issue' }).click();

  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Workflow projects');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Project properties' }).hover();
  await page.getByRole('button', { name: 'Project labels' }).hover();
  const labels = page.getByRole('menu', { name: 'Project labels options' });
  await page.getByRole('button', { name: 'Project labels' }).focus();
  await page.keyboard.press('ArrowLeft');
  await expect(labels.getByRole('menuitemcheckbox', { name: 'No project labels' })).toBeFocused();
  await labels.getByRole('menuitemcheckbox', { name: 'Workflow' }).click();

  await expect(
    page.getByRole('button', { name: 'Clear Workflow project label filter' }),
  ).toBeVisible();
  await expect(page.locator('[data-issue-id="MOB-1"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);
  await page.reload();
  await expect(page.locator('[data-issue-id="MOB-1"]')).toBeVisible();

  await page.getByRole('button', { name: 'Clear Workflow project label filter' }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Project properties' }).hover();
  await page.getByRole('button', { name: 'Project labels' }).hover();
  await labels.getByRole('menuitemcheckbox', { name: 'No project labels' }).click();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="MOB-1"]')).toHaveCount(0);
  await labels.getByRole('button', { name: 'Exclude' }).click();
  await labels.getByRole('menuitemcheckbox', { name: 'No project labels' }).click();
  await expect(page.locator('[data-issue-id="MOB-1"]')).toBeVisible();
});

test('project lead and milestone filters survive a saved view reload', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();

  const projectDialog = page.getByRole('dialog', { name: 'Create project' });
  await projectDialog.getByRole('textbox', { name: 'Project name' }).fill('Release');
  await projectDialog.getByRole('textbox', { name: 'Identifier (2–8 letters)' }).fill('REL');
  await projectDialog.getByRole('combobox', { name: 'Project lead' }).click();
  await page.getByRole('option', { name: 'Preview user' }).click();
  await projectDialog.getByRole('button', { name: 'Milestones' }).click();
  await projectDialog.getByRole('textbox', { name: 'Milestone name' }).fill('Beta');
  await projectDialog.getByRole('button', { name: 'Add milestone' }).click();
  await projectDialog.getByRole('button', { name: 'Create project' }).click();

  await page.getByRole('link', { name: 'Issues', exact: true }).click();
  await page.getByRole('button', { name: 'Create issue' }).click();
  const issueDialog = page.getByRole('dialog', { name: 'Create issue' });
  await issueDialog.getByRole('textbox', { name: 'Title' }).fill('Ship release');
  await issueDialog.getByRole('button', { name: 'Create issue' }).click();

  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Release work');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Project properties' }).hover();
  await page.getByRole('button', { name: 'Project lead' }).hover();
  await page
    .getByRole('menu', { name: 'Project lead options' })
    .getByRole('menuitemcheckbox', { name: 'Preview user' })
    .click();
  await page.getByRole('button', { name: 'Milestone name' }).focus();
  await page
    .getByRole('menu', { name: 'Milestone name options' })
    .getByRole('menuitemcheckbox', { name: 'Beta' })
    .click();

  await expect(page.locator('[data-issue-id="REL-1"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);
  await page.reload();
  await expect(
    page.getByRole('button', { name: 'Clear Project lead: Preview user' }),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Clear Project milestone: Beta' })).toBeVisible();
  await expect(page.locator('[data-issue-id="REL-1"]')).toBeVisible();

  await page.getByRole('button', { name: 'Clear Project lead: Preview user' }).click();
  await page.getByRole('button', { name: 'Clear Project milestone: Beta' }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Project properties' }).hover();
  await page.getByRole('button', { name: 'Project lead' }).hover();
  await page
    .getByRole('menu', { name: 'Project lead options' })
    .getByRole('menuitemcheckbox', { name: 'No lead' })
    .click();
  await page.getByRole('button', { name: 'Milestone name' }).focus();
  await page
    .getByRole('menu', { name: 'Milestone name options' })
    .getByRole('menuitemcheckbox', { name: 'No milestones' })
    .click();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="REL-1"]')).toHaveCount(0);
});

test('assigned milestone filters distinguish issues from project milestone names', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const projectDialog = page.getByRole('dialog', { name: 'Create project' });
  await projectDialog.getByRole('textbox', { name: 'Project name' }).fill('Release');
  await projectDialog.getByRole('textbox', { name: 'Identifier (2–8 letters)' }).fill('REL');
  await projectDialog.getByRole('button', { name: 'Milestones' }).click();
  await projectDialog.getByRole('textbox', { name: 'Milestone name' }).fill('Beta');
  await projectDialog.getByRole('button', { name: 'Add milestone' }).click();
  await projectDialog.getByRole('button', { name: 'Create project' }).click();

  await page.getByRole('link', { name: 'Issues', exact: true }).click();
  await page.getByRole('button', { name: 'Create issue' }).click();
  const issueDialog = page.getByRole('dialog', { name: 'Create issue' });
  await issueDialog.getByRole('textbox', { name: 'Title' }).fill('Ship release');
  await issueDialog.getByRole('combobox', { name: 'Milestone' }).click();
  await page.getByRole('option', { name: 'Beta' }).click();
  await issueDialog.getByRole('button', { name: 'Create issue' }).click();

  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Beta work');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Milestone', exact: true }).click();
  const menu = page.getByRole('menu', { name: 'Milestone options' });
  await menu.getByRole('menuitemcheckbox', { name: 'Release · Beta' }).click();

  await expect(page.locator('[data-issue-id="REL-1"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Clear Milestone: Release · Beta' })).toBeVisible();
  await expect(page.locator('[data-issue-id="REL-1"]')).toBeVisible();

  await page.getByRole('button', { name: 'Clear Milestone: Release · Beta' }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Milestone', exact: true }).click();
  await menu.getByRole('menuitemcheckbox', { name: 'No milestone' }).click();
  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="REL-1"]')).toHaveCount(0);
});

test('multiple choices within one filter match any selected value', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Status', exact: true }).click();

  const todo = page.getByRole('menuitemcheckbox', { name: 'Todo', exact: true });
  await todo.click();
  await expect(todo).toHaveAttribute('aria-checked', 'true');
  await page.getByRole('menuitemcheckbox', { name: 'In Progress', exact: true }).click();

  await page.getByRole('button', { name: 'Priority', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'High', exact: true }).click();

  await expect(page.locator('[data-issue-id="APP-121"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-123"]')).toHaveCount(0);

  await page.getByRole('button', { name: 'Clear Todo status filter' }).click();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();
});

test('excluded values remain active in a saved view', async ({ page }) => {
  await page.goto('/#/views');
  await page.getByRole('button', { name: 'New view' }).click();
  await page.getByRole('textbox', { name: 'View name' }).fill('Without medium');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Priority', exact: true }).click();
  await page.getByRole('button', { name: 'Exclude', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'Medium', exact: true }).click();

  await expect(page.getByRole('button', { name: 'Clear excluded Medium priority' })).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-119"]')).toHaveCount(0);
  await expect(page.locator('[data-issue-id="APP-124"]')).toBeVisible();

  await page.getByRole('button', { name: 'Include', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'Medium', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Clear excluded Medium priority' })).toHaveCount(0);
  await expect(page.locator('[data-issue-id="APP-119"]')).toBeVisible();

  await page.getByRole('button', { name: 'Exclude', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'Medium', exact: true }).click();

  await page.reload();
  await expect(page.getByRole('button', { name: 'Clear excluded Medium priority' })).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-119"]')).toHaveCount(0);
});

test('empty label choices include or exclude unlabeled issues', async ({ page }) => {
  await page.goto('/#/triage');
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();

  await page.getByRole('menuitemcheckbox', { name: 'No labels' }).click();
  await expect(page.locator('[data-issue-id="LOCAL-3"]')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Clear no labels filter' })).toBeVisible();

  await page.getByRole('button', { name: 'Exclude', exact: true }).click();
  await page.getByRole('menuitemcheckbox', { name: 'No labels' }).click();
  await expect(page.locator('[data-issue-id="LOCAL-3"]')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Clear has labels filter' })).toBeVisible();
});

test('filter submenus open on focus and close back to the main menu', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Filter issues' }).click();
  const status = page.getByRole('button', { name: 'Status', exact: true });
  await status.focus();
  await expect(page.getByRole('menu', { name: 'Status options' })).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  await expect(page.getByRole('menuitemcheckbox', { name: 'All statuses' })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('menu', { name: 'Status options' })).toHaveCount(0);
  await expect(page.getByRole('textbox', { name: 'Search issues' })).toBeFocused();
  await expect(status).toBeVisible();
});
