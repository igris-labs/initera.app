# Contributing

Thanks for working on Initera. Open an issue before a large change so maintainers can confirm its scope.

## Set up

Follow the [source setup](README.md#install-and-run-from-source). Run the checks in the README before opening a pull request. Add a focused test when a change affects persistence, data conversion, or a user workflow.

## Code

- Keep each Vue component responsible for one part of the interface. Move shared domain logic into a composable, store, or library module only when two callers need it.
- Put a UI component's visual styles beside that component. Use Akaza UI primitives when one exists.
- Put shared domain types in `apps/desktop/src/types`. Put a component's prop interface after its imports.
- Write comments for behavior a reader cannot infer from the code, especially recovery and transaction boundaries.
- Use Rust transactions for changes spanning records. Keep SQLite schema changes in `schema.sql` until the first stable release.
- Update user documentation when behavior or data handling changes.

Do not copy code or assets from `research-data/references` into the app without checking their license. The reference material exists for design and behavior study.

## Pull requests

Describe the behavior, the checks you ran, and any data migration risk. Keep unrelated cleanup out of a feature change.
