import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import ts from 'typescript';

type ComponentSize = 'sm' | 'md' | 'lg';

async function componentStyles(page: Page, name: string, size?: ComponentSize) {
  return page.evaluate(
    async ({ name, size }) => {
      const component = await import(/* @vite-ignore */ `/src/components/ui/${name}/index.ts`);
      const functionName = `${name.replace(/-([a-z])/g, (_, letter: string) => letter.toUpperCase())}Ui`;
      return component[functionName](size) as Record<string, string>;
    },
    { name, size },
  );
}

test('application icon names are registered in the shared icon component', () => {
  const root = path.join(process.cwd(), 'src');
  const iconSource = fs.readFileSync(path.join(root, 'components/ui/icon/Icon.vue'), 'utf8');
  const registered = new Set(
    [...iconSource.matchAll(/'i-tabler-([^']+)':/g)].map((match) => match[1]),
  );
  const missing: string[] = [];

  function inspect(directory: string) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) inspect(file);
      else if (
        /\.(?:vue|ts)$/.test(file) &&
        file !== path.join(root, 'components/ui/icon/Icon.vue')
      ) {
        const source = fs.readFileSync(file, 'utf8');
        for (const match of source.matchAll(/i-tabler-([a-z0-9-]+)/g)) {
          if (!registered.has(match[1])) missing.push(`${path.relative(root, file)}: ${match[0]}`);
        }
      }
    }
  }

  inspect(root);
  expect(missing).toEqual([]);
});

test('component recipes use supported Akaza UI slots', () => {
  const root = process.cwd();
  const typeRoot = path.join(root, 'node_modules/akaza-ui/dist/types/components');
  const uiRoot = path.join(root, 'src/components/ui');
  const periodTypes = new Set([
    'month-picker',
    'month-range-picker',
    'year-picker',
    'year-range-picker',
  ]);
  const menuTypes = new Set(['context-menu', 'menubar']);
  const slots = (name: string) => {
    const file = path.join(
      typeRoot,
      periodTypes.has(name) ? 'date-period/PeriodPicker.vue.d.ts' : `${name}/index.d.ts`,
    );
    const types = ts.createSourceFile(
      file,
      fs.readFileSync(file, 'utf8'),
      ts.ScriptTarget.Latest,
      true,
    );
    const supported = new Set<string>();
    for (const item of types.statements) {
      if (!ts.isInterfaceDeclaration(item) || !item.name.text.endsWith('Ui')) continue;
      for (const member of item.members) if (member.name) supported.add(member.name.getText(types));
    }

    return supported;
  };

  const unsupported: string[] = [];
  for (const entry of fs.readdirSync(uiRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const index = path.join(uiRoot, entry.name, 'index.ts');
    if (!fs.existsSync(index)) continue;
    const source = ts.createSourceFile(
      index,
      fs.readFileSync(index, 'utf8'),
      ts.ScriptTarget.Latest,
      true,
    );
    const recipe = source.statements
      .filter(ts.isVariableStatement)
      .flatMap((statement) => [...statement.declarationList.declarations])
      .find((item) => item.name.getText(source) === 'uiParts');
    if (!recipe || !ts.isObjectLiteralExpression(recipe.initializer)) continue;
    const name = entry.name;
    const allowed = slots(name);
    if (menuTypes.has(name)) for (const slot of slots('menu')) allowed.add(slot);
    for (const part of recipe.initializer.properties) {
      if (!allowed.has(part.name?.getText(source) ?? ''))
        unsupported.push(`${name}.${part.name?.getText(source)}`);
    }
  }

  expect(unsupported).toEqual([]);
});

test('Akaza wrappers forward slots and models', async ({ page }) => {
  await page.goto('/tests/fixtures/ui-wrappers.html');
  await page.getByRole('button', { name: 'Details' }).click();
  await expect(page.getByText('Accordion content')).toBeVisible();
  await page.getByRole('tab', { name: 'Two' }).click();
  await expect(page.getByText('Second panel')).toBeVisible();
  await page.getByRole('button', { name: 'Pin item' }).click();
  await expect(page.getByRole('button', { name: 'Pin item' })).toHaveText('Pinned');
});

test('component sizes scale text and spacing together', async ({ page }) => {
  await page.goto('/tests/fixtures/ui-wrappers.html');
  const classes = await Promise.all(
    (['sm', 'md', 'lg'] as const).map(async (size) => ({
      tab: (await componentStyles(page, 'tabs', size)).tab,
      tableCell: (await componentStyles(page, 'data-table', size)).cell,
      combobox: (await componentStyles(page, 'combobox', size)).input,
      checkboxLabel: (await componentStyles(page, 'checkbox', size)).label,
      avatarInitials: (await componentStyles(page, 'avatar', size)).fallback,
      toolbarButton: (await componentStyles(page, 'toolbar', size)).button,
    })),
  );
  expect(classes[0]).toMatchObject({
    tab: expect.stringContaining('min-h-7'),
    tableCell: expect.stringContaining('px-2 py-1.5'),
    combobox: expect.stringContaining('text-xs'),
    checkboxLabel: expect.stringContaining('text-xs'),
    avatarInitials: expect.stringContaining('text-xs'),
    toolbarButton: expect.stringContaining('min-h-7'),
  });
  expect(classes[1]).toMatchObject({
    tab: expect.stringContaining('min-h-8'),
    tableCell: expect.stringContaining('px-3 py-2'),
    combobox: expect.stringContaining('text-sm'),
    checkboxLabel: expect.stringContaining('text-sm'),
    avatarInitials: expect.stringContaining('text-sm'),
    toolbarButton: expect.stringContaining('min-h-8'),
  });
  expect(classes[2]).toMatchObject({
    tab: expect.stringContaining('min-h-9'),
    tableCell: expect.stringContaining('px-3.5 py-2.5'),
    combobox: expect.stringContaining('text-base'),
    checkboxLabel: expect.stringContaining('text-base'),
    avatarInitials: expect.stringContaining('text-base'),
    toolbarButton: expect.stringContaining('min-h-9'),
  });
});

test('menu primitives share surface, item, and submenu treatment', async ({ page }) => {
  await page.goto('/tests/fixtures/ui-wrappers.html');
  const styles = {
    context: await componentStyles(page, 'context-menu', 'md'),
    menu: await componentStyles(page, 'menu', 'md'),
    menubar: await componentStyles(page, 'menubar', 'md'),
    listbox: await componentStyles(page, 'listbox', 'md'),
    combobox: await componentStyles(page, 'combobox', 'md'),
  };
  for (const name of ['context', 'menu', 'menubar'] as const) {
    expect(styles[name].content).toContain('p-1.5');
    expect(styles[name].item).toContain('min-h-8');
    expect(styles[name].item).toContain('text-sm');
    expect(styles[name].submenuContent).toContain('border-outline');
    expect(styles[name].separator).toContain('border-outline');
  }

  expect(styles.menubar.trigger).toContain('min-h-8');
  for (const name of ['listbox', 'combobox'] as const) {
    expect(styles[name].content).toContain('p-1.5');
    expect(styles[name].option).toContain('min-h-8');
  }
});

test('unconfigured menu and button use their small fallbacks', async ({ page }) => {
  await page.goto('/tests/fixtures/ui-wrappers.html');
  const styles = {
    menu: await componentStyles(page, 'menu'),
    contextMenu: await componentStyles(page, 'context-menu'),
    button: await page.evaluate(async () => {
      const { buttonVariants } = await import('/src/components/ui/button/index.ts');
      return buttonVariants({});
    }),
  };
  for (const name of ['menu', 'contextMenu'] as const) {
    expect(styles[name].content).toContain('min-w-36');
    expect(styles[name].content).toContain('p-1');
    expect(styles[name].item).toContain('min-h-7');
    expect(styles[name].item).toContain('text-xs');
  }

  expect(styles.button).toContain('min-h-7');
  expect(styles.button).toContain('text-xs');
});

test('component props override app defaults, which override local fallbacks', async ({ page }) => {
  await page.goto('/tests/fixtures/ui-wrappers.html');

  const styles = await page.evaluate(async () => {
    const { default: config } = await import('/src/components/ui/ui.config.ts');
    config.defaults = { size: 'lg', variant: 'outline' };
    config.components = { button: { size: 'xs' } };

    const { buttonVariants } = await import('/src/components/ui/button/index.ts?config-test=1');

    return {
      configured: buttonVariants({}),
      explicit: buttonVariants({ size: 'md', variant: 'solid' }),
    };
  });

  expect(styles.configured).toContain('min-h-6');
  expect(styles.configured).toContain('border-outline');
  expect(styles.explicit).toContain('min-h-8');
  expect(styles.explicit).toContain('ipex-execution');
});

test('date range trigger and month/year grids use their actual Akaza slots', async ({ page }) => {
  await page.goto('/tests/fixtures/ui-wrappers.html');
  const pickerStyles = {
    date: [
      (await componentStyles(page, 'date-range-picker', 'sm')).trigger,
      (await componentStyles(page, 'date-range-picker', 'md')).trigger,
    ],
    periods: await Promise.all(
      (['month-range-picker', 'year-range-picker'] as const).map(async (name) => [
        await componentStyles(page, name, 'sm'),
        await componentStyles(page, name, 'md'),
      ]),
    ),
  };
  expect(pickerStyles.date[0]).toContain('min-h-7');
  expect(pickerStyles.date[1]).toContain('min-h-8');
  for (const [small, medium] of pickerStyles.periods) {
    expect(small.root).toContain('rounded-lg');
    expect(small.cellTrigger).toContain('size-7');
    expect(small.previous).toContain('size-7');
    expect(medium.cellTrigger).toContain('size-8');
    expect(medium.next).toContain('size-8');
    expect(small).not.toHaveProperty('trigger');
  }
});
