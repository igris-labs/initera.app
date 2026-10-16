import type { AppShortcut } from './app-shortcuts';

export interface Command {
  id: string;
  label: string;
  icon: string;
  section: string;
  shortcut?: AppShortcut;
  detail?: string;
  keywords?: string;
  run: () => void | Promise<unknown>;
}

export interface CommandSearchContext {
  mode: 'all' | 'issues' | 'projects' | 'workspaces' | 'shortcuts';
  routeName: string;
  projectName: string;
  issueName: string;
}

function searchRank(command: Command, search: string): number {
  const label = command.label.toLowerCase();
  const detail = command.detail?.toLowerCase();

  if (label === search || detail === search) return 0;
  if (label.startsWith(search)) return 1;
  if (label.includes(search)) return 2;
  if (detail?.includes(search)) return 3;

  return 4;
}

function suggestedCommands(commands: Command[], context: CommandSearchContext): Command[] {
  // Empty query suggests actions for this screen; typed search covers the full command catalog.
  if (context.mode !== 'all') return commands;

  if (context.projectName) {
    return commands.filter((command) => command.section === context.projectName);
  }

  if (context.issueName) {
    return commands.filter(
      (command) => command.section === context.issueName || command.id === 'create-issue',
    );
  }

  if (context.routeName === 'projects') {
    return commands.filter(
      (command) => command.id === 'create-project' || command.section === 'Projects',
    );
  }

  if (context.routeName === 'cycles') {
    return commands.filter(
      (command) => command.id === 'create-cycle' || command.section === 'Cycles',
    );
  }

  if (context.routeName === 'views' || context.routeName === 'saved-view') {
    return commands.filter(
      (command) => command.id === 'create-view' || command.section === 'Views',
    );
  }

  if (['issues', 'triage', 'project-issues'].includes(context.routeName)) {
    return commands.filter(
      (command) => command.section === 'This view' || command.id === 'create-issue',
    );
  }

  return commands.filter((command) => command.section === 'Go to' || command.id === 'create-issue');
}

export function findCommands(
  commands: Command[],
  query: string,
  context: CommandSearchContext,
): Command[] {
  const search = query.trim().toLowerCase();

  if (!search) return suggestedCommands(commands, context).slice(0, 40);

  return commands
    .filter((command) =>
      `${command.label} ${command.detail ?? ''} ${command.keywords ?? ''}`
        .toLowerCase()
        .includes(search),
    )
    .sort((left, right) => searchRank(left, search) - searchRank(right, search))
    .slice(0, 40);
}
