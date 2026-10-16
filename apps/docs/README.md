# Initera documentation

This Nuxt site contains the Initera user guide. Rahul Vashishtha writes and maintains the project under Igris Labs, his personal laboratory.

## Run locally

From the repository root:

```sh
pnpm install --frozen-lockfile
pnpm --filter docs dev
```

Open the URL printed by Nuxt. User guides live in numbered directories under `apps/docs/content/`. Add a Markdown file to a section and Nuxt Content includes it in the sidebar and search automatically. Its title comes from the Markdown heading or `title` frontmatter, and its URL is generated from its location. Prefix filenames with a number to set their order, such as `3.issue-properties.md`. The optional `icon` field controls the sidebar icon; pages without it use the default icon.

The Nuxt app lives under `app/`. The collection source and frontmatter schema live in `content.config.ts`. `app/composables/useDocsNavigation.ts` loads Nuxt Content navigation for the sidebar, search, breadcrumbs, and adjacent-page links. Keep guide text in Markdown rather than in Vue components.

## Refresh screenshots

Screenshots come from the running desktop browser preview with test fixtures. They are stored in `apps/docs/public/images/app`. From the repository root, run:

```sh
CAPTURE_DOCS=1 pnpm --filter initera exec playwright test tests/docs-screenshots.spec.ts
```

The capture test starts the Vite preview through Playwright and uses a fresh browser profile. It does not read a personal workspace.

## Check the site

```sh
pnpm --filter docs exec nuxt typecheck
pnpm --filter docs build
```
