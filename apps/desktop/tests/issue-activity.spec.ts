import { expect, test } from '@playwright/test';
import { prepareWorkspace } from './support/workspace';

test.beforeEach(prepareWorkspace);

test('issue comments and activity survive a preview restart', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const detail = page.getByRole('region', { name: 'Full issue view' });
  await expect(detail.getByText('created this issue')).toBeVisible();
  await detail.getByRole('textbox', { name: 'Write a comment' }).fill('**Ready** for review');
  await detail.getByRole('button', { name: 'Post comment' }).click();
  await expect(detail.getByLabel('Comment content').getByText('Ready')).toBeVisible();
  await page.reload();
  await expect(detail.getByLabel('Comment content').getByText('Ready')).toBeVisible();
});

test('an unfinished comment recovers after restart and clears after posting', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const detail = page.getByRole('region', { name: 'Full issue view' });
  const editor = detail.getByRole('textbox', { name: 'Write a comment' });

  await editor.fill('Recovered comment');
  await expect
    .poll(() =>
      page.evaluate(() =>
        Object.keys(localStorage)
          .filter((key) => key.startsWith('issue-comment:'))
          .map((key) => localStorage.getItem(key))
          .join(''),
      ),
    )
    .toContain('Recovered comment');

  await page.reload();
  await expect(editor).toContainText('Recovered comment');
  await detail.getByRole('button', { name: 'Post comment' }).click();
  await page.reload();

  await expect(detail.getByLabel('Comment content').getByText('Recovered comment')).toBeVisible();
  await expect(editor).toBeEmpty();
});

test('an image pasted into a comment survives a preview restart', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const detail = page.getByRole('region', { name: 'Full issue view' });
  const editor = detail.getByRole('textbox', { name: 'Write a comment' });

  await editor.evaluate((element) => {
    const bytes = Uint8Array.from(
      atob(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScL/nwAAAABJRU5ErkJggg==',
      ),
      (character) => character.charCodeAt(0),
    );
    const clipboard = new DataTransfer();
    clipboard.items.add(new File([bytes], 'comment.png', { type: 'image/png' }));
    element.dispatchEvent(
      new ClipboardEvent('paste', { bubbles: true, cancelable: true, clipboardData: clipboard }),
    );
  });

  await expect(editor.locator('img')).toHaveCount(1);
  await detail.getByRole('button', { name: 'Post comment' }).click();
  const image = detail.getByLabel('Comment content').locator('img');
  await expect(image).toHaveAttribute('alt', 'comment');
  await page.reload();
  await expect(image).toHaveAttribute('src', /^data:image\/png;base64,/);
});

test('comments can be edited and soft deleted', async ({ page }) => {
  await page.goto('/#/issues/APP-123/full');
  const detail = page.getByRole('region', { name: 'Full issue view' });

  await detail.getByRole('textbox', { name: 'Write a comment' }).fill('First draft');
  await detail.getByRole('button', { name: 'Post comment' }).click();
  await detail.getByRole('button', { name: 'Edit', exact: true }).click();
  await detail.getByRole('textbox', { name: 'Edit comment' }).fill('Revised comment');
  await detail.getByRole('button', { name: 'Save comment' }).click();
  await expect(detail.getByLabel('Comment content').getByText('Revised comment')).toBeVisible();

  await detail.getByRole('button', { name: 'Delete', exact: true }).click();
  await detail.getByRole('button', { name: 'Delete', exact: true }).click();
  await expect(detail.getByText('Comment deleted')).toBeVisible();
  await expect(detail.getByLabel('Comment content')).toHaveCount(0);
});
