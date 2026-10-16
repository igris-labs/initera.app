import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('cycle schedule stays inside a short window', async ({ page }) => {
  await page.setViewportSize({ width: 600, height: 500 });
  await page.goto('/#/cycles');
  await page.getByRole('button', { name: 'Set up cycles' }).click();

  const dialog = page.getByRole('dialog', { name: 'Set up cycles' });
  const bounds = await dialog.boundingBox();

  expect(bounds).not.toBeNull();
  expect(bounds!.x).toBeGreaterThanOrEqual(0);
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(600);
  expect(bounds!.y).toBeGreaterThanOrEqual(0);
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(500);
  await expect(dialog.getByRole('button', { name: 'Start cycles' })).toBeInViewport();
  expect(
    await dialog
      .locator('.akaza-dialog-body')
      .evaluate((body) => body.scrollHeight > body.clientHeight),
  ).toBe(true);
});

test('cycles can be created, populated, completed, and reopened after reload', async ({ page }) => {
  await page.goto('/#/cycles');
  await expect(page.getByRole('link', { name: 'Cycles' })).toBeVisible();
  await page.getByRole('button', { name: 'New cycle' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create cycle' });
  await dialog.getByRole('textbox', { name: 'Cycle name' }).fill('October sprint');

  for (const date of ['start', 'end']) {
    await dialog.getByRole('button', { name: `Choose cycle ${date} date` }).click();
    await page
      .getByRole('dialog', { name: `Choose cycle ${date} date` })
      .locator('.akaza-calendar-cell-trigger:not([data-akaza-outside-view])')
      .first()
      .click();
  }

  await dialog.getByRole('button', { name: 'Create cycle' }).click();
  const cycle = page.getByRole('main', { name: 'Cycles' });
  await expect(cycle.getByText('October sprint')).toBeVisible();
  await expect(cycle.getByText(/\b\d{4}-\d{2}-\d{2}\b/)).toHaveCount(0);
  await cycle.getByRole('button', { name: 'Edit' }).click();
  const edit = page.getByRole('dialog', { name: 'Edit cycle' });
  await edit.getByRole('textbox', { name: 'Cycle description' }).fill('Release planning');
  await edit.getByRole('button', { name: 'Save changes' }).click();
  await expect(cycle.getByText('Release planning')).toBeVisible();
  await cycle.getByRole('button', { name: 'Add issues' }).click();
  const picker = page.getByRole('dialog', { name: 'Issues in October sprint' });
  await picker.getByRole('textbox', { name: 'Search cycle issues' }).fill('APP-123');
  await picker.getByRole('button', { name: /APP-123/ }).click();
  await picker.getByRole('button', { name: 'Done' }).click();
  await expect(cycle.getByText('APP-123')).toBeVisible();
  await cycle.getByRole('button', { name: 'Complete' }).click();
  await page
    .getByRole('dialog', { name: 'Complete October sprint' })
    .getByRole('button', { name: 'Complete cycle' })
    .click();
  await expect(cycle.getByText('completed', { exact: true })).toBeVisible();
  await page.reload();
  await expect(cycle.getByText('October sprint')).toBeVisible();
  await expect(cycle.getByText('APP-123')).toBeVisible();

  await page.goto('/#/issues');
  await page.keyboard.press('ControlOrMeta+k');
  await page.getByRole('combobox', { name: 'Search commands' }).fill('October sprint');
  await expect(page.getByRole('option', { name: /October sprint/ })).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#\/cycles$/);
  await page.keyboard.press('n');
  await page.keyboard.press('c');
  await expect(page.getByRole('dialog', { name: 'Create cycle' })).toBeVisible();
});

test('cycle schedule creates upcoming cycles and survives reload', async ({ page }) => {
  await page.goto('/#/cycles');
  await page.getByRole('button', { name: 'Set up cycles' }).click();
  const dialog = page.getByRole('dialog', { name: 'Set up cycles' });
  await expect(dialog.getByText('A predictable rhythm')).toBeVisible();
  await dialog.getByRole('textbox', { name: 'Cycle series name' }).fill('Delivery');
  await dialog.getByRole('spinbutton', { name: 'Recurrence interval' }).fill('4');
  await dialog.getByRole('button', { name: 'Start cycles' }).click();

  const cycles = page.getByRole('main', { name: 'Cycles' });
  await expect(cycles.getByText('Delivery 1', { exact: true })).toBeVisible();
  await expect(cycles.getByText('Delivery 2', { exact: true })).toBeVisible();
  await expect(cycles.getByText('Delivery · 14-day cycles')).toBeVisible();

  await cycles.getByRole('button', { name: 'Add issues' }).first().click();
  const picker = page.getByRole('dialog', { name: 'Issues in Delivery 1' });
  await picker.getByRole('textbox', { name: 'Search cycle issues' }).fill('APP-123');
  await picker.getByRole('button', { name: /APP-123/ }).click();
  await picker.getByRole('button', { name: 'Done' }).click();

  await cycles.getByRole('link', { name: /APP-123/ }).click();
  await expect(page.getByRole('combobox', { name: 'Cycle', exact: true })).toContainText(
    'Delivery 1',
  );

  await page.reload();
  await expect(page.getByRole('combobox', { name: 'Cycle', exact: true })).toContainText(
    'Delivery 1',
  );

  await page.goto('/#/cycles');
  await cycles.getByRole('combobox', { name: 'Cycle series' }).click();
  await page.getByRole('option', { name: 'Delivery' }).click();
  await expect(cycles.getByText('Every 28 days')).toBeVisible();
  await cycles.getByRole('button', { name: 'Disable series' }).click();
  await expect(cycles.getByText('Scheduling disabled', { exact: true })).toBeVisible();
  await page.reload();
  await cycles.getByRole('combobox', { name: 'Cycle series' }).click();
  await page.getByRole('option', { name: 'Delivery' }).click();
  await expect(cycles.getByRole('button', { name: 'Enable series' })).toBeVisible();
  await cycles.getByRole('button', { name: 'Enable series' }).click();
  await expect(cycles.getByText('12 upcoming')).toBeVisible();
  await cycles.getByRole('button', { name: 'Edit series' }).click();
  const editSchedule = page.getByRole('dialog', { name: 'Edit cycle schedule' });
  const preview = editSchedule.getByRole('region', { name: 'Schedule preview' });
  await expect(preview).toContainText('Current:');
  await editSchedule.getByRole('spinbutton', { name: 'Recurrence interval' }).fill('5');
  await expect(preview).toContainText('Cycle 2');
  await expect(preview).toContainText('UTC');
  await expect(preview).not.toContainText(/\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/);
  await editSchedule.getByRole('button', { name: 'Save schedule' }).click();
  await expect(cycles.getByText('Every 35 days')).toBeVisible();
  await expect(cycles.getByText('Delivery 1', { exact: true })).toBeVisible();
});

test('project scope limits issues available to a cycle', async ({ page }) => {
  await page.goto('/#/cycles');
  await page.getByRole('button', { name: 'New cycle' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create cycle' });
  await dialog.getByRole('textbox', { name: 'Cycle name' }).fill('Desktop work');
  await dialog.getByRole('button', { name: 'Design system' }).click();

  for (const date of ['start', 'end']) {
    await dialog.getByRole('button', { name: `Choose cycle ${date} date` }).click();
    await page
      .getByRole('dialog', { name: `Choose cycle ${date} date` })
      .locator('.akaza-calendar-cell-trigger:not([data-akaza-outside-view])')
      .first()
      .click();
  }

  await dialog.getByRole('button', { name: 'Create cycle' }).click();
  await page
    .getByRole('main', { name: 'Cycles' })
    .getByRole('button', { name: 'Add issues' })
    .click();
  const picker = page.getByRole('dialog', { name: 'Issues in Desktop work' });
  await picker.getByRole('textbox', { name: 'Search cycle issues' }).fill('DSN-42');
  await expect(picker.getByRole('button', { name: /DSN-42/ })).toHaveCount(0);
  await picker.getByRole('textbox', { name: 'Search cycle issues' }).fill('APP-123');
  await expect(picker.getByRole('button', { name: /APP-123/ })).toBeVisible();
});

test('preview catch-up progress survives reloads', async ({ page }) => {
  await page.addInitScript(() => {
    if (sessionStorage.getItem('cycle-batch-seeded')) return;

    sessionStorage.setItem('cycle-batch-seeded', 'true');
    const settings = {
      id: 'batch-series',
      name: 'Batch',
      durationDays: 1,
      intervalDays: 2,
      enabled: false,
      startDate: '2020-01-01',
      rolloverPolicy: 'leave',
      projectIds: [],
      timeZone: 'UTC',
      startLocalTime: '09:00',
    };
    const items = Array.from({ length: 205 }, (_, index) => {
      const date = new Date(Date.UTC(2020, 0, 1 + index * 2)).toISOString().slice(0, 10);
      return {
        id: `cycle-${index}`,
        title: `Batch ${index + 1}`,
        startDate: date,
        endDate: date,
        status: 'planned',
        scheduled: true,
        seriesId: settings.id,
        scheduleIndex: index,
        issueIds: [],
        projectIds: [],
        rolloverPolicy: 'leave',
      };
    });
    localStorage.setItem(
      'cycles',
      JSON.stringify({ items, series: [settings], settings, reconcileProgress: [] }),
    );
  });
  await page.clock.pauseAt(new Date('2026-10-10T12:00:00Z'));
  await page.goto('/#/cycles');
  await expect(page.getByRole('status')).toContainText('100 of 205');
  await page.reload();
  await expect(page.getByRole('status')).toContainText('200 of 205');
  await page.reload();
  await expect(page.getByRole('status')).toHaveCount(0);
  await expect(page.getByText('Batch 205', { exact: true })).toBeVisible();
});
