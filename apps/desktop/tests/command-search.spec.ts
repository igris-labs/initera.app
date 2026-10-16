import { expect, test } from '@playwright/test';
import { findCommands } from '../src/lib/command-search';
import { searchableDescription } from '../src/lib/rich-text-search';
import type { Command, CommandSearchContext } from '../src/lib/command-search';

const commands: Command[] = [
  { id: 'create-issue', label: 'Create issue', section: 'Actions', icon: '', run: () => {} },
  { id: 'create-project', label: 'Create project', section: 'Actions', icon: '', run: () => {} },
  { id: 'project-status', label: 'Change status', section: 'Atlas', icon: '', run: () => {} },
  { id: 'issue-status', label: 'Change status', section: 'APP-1', icon: '', run: () => {} },
  { id: 'show-board', label: 'Show board', section: 'This view', icon: '', run: () => {} },
  { id: 'projects', label: 'Projects', section: 'Go to', icon: '', run: () => {} },
  {
    id: 'issue-result',
    label: 'Fix calendar',
    detail: 'APP-42',
    keywords: 'date picker',
    section: 'Issues',
    icon: '',
    run: () => {},
  },
];

const context: CommandSearchContext = {
  mode: 'all',
  routeName: 'issues',
  projectName: '',
  issueName: '',
};

test('command suggestions follow the current screen', () => {
  expect(findCommands(commands, '', context).map((item) => item.id)).toEqual([
    'create-issue',
    'show-board',
  ]);
  expect(
    findCommands(commands, '', { ...context, projectName: 'Atlas' }).map((item) => item.id),
  ).toEqual(['project-status']);
  expect(
    findCommands(commands, '', { ...context, issueName: 'APP-1' }).map((item) => item.id),
  ).toEqual(['create-issue', 'issue-status']);
});

test('search finds issue IDs and keywords across screens', () => {
  expect(findCommands(commands, 'app-42', context)[0]?.id).toBe('issue-result');
  expect(findCommands(commands, 'date picker', context)[0]?.id).toBe('issue-result');
});

test('structured description search reads visible text and ignores stale Markdown', () => {
  const description = searchableDescription(
    {
      version: 1,
      content: {
        type: 'doc',
        content: [
          { type: 'paragraph', content: [{ type: 'text', text: 'Review' }] },
          {
            type: 'taskList',
            content: [
              {
                type: 'taskItem',
                attrs: { checked: true },
                content: [{ type: 'paragraph', content: [{ type: 'text', text: 'launch' }] }],
              },
            ],
          },
        ],
      },
    },
    'stale draft',
  );

  expect(description).toBe('Review launch');
  expect(searchableDescription(null, 'Legacy **Markdown**')).toBe('Legacy **Markdown**');
});
