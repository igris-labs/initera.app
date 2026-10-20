# Initera roadmap

Initera is a local-first desktop app for planning projects and managing issues. The first table lists features in the current app. The second lists planned work, in implementation order.

| Feature                   | Description                                                                      | Status       |
| ------------------------- | -------------------------------------------------------------------------------- | ------------ |
| Workspaces                | Create and switch between separate local workspaces.                             | ✅ Available |
| Projects                  | Plan work with milestones, resources, updates, and issue assignments.            | ✅ Available |
| Issues and workflow       | Track issues, sub-issues, relations, statuses, priorities, labels, and activity. | ✅ Available |
| List and board views      | Review and move issues in either layout.                                         | ✅ Available |
| Cycles                    | Plan work in manual cycles or recurring series.                                  | ✅ Available |
| Time tracking             | Run timers, add manual entries, and review time by issue.                        | ✅ Available |
| Saved views and search    | Save filtered views and find issues or projects.                                 | ✅ Available |
| Command bar and shortcuts | Search, navigate, and run actions from the keyboard.                             | ✅ Available |
| Rich descriptions         | Write formatted content and attach managed images.                               | ✅ Available |
| Backups and restore       | Export and restore workspace data with its images.                               | ✅ Available |
| Automatic updates         | Check GitHub releases and install an update from the app.                        | ✅ Available |
| Desktop installers        | Build macOS, Windows, and Linux packages.                                        | ✅ Available |

## Planned: self-hosted collaboration

Initera will keep its account-free local mode. A user who wants to collaborate can connect the desktop app to a self-hosted Initera server, sign in, and link or upload a local workspace. The app will show the transfer and report conflicts during the first connection; later changes will sync automatically.

The server will use Go and PostgreSQL. It will provide one documented REST API with an OpenAPI specification. Self-hosters will be able to run a binary or a Docker image and configure the server through a configuration file and environment variables. A single-server installation will need PostgreSQL and a writable data directory. It will store attachments on local disk unless the operator configures an S3-compatible bucket. The server will handle sessions and background work without requiring Redis or a separate message broker; larger deployments can add shared infrastructure when needed.

| Order | Feature                        | Scope                                                                                                                                                                                                                            | Status     |
| ----- | ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| 1     | Server and deployment          | A PostgreSQL connection URL, versioned migrations, health checks, a public server URL, configuration validation, a Go binary, Docker deployment, and backup and restore instructions.                                            | 📋 Planned |
| 2     | Accounts and sign-in           | Email and password, emailed one-time codes, and configurable OpenID Connect sign-in with Google and Microsoft presets. Keep sessions and recovery flows on the server.                                                           | 📋 Planned |
| 3     | Workspace access               | A workspace owns its teams, projects, issues, settings, and integrations. Admins invite people; owner, admin, member, and restricted guest permissions control access.                                                           | 📋 Planned |
| 4     | Teams                          | Team membership, issue ownership and identifiers, team workflows, assignees, and projects that can include more than one team.                                                                                                   | 📋 Planned |
| 5     | REST API                       | Cover server-managed features through authenticated endpoints. Publish OpenAPI documentation and use the same permission rules for desktop clients and API callers.                                                              | 📋 Planned |
| 6     | Desktop sync                   | Connect by server URL, sign in, transfer a local workspace and its files, then sync changes across devices. Handle retries, conflicts, deletions, offline edits, and reconnects.                                                 | 📋 Planned |
| 7     | Live collaboration             | Deliver change notifications so connected clients refresh shared work, membership changes, and permissions without waiting for a manual reload.                                                                                  | 📋 Planned |
| 8     | Configurable storage and email | Store structured records in PostgreSQL. Use local disk for images and attachments by default, with an optional S3-compatible bucket. Configure an SMTP service before enabling emailed codes, invitations, or password recovery. | 📋 Planned |
| 9     | Personal work                  | Add assigned work, mentions, subscriptions, and an inbox with per-user notification state.                                                                                                                                       | 📋 Planned |
| 10    | Source-control links           | Connect GitHub and GitLab repositories; link commits and pull or merge requests to issues; configure workflow changes from supported events.                                                                                     | 📋 Planned |
| 11    | Customers and feedback         | Customer records grant no access. Give customer guests explicit, restricted permissions. Record requests with their source, link several requests to one issue or project, and track importance per customer.                    | 📋 Planned |
| 12    | External intake                | Let configured channels create or link requests after the manual customer workflow is in place. Preserve source links and avoid duplicate work on webhook retries.                                                               | 📋 Planned |

The server will check workspace and team permissions for reads, writes, search, files, and notifications. It will protect credentials, validate provider callbacks, rate-limit sign-in attempts, and keep an audit trail for access and administrative changes. Self-hosted installations will use the operator's PostgreSQL, storage, identity, and email configuration. These security controls require testing before a server release.

“Available” means the feature is in the desktop app. “Planned” work has no release date.

For current behavior, read the [user guide](https://initera.app/getting-started/quick-start). To report a bug or discuss a proposed feature, open a [GitHub issue](https://github.com/igris-labs/initera.app/issues). Contributors can start with [CONTRIBUTING.md](CONTRIBUTING.md).
