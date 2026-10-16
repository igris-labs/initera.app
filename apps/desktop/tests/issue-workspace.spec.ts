import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('Escape closes issue preview when the key event targets the window', async ({ page }) => {
  const errors: Error[] = [];
  page.on('pageerror', (error) => errors.push(error));

  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open APP-123:/ }).click();
  await expect(page).toHaveURL(/#\/issues\/APP-123/);

  await page.evaluate(() => window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' })));

  await expect(page).toHaveURL(/#\/issues$/);
  expect(errors).toEqual([]);
});

test('label options place the color before the name and the selected check on the right', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  await page.getByRole('button', { name: 'Add labels' }).click();
  const option = page.getByRole('checkbox', { name: 'Bug' });
  await option.locator('..').locator('.akaza-checkbox-label').click();
  await expect(option).toHaveAttribute('aria-checked', 'true');
  await page.mouse.move(0, 0);
  await expect(option.locator('svg.tabler-icon-check')).toBeVisible();
  const positions = await option.locator('..').evaluate((row) => {
    const color = row.querySelector('.akaza-checkbox-label span:first-child');
    const name = row.querySelector('.akaza-checkbox-label span:last-child');
    const check = row.querySelector('.akaza-checkbox-indicator');
    return [color, name, check].map((element) => element?.getBoundingClientRect().x ?? -1);
  });
  expect(positions[0]).toBeGreaterThanOrEqual(0);
  expect(positions[0]).toBeLessThan(positions[1]);
  expect(positions[1]).toBeLessThan(positions[2]);
  await page.getByRole('textbox', { name: 'Title' }).click();
  await page.getByRole('button', { name: 'Labels: Bug' }).click();
  await expect(
    page.getByRole('checkbox', { name: 'Bug' }).locator('svg.tabler-icon-check'),
  ).toBeVisible();
});

test('quick capture persists a new issue and routes unassigned work to triage', async ({
  page,
}) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('textbox', { name: 'Title' })).toBeFocused();
  await dialog.getByRole('button', { name: 'Add labels' }).click();
  await page.getByRole('checkbox', { name: 'Bug' }).click();
  await expect(dialog.getByRole('button', { name: 'Labels: Bug' })).toContainText('Bug');
  await expect(dialog.getByRole('button', { name: 'Labels: Bug' })).not.toContainText('1 label');
  await dialog.getByRole('textbox', { name: 'Title' }).fill('Review local capture');
  await dialog.getByRole('textbox', { name: 'Title' }).press('Enter');
  await expect(page).toHaveURL(/#\/triage$/);
  await expect(
    page.getByRole('button', { name: /Open LOCAL-4: Review local capture/ }),
  ).toBeVisible();
  await page.getByRole('button', { name: /Open LOCAL-4: Review local capture/ }).click();
  await expect(
    page.getByRole('complementary', { name: 'Issue details' }).getByText('Bug'),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole('button', { name: /Open LOCAL-4: Review local capture/ }),
  ).toBeVisible();
});

test('multiple labels stay legible from composer to list and full issue', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByRole('textbox', { name: 'Title' }).fill('Check multiple labels');
  await dialog.getByRole('combobox', { name: 'Project' }).click();
  await page.getByRole('option', { name: 'Desktop app' }).click();
  await dialog.getByRole('button', { name: 'Add labels' }).click();
  for (const label of ['Bug', 'Feature', 'Improvement']) {
    await page.getByRole('checkbox', { name: label }).click();
  }

  const trigger = dialog.getByRole('button', { name: 'Labels: Bug, Feature, Improvement' });
  await expect(trigger).toContainText('3 labels');
  await expect(trigger.locator('[aria-hidden="true"] > span')).toHaveCount(3);
  await dialog.getByRole('textbox', { name: 'Title' }).click();
  await dialog.getByRole('button', { name: 'Create issue' }).click();

  const row = page.getByRole('button', { name: /Open APP-125: Check multiple labels/ });
  await expect(row).toBeVisible();
  for (const label of ['Bug', 'Feature', 'Improvement']) {
    await expect(row.getByText(label, { exact: true })).toBeVisible();
  }

  await row.click();
  await page.getByRole('button', { name: 'Open full issue view' }).click();
  const details = page.getByRole('region', { name: 'Full issue view' });
  for (const label of ['Bug', 'Feature', 'Improvement']) {
    await expect(details.getByText(label, { exact: true })).toBeVisible();
  }

  await expect(details.getByRole('button', { name: 'Add label' })).toBeVisible();
});

test('composer property menu stays aligned below its compact trigger', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  await page.getByRole('combobox', { name: 'Priority' }).click();

  const menu = page.locator('.akaza-select-content');
  await expect(menu).toHaveAttribute('data-akaza-side', 'bottom');
  const labels = await menu
    .locator('.akaza-select-option-label')
    .evaluateAll((items) => items.slice(0, 2).map((item) => item.getBoundingClientRect().x));
  expect(labels[0]).toBe(labels[1]);
  await expect(menu.getByRole('option', { name: 'Low' })).toHaveCSS('font-size', '12px');
  await menu.getByRole('searchbox', { name: 'Set priority...' }).press('4');
  await expect(page.getByRole('combobox', { name: 'Priority' })).toContainText('High');
});

test('issue route, property edit, project navigation, and narrow layout work', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open APP-123:/ }).click();
  await expect(page).toHaveURL(/#\/issues\/APP-123/);
  await expect(page.getByRole('heading', { name: 'Persist issues to local SQLite' })).toBeVisible();
  await expect(
    page.getByRole('complementary', { name: 'Issue details' }).locator('..'),
  ).not.toHaveClass(/issue-detail-enter-active/);
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  await expect(detail.locator('dl > dt')).toHaveText([
    'Status',
    'Priority',
    'Project',
    'Estimate',
    'Cycle',
    'Parent',
    'Due date',
  ]);
  await expect(detail.locator('dl > dd').last()).toContainText('Sep 30');
  const heading = await detail.getByRole('heading', { name: 'Properties' }).boundingBox();
  const firstLabel = await detail.locator('dl > dt').first().boundingBox();
  expect(Math.abs(heading!.x - firstLabel!.x)).toBeLessThan(1);
  await expect(
    page.getByRole('complementary', { name: 'Issue details' }).getByText('No date'),
  ).toHaveCount(0);
  await page.getByRole('combobox', { name: 'Priority' }).click();
  await page.getByRole('option', { name: 'High' }).click();
  await expect(page.getByRole('combobox', { name: 'Priority' })).toContainText('High');
  await expect(page.getByText('APP-123 priority changed to High')).toBeVisible();
  await page.getByRole('button', { name: 'Close issue details' }).click();
  await expect(page.getByRole('button', { name: /Open APP-123:/ })).toBeFocused();
  await page.getByRole('link', { name: /Design system/ }).click();
  await expect(page).toHaveURL(/#\/projects\/Design%20system$/);
  await expect(
    page
      .getByRole('region', { name: 'Project overview' })
      .getByRole('heading', { name: 'Design system' }),
  ).toBeVisible();
  await page.setViewportSize({ width: 600, height: 700 });
  await page.getByRole('button', { name: 'Show sidebar' }).click();
  const sidebar = page.getByRole('dialog', { name: 'Workspace navigation' });
  await expect(sidebar.getByRole('navigation', { name: 'Primary' })).toBeVisible();
  await page
    .getByRole('dialog', { name: 'Workspace navigation' })
    .getByRole('button', { name: 'Hide sidebar' })
    .click();
  await expect(page.getByRole('button', { name: 'Show sidebar' })).toBeVisible();
  await expect(page.locator('body')).toHaveJSProperty('scrollWidth', 600);
  await page.getByRole('button', { name: 'Create issue' }).click();
  await page.getByRole('button', { name: 'Add labels' }).click();
  const menu = page.locator('.akaza-popover-content');
  await expect(menu).toBeInViewport();
  const picker = await menu.boundingBox();
  expect(picker).not.toBeNull();
  expect(picker!.x).toBeGreaterThanOrEqual(0);
  expect(picker!.x + picker!.width).toBeLessThanOrEqual(600);
  expect(picker!.y + picker!.height).toBeLessThanOrEqual(700);
});

test('marking an issue Done from details keeps it in the list', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open APP-121:/ }).click();
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  await detail.getByRole('combobox', { name: 'Status' }).click();
  await page.getByRole('option', { name: 'Done' }).click();
  await page.getByRole('button', { name: 'Close issue details' }).click();
  await expect(
    page
      .getByRole('region', { name: 'Done issues' })
      .getByRole('button', { name: /Open APP-121:/ }),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'All', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
});

test('opening issue details keeps list text in place', async ({ page }) => {
  await page.goto('/#/issues');
  await page.evaluate(() => document.fonts.ready);
  const issue = page.getByRole('button', { name: /Open APP-123:/ });
  const title = issue.getByText('Persist issues to local SQLite');
  const before = await title.boundingBox();

  await issue.click();
  await expect(page.getByRole('complementary', { name: 'Issue details' })).toBeVisible();
  const after = await title.boundingBox();

  expect(after).toMatchObject({ x: before?.x, width: before?.width });
});

test('issue preview closes on outside click and Escape while property menus stay open', async ({
  page,
}) => {
  await page.goto('/#/issues');
  const issue = page.getByRole('button', { name: /Open APP-123:/ });
  await issue.click();
  const detail = page.getByRole('complementary', { name: 'Issue details' });
  const status = detail.getByRole('combobox', { name: 'Status' });
  const resting = await status.evaluate((element) => ({
    background: getComputedStyle(element).backgroundColor,
    radius: Number.parseFloat(getComputedStyle(element).borderTopLeftRadius),
    width: element.getBoundingClientRect().width,
    panelWidth: element.closest('[aria-label="Issue details"]')!.getBoundingClientRect().width,
  }));
  expect(resting.background).toBe('rgba(0, 0, 0, 0)');
  expect(resting.radius).toBe(8);
  expect(resting.width).toBeLessThan(resting.panelWidth / 2);
  await status.hover();
  await expect
    .poll(() => status.evaluate((element) => getComputedStyle(element).backgroundColor))
    .not.toBe(resting.background);
  await status.click();
  await expect(page.getByRole('listbox')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(detail).toBeVisible();
  await page.getByRole('main', { name: 'All issues' }).click({ position: { x: 600, y: 750 } });
  await expect(detail).not.toBeVisible();

  await issue.click();
  await expect(detail).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(detail).not.toBeVisible();

  await issue.click();
  await page.getByRole('button', { name: 'Help and settings' }).click();
  await expect(detail).not.toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Settings' })).toBeVisible();
});

test('issue preview expands and returns with label edits preserved', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: /Open APP-123:/ }).click();
  await page.getByRole('button', { name: 'Open full issue view' }).click();
  await expect(page).toHaveURL(/#\/issues\/APP-123\/full/);
  const fullView = page.getByRole('region', { name: 'Full issue view' });
  await expect(
    fullView.getByRole('heading', { name: 'Persist issues to local SQLite' }),
  ).toBeVisible();
  await fullView.getByRole('button', { name: 'Add label' }).click();
  await page.getByRole('checkbox', { name: 'Bug' }).click();
  await fullView.getByRole('heading', { name: 'Persist issues to local SQLite' }).click();
  await expect(fullView.getByText('Bug', { exact: true }).first()).toBeVisible();
  await fullView.getByRole('button', { name: 'Back to issue preview' }).click();
  await expect(page).toHaveURL(/#\/issues\/APP-123\?from=/);
  await expect(page.getByRole('complementary', { name: 'Issue details' })).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole('complementary', { name: 'Issue details' }).getByText('Bug'),
  ).toBeVisible();
});

test('display settings change the issue list and persist', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Display' }).click();
  const listTab = page.getByRole('button', { name: 'List', exact: true });
  const layoutBackgrounds = await listTab.evaluate((element) => ({
    selected: getComputedStyle(element).backgroundColor,
    container: getComputedStyle(element.parentElement!).backgroundColor,
  }));
  expect(layoutBackgrounds.selected).not.toBe(layoutBackgrounds.container);
  await page.getByRole('combobox', { name: 'Grouping' }).click();
  await page.getByRole('option', { name: 'None' }).click();
  await expect(page.getByRole('heading', { name: 'Todo 3' })).toHaveCount(0);
  await page.getByRole('switch', { name: 'Show project' }).click();
  await expect(page.getByRole('switch', { name: 'Show project' })).not.toBeChecked();
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Todo 3' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Display' }).click();
  await expect(page.getByRole('switch', { name: 'Show project' })).not.toBeChecked();
});

test('workbench controls open an inset overview and switch layouts', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Toggle issue overview' }).click();
  const overview = page.getByRole('complementary', { name: 'Issue overview' });
  await expect(overview).toBeVisible();
  await expect(overview.getByText('Desktop app')).toBeVisible();
  await overview.getByRole('tab', { name: 'Labels' }).click();
  await expect(overview.getByText('Core')).toBeVisible();
  const box = await overview.boundingBox();
  expect(box).toBeTruthy();
  expect(box!.x + box!.width).toBeLessThan(page.viewportSize()!.width);
  await page.getByRole('main', { name: 'All issues' }).click({ position: { x: 600, y: 750 } });
  await expect(overview).toHaveCount(0);
  await page.getByRole('button', { name: 'Toggle issue overview' }).click();
  await expect(overview).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(overview).toHaveCount(0);
  await page.getByRole('button', { name: 'Toggle issue overview' }).click();
  await expect(overview).toBeVisible();
  await page.getByRole('button', { name: 'Display' }).click();
  await expect(overview).toHaveCount(0);
  await page.getByRole('button', { name: 'Board', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Todo lane' })).toBeVisible();
  await page.getByRole('button', { name: 'Toggle issue overview' }).click();
  await expect(overview).toBeVisible();
  await page.getByRole('button', { name: 'Toggle issue overview' }).click();
  await expect(overview).toHaveCount(0);
  await page.getByRole('button', { name: 'Filter issues' }).click();
  await page.getByRole('textbox', { name: 'Search issues' }).fill('APP-123');
  await expect(page.locator('[data-issue-id="APP-123"]')).toBeVisible();
  await expect(page.locator('[data-issue-id="APP-121"]')).toHaveCount(0);
});

test('desktop sidebar moves the workbench when toggled', async ({ page }) => {
  await page.goto('/#/issues');
  const sidebar = page.getByRole('complementary', { name: 'Workspace navigation' });
  const root = page.locator('.akaza-sidebar-root');
  await expect(sidebar).toBeVisible();
  const expandedWidth = await root.evaluate((element) => getComputedStyle(element).width);
  await expect(root).toHaveCSS('transition-property', 'width');
  await page.getByRole('button', { name: 'Hide sidebar' }).click();
  await expect(root).toHaveAttribute('data-akaza-collapsed', 'true');
  await expect(root).toHaveCSS('width', '0px');
  await page.getByRole('button', { name: 'Show sidebar' }).click();
  await expect(root).toHaveCSS('width', expandedWidth);
  await expect(sidebar).toBeVisible();
});

test('dialog and issue detail enter and exit with motion', async ({ page }) => {
  await page.goto('/#/issues');
  await page.evaluate(() => {
    document.body.dataset.dialogMotion = '';
    new MutationObserver(() => {
      const dialog = document.querySelector('.akaza-dialog-content');
      if (dialog?.classList.contains('akaza-dialog-enter-active'))
        document.body.dataset.dialogMotion += 'enter ';
      if (dialog?.classList.contains('akaza-dialog-leave-active'))
        document.body.dataset.dialogMotion += 'leave ';
    }).observe(document.body, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ['class'],
    });
  });
  await page.getByRole('button', { name: 'Create issue' }).click();
  await expect(page.locator('body')).toHaveAttribute('data-dialog-motion', /enter/);
  await page.getByRole('button', { name: 'Close dialog' }).click();
  await expect(page.locator('body')).toHaveAttribute('data-dialog-motion', /leave/);

  await page.evaluate(() => {
    document.body.dataset.issueMotion = '';
    new MutationObserver(() => {
      const panel = document.querySelector('[aria-label="Issue details"]')?.parentElement;
      if (panel?.classList.contains('issue-detail-enter-from')) {
        document.body.dataset.issueStartX = getComputedStyle(panel).transform;
        document.body.dataset.issueDuration = getComputedStyle(panel).transitionDuration;
      }

      if (panel?.classList.contains('issue-detail-enter-active'))
        document.body.dataset.issueMotion += 'enter ';
      if (panel?.classList.contains('issue-detail-leave-active'))
        document.body.dataset.issueMotion += 'leave ';
    }).observe(document.body, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ['class'],
    });
  });
  await page.getByRole('button', { name: /Open APP-123:/ }).click();
  await expect(page.locator('body')).toHaveAttribute('data-issue-motion', /enter/);
  await expect(page.locator('body')).toHaveAttribute(
    'data-issue-start-x',
    /matrix\(1, 0, 0, 1, [1-9]\d+, 0\)/,
  );
  await expect(page.locator('body')).toHaveAttribute('data-issue-duration', /0\.22s/);
  await page.getByRole('button', { name: 'Close issue details' }).click();
  await expect(page.locator('body')).toHaveAttribute('data-issue-motion', /leave/);
});
