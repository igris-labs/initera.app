import { mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import process from 'node:process';
import { test } from '@playwright/test';
import type { Locator } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);
test.skip(!process.env.CAPTURE_DOCS, 'Run with CAPTURE_DOCS=1 to refresh documentation images.');

test('capture documentation screens from the running app', async ({ page }) => {
  test.setTimeout(90_000);
  const output = resolve('../docs/public/images/app');
  await mkdir(output, { recursive: true });

  async function capture(name: string, target?: Locator) {
    await page.waitForTimeout(250);
    const options = { path: resolve(output, `${name}.png`), animations: 'disabled' as const };
    if (target) await target.screenshot(options);
    else await page.screenshot(options);
  }

  await page.goto('/#/issues');
  await page.getByText('APP-123', { exact: true }).waitFor();
  await capture('issues-list');

  await page.getByRole('button', { name: 'Switch workspace' }).click();
  await capture('workspace-picker');
  await page.reload();

  await page.keyboard.press('Meta+k');
  await page.getByRole('dialog', { name: 'Command bar' }).waitFor();
  await capture('command-bar', page.getByRole('dialog', { name: 'Command bar' }));
  await page.reload();

  await page.getByRole('button', { name: 'Create issue' }).click();
  await page.getByRole('dialog', { name: 'Create issue' }).waitFor();
  await capture('create-issue', page.getByRole('dialog', { name: 'Create issue' }));
  await page.reload();

  await page.goto('/#/issues/APP-123/full');
  await page.getByRole('heading', { name: 'Persist issues to local SQLite' }).waitFor();
  await capture('issue-detail');

  await page.getByRole('button', { name: 'Edit issue' }).click();
  await page.getByRole('textbox', { name: 'Edit issue title' }).waitFor();
  await capture('edit-issue');
  await page.reload();

  await page.getByRole('button', { name: 'Blocks', exact: true }).click();
  await page.getByRole('dialog', { name: 'Link issue' }).waitFor();
  await capture('issue-relation-picker', page.getByRole('dialog', { name: 'Link issue' }));

  await page.goto('/#/triage');
  await page.getByText('LOCAL-3', { exact: true }).waitFor();
  await capture('triage');

  await page.goto('/#/archives');
  await page.getByRole('main', { name: 'Archived issues' }).waitFor();
  await capture('archives');

  await page.goto('/#/projects');
  await page.getByRole('table').waitFor();
  await capture('projects-index');

  await page.getByRole('button', { name: 'New project' }).click();
  await page.getByRole('dialog', { name: 'Create project' }).waitFor();
  await capture('create-project', page.getByRole('dialog', { name: 'Create project' }));
  await page.reload();

  await page
    .getByRole('table')
    .getByRole('link', { name: /Desktop app/ })
    .click();
  await page.getByRole('region', { name: 'Project overview' }).waitFor();
  await capture('project-overview');

  await page.goto('/#/projects/Desktop%20app/structure');
  await page.getByRole('button', { name: 'Add node' }).click();
  await page
    .getByRole('dialog', { name: 'Add project node' })
    .getByRole('textbox', { name: 'Node name' })
    .fill('Release planning');
  await page
    .getByRole('dialog', { name: 'Add project node' })
    .getByRole('button', { name: 'Add node' })
    .click();
  await page
    .getByRole('main', { name: 'Project structure' })
    .getByText('Release planning', { exact: true })
    .waitFor();
  await capture('project-structure');

  await page.goto('/#/cycles');
  await page.getByRole('main', { name: 'Cycles' }).waitFor();
  await capture('cycles');

  await page.getByRole('button', { name: 'Set up cycles' }).click();
  await page.getByRole('dialog', { name: 'Set up cycles' }).waitFor();
  await capture('cycle-schedule', page.getByRole('dialog', { name: 'Set up cycles' }));

  await page.goto('/#/views');
  await page.getByRole('heading', { name: 'Issue views' }).waitFor();
  await capture('views');

  await page.goto('/#/time');
  await page.getByRole('heading', { name: 'Time tracking' }).last().waitFor();
  await capture('time-tracking');

  await page.goto('/#/settings');
  await page.getByRole('heading', { name: 'Settings', exact: true }).last().waitFor();
  await capture('settings');

  await page.getByRole('tab', { name: 'Labels' }).click();
  await capture('settings-labels', page.locator('#labels'));

  await page.getByRole('button', { name: 'Help and settings' }).click();
  await page.getByRole('menuitem', { name: 'Help' }).click();
  await page.getByRole('menuitem', { name: 'About' }).click();
  await capture('settings-about', page.getByRole('dialog', { name: 'About Initera' }));
});
