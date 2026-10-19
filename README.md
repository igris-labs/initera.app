<img src="brand/initera-app-icon.svg" alt="Initera icon" width="64" />

# Initera

Initera is an open-source desktop app for planning projects and managing issues. It keeps project plans, individual tasks, cycles, and time records in one workspace. Your work stays on your computer unless you export it.

## What you can do

- Track issues with priorities, labels, sub-issues, and relations.
- Plan projects with milestones, resources, and updates.
- Organize upcoming work in cycles and saved views.
- Find work through search, the command bar, and keyboard shortcuts.
- Track time and export backups that include uploaded images.

[Download the latest release](https://github.com/igris-labs/initera.app/releases/latest) or [read the user guide](https://initera.app/getting-started/quick-start).

## Install and run from source

Install Node 24, pnpm 10, Rust, and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/). Then run:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

The desktop app lives in `apps/desktop`. `pnpm dev` starts a browser preview backed by local storage; use the desktop app for SQLite persistence. Browser tests seed their own fixtures.

The website and user guide live in `apps/docs`. Run them with `pnpm --filter docs dev`.

## Checks

```sh
pnpm typecheck
pnpm lint
pnpm format:check
pnpm test:ui
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
```

Install Chromium with `pnpm --filter initera exec playwright install chromium` before running browser tests.

## Documentation

- [Website and user guide](apps/docs/README.md)
- [Data and backups](docs/data-and-backups.md)
- [Contributing](CONTRIBUTING.md)
- [Architecture](docs/architecture.md)
- [Release process](docs/releasing.md)

The schema in `apps/desktop/src-tauri/src/schema.sql` is version 1. This baseline supports clean installs. Databases made by development builds are outside its upgrade path; export the data you need before installing a release.

## License

[MIT](LICENSE)

Created by [Rahul Vashishtha](https://rahulv.dev) at [Igris Labs](https://igrislabs.com).
