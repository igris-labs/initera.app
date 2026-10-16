import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('projects can be archived, moved to trash, and restored with linked issues', async ({
  page,
}) => {
  await page.goto('/#/projects');
  const project = page.getByRole('table').getByRole('link', { name: /Desktop app/ });
  await expect(project).toBeVisible();

  await page.getByRole('button', { name: 'Archive Desktop app' }).click();
  const archive = page.getByRole('dialog', { name: 'Archive project' });
  await expect(archive.getByText(/active issues remain linked/)).toBeVisible();
  await archive.getByRole('button', { name: 'Archive', exact: true }).click();
  await expect(project).toHaveCount(0);

  await page.reload();
  await page.getByRole('button', { name: 'Archive and trash' }).click();
  await expect(page.getByText('Desktop app', { exact: true })).toBeVisible();
  await expect(page.getByText('Archived')).toBeVisible();
  await page.getByRole('button', { name: 'Restore' }).click();
  await page.getByRole('button', { name: 'Active projects' }).click();
  await expect(project).toBeVisible();

  await page.getByRole('button', { name: 'Move Desktop app to trash' }).click();
  const trash = page.getByRole('dialog', { name: 'Move project to trash' });
  await trash.getByRole('button', { name: 'Move to trash', exact: true }).click();
  await page.reload();
  await page.getByRole('button', { name: 'Archive and trash' }).click();
  await expect(page.getByText('Trash', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Restore' }).click();
  await page.getByRole('button', { name: 'Active projects' }).click();
  await expect(project).toBeVisible();
});

test('project icon and color can be selected, edited, and reopened', async ({ page }) => {
  await page.goto('/#/projects');
  await page.getByRole('button', { name: 'New project' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('button', { name: 'Choose project icon and color' }).click();
  const pickerBounds = await page.locator('.akaza-popover-content').evaluate((popover) => {
    const bounds = popover.getBoundingClientRect();
    const controls = [
      popover.querySelector('input[aria-label="Search icons"]'),
      popover.querySelector('[aria-label="Inbox"]'),
      popover.querySelector('[aria-label="Peach color"]'),
    ];
    return controls.map((control) => {
      const rect = control?.getBoundingClientRect();
      return rect ? { left: rect.left - bounds.left, right: bounds.right - rect.right } : null;
    });
  });
  for (const bounds of pickerBounds) {
    expect(bounds).not.toBeNull();
    expect(bounds!.left).toBeGreaterThanOrEqual(-1);
    expect(bounds!.right).toBeGreaterThanOrEqual(-1);
  }

  await page.getByRole('button', { name: 'Rocket', exact: true }).click();
  await page.getByRole('button', { name: 'Blue color' }).click();
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Orbit');
  await dialog.getByRole('button', { name: 'Create project' }).click();
  await expect(
    page.getByRole('region', { name: 'Project overview' }).locator('svg.tabler-icon-rocket'),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Choose project icon and color' }).click();
  await page.getByRole('button', { name: 'Target', exact: true }).click();
  await page.getByRole('button', { name: 'Green color' }).click();
  await expect(
    page.getByRole('navigation', { name: 'Projects' }).locator('svg.tabler-icon-target'),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole('region', { name: 'Project overview' }).locator('svg.tabler-icon-target'),
  ).toBeVisible();
  await expect(
    page.getByRole('navigation', { name: 'Projects' }).locator('svg.tabler-icon-target'),
  ).toBeVisible();
});

test('workspace projects list opens an overview with persistent project labels', async ({
  page,
}) => {
  await page.goto('/#/projects');
  await expect(page.getByRole('button', { name: 'New project' })).toBeVisible();
  await expect(page.getByRole('table').getByRole('link', { name: /Desktop app/ })).toBeVisible();
  const tableWidth = await page
    .getByRole('table')
    .evaluate((element) => element.getBoundingClientRect().width);
  const viewportWidth = await page
    .locator('.akaza-data-table-viewport')
    .evaluate((element) => element.getBoundingClientRect().width);
  expect(tableWidth).toBeGreaterThanOrEqual(viewportWidth - 1);

  await page.getByRole('button', { name: 'New project' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Release tracker');
  await dialog.getByRole('button', { name: 'Add labels' }).click();
  await page.getByRole('checkbox', { name: 'Workflow' }).click();
  await dialog.getByRole('button', { name: 'Create project' }).click();

  await expect(page).toHaveURL(/Release%20tracker$/);
  await expect(
    page.getByRole('complementary', { name: 'Project details' }).getByText('Workflow'),
  ).toBeVisible();
  await page.getByRole('combobox', { name: 'Project status' }).click();
  await expect(
    page.getByRole('option', { name: 'Backlog' }).locator('svg.tabler-icon-circle-dashed'),
  ).toBeVisible();
  await expect(
    page.getByRole('option', { name: 'In Progress' }).locator('svg.tabler-icon-circle-dot'),
  ).toBeVisible();
  await expect(
    page.getByRole('option', { name: 'Completed' }).locator('svg.tabler-icon-circle-check-filled'),
  ).toBeVisible();
  await page.getByRole('option', { name: 'In Progress' }).click();
  await expect(page.getByRole('combobox', { name: 'Project status' })).toContainText('In Progress');
  const activity = page.getByRole('region', { name: 'Project activity' });
  await expect(activity.getByText(/changed status from Backlog to In Progress/)).toBeVisible();
  await expect(
    page.locator('.akaza-toast').filter({ hasText: 'Release tracker: changed status' }),
  ).toBeVisible({ timeout: 10_000 });
  await page.getByRole('combobox', { name: 'Project priority' }).click();
  await page.getByRole('option', { name: 'High' }).click();
  await expect(page.getByRole('combobox', { name: 'Project priority' })).toContainText('High');
  await page
    .getByRole('complementary', { name: 'Project details' })
    .getByRole('button', { name: 'Add label' })
    .click();
  await page.getByRole('checkbox', { name: 'UI' }).click();
  await expect(
    page.getByRole('complementary', { name: 'Project details' }).getByText('UI', { exact: true }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Choose project target date' }).click();
  await page
    .getByRole('dialog', { name: 'Choose project target date' })
    .locator('.akaza-calendar-cell-trigger:not([data-akaza-outside-view])')
    .first()
    .click();
  await expect(page.getByRole('button', { name: 'Choose project target date' })).not.toContainText(
    'Set target date',
  );
  await page
    .getByRole('region', { name: 'Project resources' })
    .getByRole('button', { name: 'Add document or link…' })
    .click();
  const linkDialog = page.getByRole('dialog', { name: 'Add project link' });
  await linkDialog.getByRole('textbox', { name: 'Link title' }).fill('Brief');
  await linkDialog.getByRole('textbox', { name: 'Link URL' }).fill('https://example.com/brief');
  await linkDialog.getByRole('button', { name: 'Add link' }).click();
  await expect(
    page.getByRole('region', { name: 'Project resources' }).getByRole('link', { name: 'Brief' }),
  ).toHaveAttribute('href', 'https://example.com/brief');
  await page.reload();
  await expect(
    page.getByRole('complementary', { name: 'Project details' }).getByText('Workflow'),
  ).toBeVisible();
  await expect(page.getByRole('combobox', { name: 'Project status' })).toContainText('In Progress');
  await expect(
    page
      .getByRole('region', { name: 'Project activity' })
      .getByText(/changed status from Backlog to In Progress/),
  ).toBeVisible();
  await expect(page.getByRole('combobox', { name: 'Project priority' })).toContainText('High');
  await expect(
    page.getByRole('complementary', { name: 'Project details' }).getByText('UI', { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Choose project target date' })).not.toContainText(
    'Set target date',
  );
  await expect(
    page.getByRole('region', { name: 'Project resources' }).getByRole('link', { name: 'Brief' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Remove Brief' }).click();
  await expect(
    page.getByRole('region', { name: 'Project resources' }).getByRole('link', { name: 'Brief' }),
  ).toHaveCount(0);
  await page.getByRole('link', { name: 'Projects', exact: true }).first().click();
  await expect(
    page.getByRole('table').getByRole('link', { name: /Release tracker/ }),
  ).toBeVisible();
});

test('project updates appear in activity and survive a preview restart', async ({ page }) => {
  await page.goto('/#/projects/Desktop%20app');
  const activity = page.getByRole('region', { name: 'Project activity' });
  const composer = page.getByRole('region', { name: 'Project update composer' });
  const milestoneBottom = await page
    .getByRole('region', { name: 'Project milestones' })
    .evaluate((element) => element.getBoundingClientRect().bottom);
  const composerTop = await composer.evaluate((element) => element.getBoundingClientRect().top);
  expect(composerTop).toBeGreaterThanOrEqual(milestoneBottom);
  await composer.getByRole('button', { name: 'Write first project update' }).click();
  await composer
    .getByRole('textbox', { name: 'Project update' })
    .fill('**Shipped** the first pass.');
  await composer.getByRole('button', { name: 'Post update' }).click();
  await expect(activity.getByText('Shipped')).toBeVisible();
  await expect(activity.locator('li').first().locator('p').first()).toContainText(
    'Preview user posted an update',
  );
  await page.reload();
  await expect(
    page.getByRole('region', { name: 'Project activity' }).getByText('Shipped'),
  ).toBeVisible();
});

test('project lead can be assigned, reopened, and cleared', async ({ page }) => {
  await page.goto('/#/projects');
  await page.getByRole('button', { name: 'New project' }).click();

  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Lead test');
  await dialog.getByRole('combobox', { name: 'Project lead' }).click();
  await page.getByRole('option', { name: 'Preview user' }).click();
  await dialog.getByRole('button', { name: 'Create project' }).click();

  const lead = page.getByRole('complementary', { name: 'Project details' }).getByRole('combobox', {
    name: 'Project lead',
  });
  await expect(lead).toContainText('Preview user');
  await page.reload();
  await expect(lead).toContainText('Preview user');

  await page.getByRole('link', { name: 'Projects', exact: true }).first().click();
  await expect(page.getByRole('table').getByRole('row', { name: /Lead test/ })).toContainText(
    'Preview user',
  );
  await page
    .getByRole('table')
    .getByRole('link', { name: /Lead test/ })
    .click();

  await lead.click();
  await page.getByRole('option', { name: 'No lead' }).click();
  await expect(lead).toContainText('Set lead');
  await expect(
    page.getByRole('region', { name: 'Project activity' }).getByText(/removed the project lead/),
  ).toBeVisible();
});

test('a new project is selectable in issue creation and survives a preview restart', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const projectDialog = page.getByRole('dialog', { name: 'Create project' });
  await projectDialog.getByRole('textbox', { name: 'Project name' }).fill('Mobile app');
  await projectDialog.getByRole('textbox', { name: 'Identifier (2–8 letters)' }).fill('MOB');
  await projectDialog.getByRole('button', { name: 'Create project' }).click();
  await expect(page.getByRole('link', { name: /Mobile app/ })).toBeVisible();
  await page.getByRole('link', { name: 'Issues', exact: true }).click();

  await page.getByRole('button', { name: 'Create issue' }).click();
  const issueDialog = page.getByRole('dialog', { name: 'Create issue' });
  await issueDialog.getByRole('textbox', { name: 'Title' }).fill('Ship mobile navigation');
  await issueDialog.getByRole('combobox', { name: 'Project' }).click();
  await page.getByRole('option', { name: 'Mobile app' }).click();
  await issueDialog.getByRole('button', { name: 'Create issue' }).click();
  await expect(
    page.getByRole('button', { name: /Open MOB-1: Ship mobile navigation/ }),
  ).toBeVisible();
  await page.reload();
  await expect(page.getByRole('link', { name: /Mobile app/ })).toBeVisible();
  await expect(
    page.getByRole('button', { name: /Open MOB-1: Ship mobile navigation/ }),
  ).toBeVisible();
});

test('project creation keeps milestones with their description and target date', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Release');
  await dialog.getByRole('textbox', { name: 'Identifier (2–8 letters)' }).fill('REL');
  await dialog.getByRole('button', { name: 'Milestones' }).click();
  await dialog.getByRole('textbox', { name: 'Milestone name' }).fill('Beta');
  await dialog.getByRole('textbox', { name: 'Milestone description' }).fill('Private rollout');
  const targetDate = dialog.getByRole('group', { name: 'Milestone target date' });
  await targetDate.getByRole('button', { name: 'Choose milestone target date' }).click();
  await expect(page.getByRole('dialog')).toHaveCount(2);
  await targetDate.getByRole('button', { name: 'Choose milestone target date' }).click();
  await targetDate.getByRole('spinbutton', { name: 'month' }).fill('10');
  await targetDate.getByRole('spinbutton', { name: 'day' }).fill('15');
  await targetDate.getByRole('spinbutton', { name: 'year' }).fill('2026');
  await dialog.getByRole('button', { name: 'Add milestone' }).click();
  await expect(dialog.getByText('Beta')).toBeVisible();
  await expect(dialog.getByText('2026-10-15')).toBeVisible();
  await dialog.getByRole('button', { name: 'Create project' }).click();
  await expect(page.getByRole('link', { name: /Release/ })).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('Beta'),
  ).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('Private rollout'),
  ).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('2026-10-15'),
  ).toBeVisible();
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('projects') ?? '{}'));
  expect(
    stored.items.find((project: { name: string }) => project.name === 'Release')?.milestones,
  ).toEqual([
    expect.objectContaining({
      title: 'Beta',
      description: 'Private rollout',
      targetDate: '2026-10-15',
    }),
  ]);
  await page.reload();
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('Beta'),
  ).toBeVisible();
});

test('issues keep milestone assignments and clear them when moved to another project', async ({
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
  await page.goto('/#/projects/Release');
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('0/1 issues'),
  ).toBeVisible();
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open REL-1: Ship release/ }).click();
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  await expect(detail.getByRole('combobox', { name: 'Milestone' })).toContainText('Beta');
  await page.reload();
  await page.getByRole('button', { name: /Open REL-1: Ship release/ }).click();
  await expect(detail.getByRole('combobox', { name: 'Milestone' })).toContainText('Beta');

  await detail.getByRole('combobox', { name: 'Project' }).click();
  await page.getByRole('option', { name: 'Desktop app' }).click();
  await page
    .getByRole('dialog', { name: 'Move REL-1 to Desktop app' })
    .getByRole('button', { name: 'Move issue' })
    .click();
  await expect(detail.getByRole('combobox', { name: 'Milestone' })).toHaveCount(0);

  await detail.getByRole('combobox', { name: 'Project' }).click();
  await page.getByRole('option', { name: 'Release' }).click();
  await page
    .getByRole('dialog', { name: 'Move REL-1 to Release' })
    .getByRole('button', { name: 'Move issue' })
    .click();
  await detail.getByRole('combobox', { name: 'Milestone' }).click();
  await page.getByRole('option', { name: 'Beta' }).click();
  await page.goto('/#/projects/Release');
  await page.getByRole('button', { name: 'Edit project', exact: true }).click();
  const editing = page.getByRole('dialog', { name: 'Edit project' });
  await editing.getByRole('button', { name: 'Remove milestone Beta' }).click();
  await editing.getByRole('button', { name: 'Save project' }).click();
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open REL-1: Ship release/ }).click();
  await expect(detail.getByRole('combobox', { name: 'Milestone' })).toHaveCount(0);
});

test('project dates select from styled calendar pills and issue rich text survives reload', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const project = page.getByRole('dialog', { name: 'Create project' });
  const projectName = project.getByRole('textbox', { name: 'Project name' });
  await projectName.fill('Calendar project');
  await projectName.press('ControlOrMeta+a');
  await projectName.press('ControlOrMeta+b');
  await project.getByRole('button', { name: 'Choose project start date' }).click();
  const calendar = page.getByRole('dialog', { name: 'Choose project start date' });
  await expect(calendar.locator('.akaza-calendar-grid')).toBeVisible();
  await calendar
    .locator('.akaza-calendar-cell-trigger:not([data-akaza-outside-view])')
    .first()
    .click();
  await expect(
    project.getByRole('button', { name: 'Choose project start date' }),
  ).not.toContainText('Start date');
  await project.getByRole('button', { name: 'Create project' }).click();
  await expect(
    page.getByRole('region', { name: 'Project overview' }).locator('strong'),
  ).toContainText('Calendar project');

  await page.getByRole('button', { name: 'Create issue' }).click();
  const issue = page.getByRole('dialog', { name: 'Create issue' });
  const issueTitle = issue.getByRole('textbox', { name: 'Title' });
  await issueTitle.fill('Rich issue');
  await issueTitle.press('ControlOrMeta+a');
  await issueTitle.press('ControlOrMeta+b');
  const description = issue.getByRole('textbox', { name: 'Description' });
  await description.fill('A formatted note');
  await expect(issueTitle).toHaveText('Rich issue');
  await description.press('ControlOrMeta+a');
  await issue.getByRole('button', { name: 'Bold' }).click();
  await expect(issueTitle).toHaveText('Rich issue');
  await issue.getByRole('button', { name: 'Create issue' }).click();
  await page
    .getByRole('navigation', { name: 'Project pages' })
    .getByRole('link', { name: 'Issues' })
    .click();
  await page.getByRole('button', { name: /Open [A-Z]+-\d+: Rich issue/ }).click();
  await expect(page.getByRole('textbox', { name: 'Issue title' }).locator('strong')).toContainText(
    'Rich issue',
  );
  await expect(
    page.getByRole('textbox', { name: 'Issue description' }).locator('strong'),
  ).toContainText('A formatted note');
  await page.reload();
  await page.getByRole('button', { name: /Open [A-Z]+-\d+: Rich issue/ }).click();
  await expect(page.getByRole('textbox', { name: 'Issue title' }).locator('strong')).toContainText(
    'Rich issue',
  );
  await expect(
    page.getByRole('textbox', { name: 'Issue description' }).locator('strong'),
  ).toContainText('A formatted note');
});

test('project details and milestones can be edited after creation', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const creation = page.getByRole('dialog', { name: 'Create project' });
  await creation.getByRole('textbox', { name: 'Project name' }).fill('Release plan');
  await creation.getByRole('button', { name: 'Milestones' }).click();
  await creation.getByRole('textbox', { name: 'Milestone name' }).fill('Beta');
  await creation.getByRole('button', { name: 'Add milestone' }).click();
  await creation.getByRole('button', { name: 'Create project' }).click();

  await page.getByRole('button', { name: 'Edit project', exact: true }).click();
  const editing = page.getByRole('dialog', { name: 'Edit project' });
  await editing.getByRole('textbox', { name: 'Project name' }).fill('Release roadmap');
  await editing.getByRole('textbox', { name: 'Project summary' }).fill('Ship a stable release');
  await editing.getByRole('button', { name: 'Edit milestone Beta' }).click();
  await editing.getByRole('textbox', { name: 'Milestone name' }).fill('Private beta');
  await editing.getByRole('textbox', { name: 'Milestone description' }).fill('Invite early users');
  await editing.getByRole('button', { name: 'Save milestone' }).click();
  await editing.getByRole('button', { name: 'Save project' }).click();

  await expect(page).toHaveURL(/Release%20roadmap$/);
  const overview = page.getByRole('region', { name: 'Project overview' });
  await expect(overview.getByText('Ship a stable release')).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('Private beta'),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole('region', { name: 'Project milestones' }).getByText('Invite early users'),
  ).toBeVisible();
});

test('issues and their sub-issues can be edited and reopened', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open APP-123:/ }).click();
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  await detail.getByRole('button', { name: 'Edit issue' }).click();
  await detail.getByRole('textbox', { name: 'Edit issue title' }).fill('Persist local workspace');
  await detail
    .getByRole('textbox', { name: 'Edit issue description' })
    .fill('Keep edits after restart');
  await detail.getByRole('button', { name: 'Save changes' }).click();
  await expect(detail.getByText('Keep edits after restart')).toBeVisible();

  await detail.getByRole('button', { name: 'Add sub-issue' }).click();
  const composer = page.getByRole('dialog', { name: 'Create sub-issue' });
  await composer.getByRole('textbox', { name: 'Title' }).fill('Verify restart');
  await composer.getByRole('button', { name: 'Choose issue start date' }).click();
  await page
    .getByRole('dialog', { name: 'Choose issue start date' })
    .locator('.akaza-calendar-cell-trigger:not([data-akaza-outside-view])')
    .first()
    .click();
  await composer.getByRole('button', { name: 'Choose issue due date' }).click();
  await page
    .getByRole('dialog', { name: 'Choose issue due date' })
    .locator('.akaza-calendar-cell-trigger:not([data-akaza-outside-view])')
    .first()
    .click();
  await composer.getByRole('button', { name: 'Create sub-issue' }).click();
  const child = detail.getByRole('button', { name: /APP-\d+ Verify restart/ });
  await expect(child).toBeVisible();
  await child.click();
  const childDetail = page.getByRole('complementary', { name: 'Issue details' }).last();
  await expect(
    childDetail.getByRole('button', { name: /APP-123 · Persist local workspace/ }),
  ).toBeVisible();
  await expect(childDetail.locator('dl > dt')).toHaveText([
    'Status',
    'Priority',
    'Project',
    'Estimate',
    'Cycle',
    'Parent',
    'Start date',
    'Due date',
  ]);
  await childDetail.getByRole('button', { name: 'Edit issue' }).click();
  await childDetail
    .getByRole('textbox', { name: 'Edit issue title' })
    .fill('Verify restart and reopen');
  await childDetail.getByRole('button', { name: 'Save changes' }).click();
  await childDetail.getByRole('combobox', { name: 'Parent issue' }).click();
  await page.getByRole('option', { name: 'No parent' }).click();
  await expect(
    childDetail.getByRole('button', { name: /APP-123 · Persist local workspace/ }),
  ).toHaveCount(0);
  await childDetail.getByRole('combobox', { name: 'Parent issue' }).click();
  await page.getByRole('option', { name: /APP-123 · Persist local workspace/ }).click();
  await page.reload();
  await expect(
    page
      .getByRole('complementary', { name: 'Issue details' })
      .getByRole('button', { name: /APP-123 · Persist local workspace/ }),
  ).toBeVisible();
  await expect(
    page
      .getByRole('complementary', { name: 'Issue details' })
      .getByText('Verify restart and reopen'),
  ).toBeVisible();
  await expect(
    page.getByRole('complementary', { name: 'Issue details' }).locator('dl > dt'),
  ).toHaveText([
    'Status',
    'Priority',
    'Project',
    'Estimate',
    'Cycle',
    'Parent',
    'Start date',
    'Due date',
  ]);
});
