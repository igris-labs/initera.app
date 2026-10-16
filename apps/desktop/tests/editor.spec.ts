import { Buffer } from 'node:buffer';
import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('editor toolbar uses real icons and formats the selected text', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  const description = dialog.getByRole('textbox', { name: 'Description' });
  await description.fill('A clear note');
  await description.press('ControlOrMeta+a');
  await dialog.getByRole('button', { name: 'Bold' }).click();
  await expect(description.locator('strong')).toContainText('A clear note');
  for (const label of [
    'Bold',
    'Italic',
    'Heading',
    'Bullet list',
    'Numbered list',
    'Checklist',
    'Quote',
    'Code',
    'Link',
  ])
    await expect(
      dialog.getByRole('button', { name: label, exact: true }).locator('svg.tabler-icon-circle'),
    ).toHaveCount(0);
});

test('slash commands insert structured content without an editor drag handle', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Editor blocks');
  const summary = dialog.getByRole('textbox', { name: 'Project summary' });
  await summary.fill('Brief summary');
  await summary.press('ControlOrMeta+a');
  await summary.press('ControlOrMeta+b');
  const description = dialog.getByRole('textbox', { name: 'Project description' });
  await description.fill('/head');
  const menu = page.getByRole('listbox', { name: 'Insert block' });
  await expect(menu.getByRole('option', { name: /Heading 2/ })).toBeVisible();
  await description.press('ArrowDown');
  await description.press('Enter');
  await description.type('Section');
  await expect(description.locator('h3')).toContainText('Section');
  await expect(page.locator('.rich-text-block-handle')).toHaveCount(0);
  await description.press('ControlOrMeta+a');
  await expect(dialog.getByRole('button', { name: 'Add block' })).toBeVisible();
  await dialog.getByRole('button', { name: 'Add block' }).click();
  await expect(menu).toBeVisible();
  await menu.getByRole('option', { name: /Bulleted list/ }).click();
  await description.type('First item');
  await expect(description.locator('ul li')).toContainText('First item');
  await dialog.getByRole('button', { name: 'Create project' }).click();
  await expect(
    page.getByRole('region', { name: 'Project overview' }).locator('strong'),
  ).toContainText('Brief summary');
  await page.reload();
  await expect(
    page.getByRole('region', { name: 'Project overview' }).locator('strong'),
  ).toContainText('Brief summary');
});

test('issue description edits without a block drag handle', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create issue' }).click();
  const dialog = page.getByRole('dialog', { name: 'Create issue' });
  const description = dialog.getByRole('textbox', { name: 'Description' });
  await description.fill('First');
  await description.press('Enter');
  await description.type('Second');
  await expect(description.locator('p').first()).toContainText('First');
  await expect(description.locator('p').last()).toContainText('Second');
  await expect(page.locator('.rich-text-block-handle')).toHaveCount(0);

  await dialog.getByRole('textbox', { name: 'Title' }).fill('Structured issue');
  await dialog.getByRole('button', { name: 'Create issue' }).click();
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('issues') ?? '{}'));
  const issue = stored.items.find((item: { title: string }) => item.title === 'Structured issue');
  expect(issue.descriptionDocument.version).toBe(1);
  expect(issue.descriptionDocument.content.content).toHaveLength(2);
});

test('checklist blocks keep their checked state after saving a project', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();

  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Checklist project');

  const description = dialog.getByRole('textbox', { name: 'Project description' });
  await description.fill('/task');
  await page
    .getByRole('listbox', { name: 'Insert block' })
    .getByRole('option', { name: /Checklist/ })
    .click();
  await description.type('Review the scope');
  await expect(description.locator('li')).toContainText('Review the scope');

  await description.locator('input[type="checkbox"]').check();
  await dialog.getByRole('button', { name: 'Create project' }).click();

  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem('projects') ?? '{}'));
  const project = stored.items.find((item: { name: string }) => item.name === 'Checklist project');
  expect(project.descriptionDocument.version).toBe(1);
  expect(project.descriptionDocument.content.content[0].content[0].attrs.checked).toBe(true);

  const saved = page.getByRole('region', { name: 'Project description' });
  await expect(saved.locator('input[type="checkbox"]')).toBeChecked();
  await page.reload();
  await expect(saved.locator('input[type="checkbox"]')).toBeChecked();
});

test('editor links validate their scheme and survive a project reload', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();

  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Linked project');

  const description = dialog.getByRole('textbox', { name: 'Project description' });
  await description.fill('Design notes');
  await description.press('ControlOrMeta+a');
  await description.press('ControlOrMeta+k');

  const linkDialog = page.getByRole('dialog', { name: 'Add link' });
  await expect(linkDialog).toBeVisible();
  await linkDialog.getByRole('textbox', { name: 'Link URL' }).fill('javascript:alert(1)');
  await linkDialog.getByRole('button', { name: 'Save link' }).click();
  await expect(linkDialog.getByRole('alert')).toContainText('http or https');

  await linkDialog.getByRole('textbox', { name: 'Link URL' }).fill('https://example.com/design');
  await linkDialog.getByRole('button', { name: 'Save link' }).click();
  await expect(description.locator('a[href="https://example.com/design"]')).toContainText(
    'Design notes',
  );

  await description.locator('a').click();
  await description.press('ControlOrMeta+k');
  await linkDialog.getByRole('button', { name: 'Remove link' }).click();
  await expect(description.locator('a')).toHaveCount(0);

  await description.press('ControlOrMeta+a');
  await description.press('ControlOrMeta+k');
  await linkDialog.getByRole('textbox', { name: 'Link URL' }).fill('https://example.com/design');
  await linkDialog.getByRole('button', { name: 'Save link' }).click();

  await dialog.getByRole('button', { name: 'Create project' }).click();
  await expect(page.getByRole('region', { name: 'Project description' })).toBeVisible();
  await page.reload();
  await expect(
    page
      .getByRole('region', { name: 'Project description' })
      .locator('a[href="https://example.com/design"]'),
  ).toContainText('Design notes');
});

test('inline image survives project save and reload', async ({ page }) => {
  await page.goto('/#/issues');
  await page.getByRole('button', { name: 'Create project' }).click();

  const dialog = page.getByRole('dialog', { name: 'Create project' });
  await dialog.getByRole('textbox', { name: 'Project name' }).fill('Illustrated project');
  await dialog.getByRole('button', { name: 'Add image' }).click();
  await dialog.locator('input[type="file"]').setInputFiles({
    name: 'diagram.png',
    mimeType: 'image/png',
    buffer: Buffer.from(
      'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScL/nwAAAABJRU5ErkJggg==',
      'base64',
    ),
  });
  await expect(
    dialog.getByRole('textbox', { name: 'Project description' }).locator('img'),
  ).toHaveCount(1);
  await dialog.getByRole('button', { name: 'Create project' }).click();
  const image = page.getByRole('region', { name: 'Project description' }).locator('img');
  await expect(image).toHaveAttribute('alt', 'diagram');
  await page.reload();
  await expect(image).toHaveAttribute('src', /^data:image\/png;base64,/);
});
