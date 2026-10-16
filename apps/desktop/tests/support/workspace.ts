import type { Page } from '@playwright/test';
import { previewIssues } from '../fixtures/preview-issues';
import { previewProjects } from '../fixtures/preview-projects';

export async function prepareWorkspace({ page }: { page: Page }) {
  await page.addInitScript(
    ({ issues, projects }) => {
      if (!localStorage.getItem('workspace'))
        localStorage.setItem(
          'workspace',
          JSON.stringify({ userName: 'Preview user', workspaceName: 'Personal workspace' }),
        );

      if (!localStorage.getItem('issues'))
        localStorage.setItem('issues', JSON.stringify({ items: issues }));
      if (!localStorage.getItem('projects'))
        localStorage.setItem('projects', JSON.stringify({ items: projects }));
    },
    { issues: previewIssues, projects: previewProjects },
  );
}
