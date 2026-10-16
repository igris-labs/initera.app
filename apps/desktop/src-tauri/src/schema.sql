-- Fresh-install schema for V1. Existing development databases use a different version.
CREATE TABLE local_actor (
               singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
               id TEXT NOT NULL UNIQUE,
               workspace_id TEXT NOT NULL REFERENCES workspace(id),
               display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 80)
             );

CREATE TABLE project (
                   id TEXT PRIMARY KEY,
                   workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                   name TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(name)) BETWEEN 1 AND 80),
                   identifier_prefix TEXT NOT NULL COLLATE NOCASE
                     CHECK (length(identifier_prefix) BETWEEN 2 AND 8),
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')), summary TEXT NOT NULL DEFAULT '', description TEXT NOT NULL DEFAULT '', status TEXT NOT NULL DEFAULT 'Backlog', priority TEXT NOT NULL DEFAULT 'No priority', start_date TEXT, target_date TEXT, name_markdown TEXT NOT NULL DEFAULT '', labels_json TEXT NOT NULL DEFAULT '[]', resources_json TEXT NOT NULL DEFAULT '[]', icon TEXT NOT NULL DEFAULT 'i-tabler-cube', color TEXT NOT NULL DEFAULT 'amber', description_document_json TEXT
                 CHECK (description_document_json IS NULL OR json_valid(description_document_json)), lead_actor_id TEXT, archived_at TEXT, deleted_at TEXT, version INTEGER NOT NULL DEFAULT 1, cascade_id TEXT, updated_at TEXT,
                   UNIQUE (workspace_id, name),
                   UNIQUE (workspace_id, identifier_prefix)
                 );

CREATE INDEX project_workspace_name ON project(workspace_id, name);

CREATE TABLE workspace_preferences (
                   workspace_id TEXT PRIMARY KEY REFERENCES workspace(id) ON DELETE CASCADE,
                   theme TEXT NOT NULL CHECK (theme IN ('system', 'light', 'dark')),
                   issue_scope TEXT NOT NULL CHECK (issue_scope IN ('active', 'all')),
                   issue_layout TEXT NOT NULL CHECK (issue_layout IN ('list', 'board')),
                   board_group_by TEXT NOT NULL CHECK (board_group_by IN ('status', 'priority')),
                   group_issues_by_status INTEGER NOT NULL CHECK (group_issues_by_status IN (0, 1)),
                   show_issue_priority INTEGER NOT NULL CHECK (show_issue_priority IN (0, 1)),
                   show_issue_project INTEGER NOT NULL CHECK (show_issue_project IN (0, 1))
                 , estimate_scale TEXT NOT NULL DEFAULT 'fibonacci' CHECK (estimate_scale IN ('fibonacci', 'linear', 'exponential', 'tshirt', 'time')), timer_on_exit TEXT NOT NULL
             DEFAULT 'pause' CHECK (timer_on_exit IN ('pause', 'continue')), concurrent_timers INTEGER NOT NULL DEFAULT 0 CHECK (concurrent_timers IN (0, 1)), timer_on_sleep TEXT NOT NULL DEFAULT 'pause' CHECK (timer_on_sleep IN ('pause', 'include')));

CREATE TABLE issue_sequence (
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               prefix TEXT NOT NULL,
               next_number INTEGER NOT NULL CHECK (next_number > 0),
               PRIMARY KEY (workspace_id, prefix)
             );

CREATE TABLE milestone (
               id TEXT PRIMARY KEY,
               project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
               title TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 240),
               description TEXT NOT NULL DEFAULT '',
               target_date TEXT,
               sort_order INTEGER NOT NULL,
               created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             , deleted_at TEXT, archived_at TEXT);

CREATE INDEX milestone_project_order ON milestone(project_id, sort_order, id);

CREATE TABLE project_update (
               id TEXT PRIMARY KEY,
               project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
               actor_name TEXT NOT NULL,
               body TEXT NOT NULL CHECK(length(trim(body)) BETWEEN 1 AND 100000),
               created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             , kind TEXT NOT NULL DEFAULT 'update'
             CHECK (kind IN ('update', 'change')));

CREATE INDEX project_update_project_date ON project_update(project_id, created_at DESC, id DESC);

CREATE TABLE "workspace" (
                   id TEXT PRIMARY KEY,
                   name TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) BETWEEN 1 AND 80)
                 , revision INTEGER NOT NULL DEFAULT 0);

CREATE TABLE "issue" (
                   id TEXT PRIMARY KEY,
                   workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                   identifier TEXT NOT NULL,
                   project_id TEXT REFERENCES project(id) ON DELETE RESTRICT,
                   title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 255),
                   description TEXT NOT NULL DEFAULT '',
                   status TEXT NOT NULL,
                   priority TEXT NOT NULL CHECK (priority IN ('No priority','Low','Medium','High','Urgent')),
                   labels_json TEXT NOT NULL DEFAULT '[]',
                   due_date TEXT,
                   triage INTEGER NOT NULL DEFAULT 0 CHECK (triage IN (0, 1)),
                   position INTEGER NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                   updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                   title_markdown TEXT NOT NULL DEFAULT '',
                   parent_id TEXT REFERENCES issue(id) ON DELETE SET NULL,
                   start_date TEXT, archived_at TEXT, archive_root_id TEXT, estimate INTEGER CHECK (estimate IN (1, 2, 3, 5, 8, 13)), description_document_json TEXT
                 CHECK (description_document_json IS NULL OR json_valid(description_document_json)), estimate_value INTEGER CHECK (estimate_value > 0 AND estimate_value <= 1000000), triage_entered_at TEXT, triaged_at TEXT, milestone_id TEXT REFERENCES milestone(id) ON DELETE SET NULL, version INTEGER NOT NULL DEFAULT 1, deleted_at TEXT,
                   UNIQUE (workspace_id, identifier)
                 );

CREATE INDEX issue_workspace_position ON issue(workspace_id, position, id);

CREATE INDEX issue_project ON issue(project_id);

CREATE INDEX issue_parent ON issue(parent_id);

CREATE INDEX issue_workspace_archive ON issue(workspace_id, archived_at, id);

CREATE TABLE issue_activity (
                   id TEXT PRIMARY KEY,
                   issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
                   actor_name TEXT NOT NULL,
                   kind TEXT NOT NULL CHECK (kind IN ('created', 'changed', 'comment', 'archived', 'restored')),
                   body TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                 , body_document_json TEXT, edited_at TEXT, deleted_at TEXT, version INTEGER NOT NULL DEFAULT 1, changes_json TEXT, mutation_id TEXT);

CREATE INDEX issue_activity_issue_date ON issue_activity(issue_id, created_at DESC, id DESC);

CREATE TABLE issue_relation (
                   id TEXT PRIMARY KEY,
                   workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                   source_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
                   target_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
                   kind TEXT NOT NULL CHECK (kind IN ('blocks', 'related', 'duplicates')),
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                   CHECK (source_id <> target_id),
                   UNIQUE(workspace_id, source_id, target_id, kind)
                 );

CREATE INDEX issue_relation_source ON issue_relation(workspace_id, source_id);

CREATE INDEX issue_relation_target ON issue_relation(workspace_id, target_id);

CREATE TABLE workflow_status (
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               name TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(name)) BETWEEN 1 AND 32),
               category TEXT NOT NULL CHECK (category IN ('Backlog','Unstarted','Started','Completed','Canceled')),
               description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 500),
               icon TEXT NOT NULL,
               color TEXT NOT NULL CHECK (color IN ('gray','blue','purple','green','amber','red')),
               position INTEGER NOT NULL,
               archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0,1)),
               is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0,1)),
               PRIMARY KEY (workspace_id, name)
             );

CREATE UNIQUE INDEX workflow_default ON workflow_status(workspace_id) WHERE is_default = 1;

CREATE TRIGGER seed_workspace_workflow AFTER INSERT ON workspace BEGIN
               INSERT INTO workflow_status (workspace_id,name,category,icon,color,position,is_default)
               VALUES
                 (NEW.id,'Backlog','Backlog','i-tabler-circle-dashed','gray',0,1),
                 (NEW.id,'Todo','Unstarted','i-tabler-circle','gray',1,0),
                 (NEW.id,'In Progress','Started','i-tabler-circle-dot','amber',2,0),
                 (NEW.id,'On Hold','Started','i-tabler-player-pause','purple',3,0),
                 (NEW.id,'Done','Completed','i-tabler-circle-check-filled','blue',4,0),
                 (NEW.id,'Canceled','Canceled','i-tabler-circle-x','gray',5,0);
             END;

CREATE TABLE saved_view (
                   id TEXT PRIMARY KEY,
                   workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                   definition_json TEXT NOT NULL CHECK (json_valid(definition_json)),
                   position INTEGER NOT NULL
                 , version INTEGER NOT NULL DEFAULT 1);

CREATE INDEX saved_view_workspace_order ON saved_view(workspace_id, position, id);

CREATE VIRTUAL TABLE issue_search USING fts5(
                   issue_id UNINDEXED, workspace_id UNINDEXED,
                   identifier, title, description, labels, project
                 );

CREATE TRIGGER issue_search_delete AFTER DELETE ON issue BEGIN
                   DELETE FROM issue_search WHERE rowid = OLD.rowid;
                 END;

CREATE TRIGGER issue_search_project_rename
                 AFTER UPDATE OF name ON project BEGIN
                   UPDATE issue_search SET project = NEW.name
                   WHERE issue_id IN (SELECT id FROM issue WHERE project_id = NEW.id);
                 END;

CREATE TRIGGER issue_search_insert AFTER INSERT ON issue
             WHEN NEW.archived_at IS NULL BEGIN
               INSERT INTO issue_search(rowid, issue_id, workspace_id, identifier, title, description, labels, project)
                 VALUES (NEW.rowid, NEW.id, NEW.workspace_id, NEW.identifier, NEW.title,
                         CASE WHEN NEW.description_document_json IS NULL THEN NEW.description
              ELSE COALESCE((SELECT group_concat(value, ' ') FROM (
                SELECT value FROM json_tree(NEW.description_document_json, '$.content')
                WHERE key = 'text' AND type = 'text'
                  AND fullkey NOT LIKE '%.attrs%'
                  AND fullkey NOT LIKE '%.marks%'
                ORDER BY id
              )), '') END, NEW.labels_json,
                         COALESCE((SELECT name FROM project WHERE id = NEW.project_id), 'Unassigned'));
             END;

CREATE TRIGGER issue_search_update
             AFTER UPDATE OF identifier, title, description, description_document_json,
                             labels_json, project_id, archived_at ON issue BEGIN
               DELETE FROM issue_search WHERE rowid = OLD.rowid;
               INSERT INTO issue_search(rowid, issue_id, workspace_id, identifier, title, description, labels, project)
                 SELECT NEW.rowid, NEW.id, NEW.workspace_id, NEW.identifier, NEW.title,
                        CASE WHEN NEW.description_document_json IS NULL THEN NEW.description
              ELSE COALESCE((SELECT group_concat(value, ' ') FROM (
                SELECT value FROM json_tree(NEW.description_document_json, '$.content')
                WHERE key = 'text' AND type = 'text'
                  AND fullkey NOT LIKE '%.attrs%'
                  AND fullkey NOT LIKE '%.marks%'
                ORDER BY id
              )), '') END, NEW.labels_json,
                        COALESCE((SELECT name FROM project WHERE id = NEW.project_id), 'Unassigned')
                 WHERE NEW.archived_at IS NULL;
             END;

CREATE TABLE cycle (
               id TEXT PRIMARY KEY,
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 80),
               start_date TEXT NOT NULL,
               end_date TEXT NOT NULL,
               status TEXT NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'active', 'completed')),
               created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             , schedule_index INTEGER, description TEXT NOT NULL DEFAULT '', completed_count INTEGER, total_count INTEGER, completed_at TEXT, project_scope_json TEXT NOT NULL DEFAULT '[]', rollover_policy TEXT NOT NULL DEFAULT 'ask' CHECK (rollover_policy IN ('ask', 'automatic', 'leave')), time_zone TEXT NOT NULL DEFAULT 'UTC', start_local_time TEXT NOT NULL DEFAULT '00:00', end_local_time TEXT NOT NULL DEFAULT '00:00', start_at TEXT, end_at TEXT, series_id TEXT REFERENCES cycle_series(id) ON DELETE SET NULL, successor_id TEXT REFERENCES cycle(id) ON DELETE SET NULL);

CREATE INDEX cycle_workspace_dates ON cycle(workspace_id, start_date, end_date);

CREATE TABLE cycle_issue (
               cycle_id TEXT NOT NULL REFERENCES cycle(id) ON DELETE CASCADE,
               issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
               PRIMARY KEY (cycle_id, issue_id)
             );

CREATE TABLE cycle_settings (
               workspace_id TEXT PRIMARY KEY REFERENCES workspace(id) ON DELETE CASCADE,
               duration_weeks INTEGER NOT NULL CHECK (duration_weeks BETWEEN 1 AND 8),
               cooldown_days INTEGER NOT NULL CHECK (cooldown_days BETWEEN 0 AND 7),
               upcoming_count INTEGER NOT NULL CHECK (upcoming_count BETWEEN 1 AND 15),
               start_date TEXT NOT NULL,
               enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1))
             , rollover_policy TEXT NOT NULL DEFAULT 'automatic' CHECK (rollover_policy IN ('automatic', 'leave', 'ask')), project_scope_json TEXT NOT NULL DEFAULT '[]', time_zone TEXT NOT NULL DEFAULT 'UTC', start_local_time TEXT NOT NULL DEFAULT '00:00');

CREATE TABLE time_entry (
               id TEXT PRIMARY KEY,
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
               started_at INTEGER NOT NULL,
               stopped_at INTEGER,
               duration_seconds INTEGER NOT NULL DEFAULT 0 CHECK (duration_seconds >= 0),
               note TEXT NOT NULL DEFAULT '' CHECK (length(note) <= 500)
             , last_seen_at INTEGER, version INTEGER NOT NULL DEFAULT 1, deleted_at TEXT, time_zone TEXT, exit_policy TEXT NOT NULL DEFAULT 'pause', sleep_policy TEXT NOT NULL DEFAULT 'pause', source TEXT NOT NULL DEFAULT 'timer' CHECK (source IN ('timer', 'manual')), started_at_ms INTEGER NOT NULL DEFAULT 0, stopped_at_ms INTEGER, last_seen_at_ms INTEGER, duration_ms INTEGER NOT NULL DEFAULT 0 CHECK (duration_ms >= 0));

CREATE INDEX time_entry_issue ON time_entry(issue_id, started_at DESC);

CREATE VIRTUAL TABLE project_search USING fts5(
               project_id UNINDEXED, workspace_id UNINDEXED,
               name, summary, description, labels, resources
             );

CREATE TRIGGER project_search_insert AFTER INSERT ON project BEGIN
               INSERT INTO project_search(rowid, project_id, workspace_id, name, summary, description, labels, resources)
                 VALUES (NEW.rowid, NEW.id, NEW.workspace_id, NEW.name, NEW.summary,
                         CASE WHEN NEW.description_document_json IS NULL THEN NEW.description
              ELSE COALESCE((SELECT group_concat(value, ' ') FROM (
                SELECT value FROM json_tree(NEW.description_document_json, '$.content')
                WHERE key = 'text' AND type = 'text'
                  AND fullkey NOT LIKE '%.attrs%'
                  AND fullkey NOT LIKE '%.marks%'
                ORDER BY id
              )), '') END, NEW.labels_json, NEW.resources_json);
             END;

CREATE TRIGGER project_search_update
             AFTER UPDATE OF name, summary, description, description_document_json,
                             labels_json, resources_json ON project BEGIN
               DELETE FROM project_search WHERE rowid = OLD.rowid;
               INSERT INTO project_search(rowid, project_id, workspace_id, name, summary, description, labels, resources)
                 VALUES (NEW.rowid, NEW.id, NEW.workspace_id, NEW.name, NEW.summary,
                         CASE WHEN NEW.description_document_json IS NULL THEN NEW.description
              ELSE COALESCE((SELECT group_concat(value, ' ') FROM (
                SELECT value FROM json_tree(NEW.description_document_json, '$.content')
                WHERE key = 'text' AND type = 'text'
                  AND fullkey NOT LIKE '%.attrs%'
                  AND fullkey NOT LIKE '%.marks%'
                ORDER BY id
              )), '') END, NEW.labels_json, NEW.resources_json);
             END;

CREATE TRIGGER project_search_delete AFTER DELETE ON project BEGIN
               DELETE FROM project_search WHERE rowid = OLD.rowid;
             END;

CREATE VIRTUAL TABLE comment_search USING fts5(
               activity_id UNINDEXED, issue_id UNINDEXED, workspace_id UNINDEXED, body
             );

CREATE TRIGGER comment_search_insert AFTER INSERT ON issue_activity
             WHEN NEW.kind = 'comment' BEGIN
               INSERT INTO comment_search(rowid, activity_id, issue_id, workspace_id, body)
                 SELECT NEW.rowid, NEW.id, NEW.issue_id, issue.workspace_id, NEW.body
                 FROM issue WHERE issue.id = NEW.issue_id;
             END;

CREATE TRIGGER comment_search_delete AFTER DELETE ON issue_activity BEGIN
               DELETE FROM comment_search WHERE rowid = OLD.rowid;
             END;

CREATE TABLE timer_session (
                   singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                   clean_exit INTEGER NOT NULL DEFAULT 1 CHECK (clean_exit IN (0, 1)),
                   recovered_timers INTEGER NOT NULL DEFAULT 0
                 );

CREATE INDEX issue_milestone ON issue(milestone_id);

CREATE TABLE planning_node (
                   id TEXT PRIMARY KEY,
                   project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                   parent_id TEXT REFERENCES planning_node(id) ON DELETE CASCADE,
                   kind TEXT NOT NULL CHECK (kind IN ('module', 'feature')),
                   name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
                   description TEXT NOT NULL DEFAULT '',
                   position INTEGER NOT NULL DEFAULT 0
                 , archived_at TEXT, archive_group_id TEXT, deleted_at TEXT, dissolve_snapshot_json TEXT);

CREATE INDEX planning_node_project ON planning_node(project_id, parent_id, position);

CREATE TABLE planning_node_issue (
                   issue_id TEXT PRIMARY KEY REFERENCES issue(id) ON DELETE CASCADE,
                   node_id TEXT NOT NULL REFERENCES planning_node(id) ON DELETE CASCADE
                 );

CREATE INDEX planning_node_issue_node ON planning_node_issue(node_id);

CREATE TRIGGER planning_node_issue_project_move
                   AFTER UPDATE OF project_id ON issue
                   WHEN OLD.project_id IS NOT NEW.project_id
                   BEGIN DELETE FROM planning_node_issue WHERE issue_id = NEW.id; END;

CREATE TABLE issue_alias (
                   workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                   alias TEXT NOT NULL,
                   issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
                   PRIMARY KEY(workspace_id, alias)
                 );

CREATE INDEX issue_alias_issue ON issue_alias(issue_id);

CREATE TABLE cycle_transition (
               source_cycle_id TEXT PRIMARY KEY REFERENCES cycle(id) ON DELETE CASCADE,
               destination_cycle_id TEXT REFERENCES cycle(id) ON DELETE SET NULL,
               policy TEXT NOT NULL,
               boundary_at TEXT NOT NULL,
               captured_at TEXT NOT NULL,
               result TEXT NOT NULL CHECK (result IN ('pending', 'completed'))
             , last_retried_at TEXT);

CREATE TABLE "cycle_snapshot_issue" (
               cycle_id TEXT NOT NULL REFERENCES cycle(id) ON DELETE CASCADE,
               issue_id TEXT NOT NULL,
               identifier TEXT NOT NULL,
               status TEXT NOT NULL,
               estimate_value INTEGER,
               completed INTEGER NOT NULL CHECK (completed IN (0, 1)),
               rollover_decision TEXT CHECK (rollover_decision IN ('moved', 'left', 'skipped')),
               PRIMARY KEY(cycle_id, issue_id)
             );

CREATE TABLE media (
               id TEXT PRIMARY KEY,
               mime TEXT NOT NULL,
               bytes INTEGER NOT NULL,
               width INTEGER NOT NULL,
               height INTEGER NOT NULL,
               sha256 TEXT NOT NULL,
               created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             );

CREATE TABLE cycle_series (
                   id TEXT PRIMARY KEY,
                   workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
                   name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
                   duration_days INTEGER NOT NULL CHECK (duration_days BETWEEN 1 AND 56),
                   cooldown_days INTEGER NOT NULL CHECK (cooldown_days BETWEEN 0 AND 7),
                   upcoming_count INTEGER NOT NULL CHECK (upcoming_count BETWEEN 1 AND 12),
                   start_date TEXT NOT NULL,
                   rollover_policy TEXT NOT NULL CHECK (rollover_policy IN ('automatic', 'leave', 'ask')),
                   project_scope_json TEXT NOT NULL DEFAULT '[]',
                   time_zone TEXT NOT NULL DEFAULT 'UTC',
                   start_local_time TEXT NOT NULL DEFAULT '00:00',
                   enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)), interval_days INTEGER,
                   UNIQUE (workspace_id, name)
                 );

CREATE UNIQUE INDEX cycle_series_index ON cycle(series_id, schedule_index);

CREATE TABLE cycle_reconcile_progress (
                   series_id TEXT PRIMARY KEY REFERENCES cycle_series(id) ON DELETE CASCADE,
                   total INTEGER NOT NULL,
                   processed INTEGER NOT NULL DEFAULT 0,
                   updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                 );

CREATE INDEX planning_node_lifecycle ON planning_node(project_id, deleted_at, archived_at, parent_id, position);

CREATE TABLE label (
               id TEXT PRIMARY KEY,
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 80),
               color TEXT NOT NULL CHECK (length(color) = 7 AND color GLOB '#[0-9A-Fa-f]*'),
               archived_at TEXT,
               deleted_at TEXT
             , version INTEGER NOT NULL DEFAULT 1);

CREATE UNIQUE INDEX label_workspace_name ON label(workspace_id, lower(trim(name)))
               WHERE deleted_at IS NULL;

CREATE TABLE issue_label (
               issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
               label_id TEXT NOT NULL REFERENCES label(id) ON DELETE CASCADE,
               PRIMARY KEY(issue_id, label_id)
             );

CREATE INDEX issue_label_reverse ON issue_label(label_id, issue_id);

CREATE TABLE project_label (
               project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
               label_id TEXT NOT NULL REFERENCES label(id) ON DELETE CASCADE,
               PRIMARY KEY(project_id, label_id)
             );

CREATE INDEX project_label_reverse ON project_label(label_id, project_id);

CREATE TABLE label_detached (
               label_id TEXT NOT NULL REFERENCES label(id) ON DELETE CASCADE,
               owner_kind TEXT NOT NULL CHECK (owner_kind IN ('issue', 'project')),
               owner_id TEXT NOT NULL,
               PRIMARY KEY(label_id, owner_kind, owner_id)
             );

CREATE TRIGGER issue_labels_block_archived_insert BEFORE INSERT ON issue
               WHEN EXISTS(SELECT 1 FROM json_each(NEW.labels_json) entry JOIN label
                 ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                 WHERE label.archived_at IS NOT NULL OR label.deleted_at IS NOT NULL)
               BEGIN SELECT RAISE(ABORT, 'Cannot assign an archived label.'); END;

CREATE TRIGGER issue_labels_block_archived_update BEFORE UPDATE OF labels_json ON issue
               WHEN EXISTS(SELECT 1 FROM json_each(NEW.labels_json) entry JOIN label
                 ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                 WHERE (label.archived_at IS NOT NULL OR label.deleted_at IS NOT NULL)
                   AND NOT EXISTS(SELECT 1 FROM json_each(OLD.labels_json) prior
                     WHERE lower(trim(prior.value)) = lower(trim(entry.value))))
               BEGIN SELECT RAISE(ABORT, 'Cannot assign an archived label.'); END;

CREATE TRIGGER project_labels_block_archived_insert BEFORE INSERT ON project
               WHEN EXISTS(SELECT 1 FROM json_each(NEW.labels_json) entry JOIN label
                 ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                 WHERE label.archived_at IS NOT NULL OR label.deleted_at IS NOT NULL)
               BEGIN SELECT RAISE(ABORT, 'Cannot assign an archived label.'); END;

CREATE TRIGGER project_labels_block_archived_update BEFORE UPDATE OF labels_json ON project
               WHEN EXISTS(SELECT 1 FROM json_each(NEW.labels_json) entry JOIN label
                 ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                 WHERE (label.archived_at IS NOT NULL OR label.deleted_at IS NOT NULL)
                   AND NOT EXISTS(SELECT 1 FROM json_each(OLD.labels_json) prior
                     WHERE lower(trim(prior.value)) = lower(trim(entry.value))))
               BEGIN SELECT RAISE(ABORT, 'Cannot assign an archived label.'); END;

CREATE TRIGGER issue_labels_insert AFTER INSERT ON issue BEGIN
               INSERT OR IGNORE INTO label(id, workspace_id, name, color)
                 SELECT lower(hex(randomblob(16))), NEW.workspace_id, trim(value), '#999aa3'
                 FROM json_each(NEW.labels_json) WHERE length(trim(value)) BETWEEN 1 AND 80;
               INSERT OR IGNORE INTO issue_label(issue_id, label_id)
                 SELECT NEW.id, label.id FROM json_each(NEW.labels_json) entry JOIN label
                   ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                   AND label.deleted_at IS NULL;
             END;

CREATE TRIGGER issue_labels_update AFTER UPDATE OF labels_json ON issue BEGIN
               INSERT OR IGNORE INTO label(id, workspace_id, name, color)
                 SELECT lower(hex(randomblob(16))), NEW.workspace_id, trim(value), '#999aa3'
                 FROM json_each(NEW.labels_json) WHERE length(trim(value)) BETWEEN 1 AND 80;
               DELETE FROM issue_label WHERE issue_id = NEW.id;
               INSERT OR IGNORE INTO issue_label(issue_id, label_id)
                 SELECT NEW.id, label.id FROM json_each(NEW.labels_json) entry JOIN label
                   ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                   AND label.deleted_at IS NULL;
             END;

CREATE TRIGGER project_labels_insert AFTER INSERT ON project BEGIN
               INSERT OR IGNORE INTO label(id, workspace_id, name, color)
                 SELECT lower(hex(randomblob(16))), NEW.workspace_id, trim(value), '#999aa3'
                 FROM json_each(NEW.labels_json) WHERE length(trim(value)) BETWEEN 1 AND 80;
               INSERT OR IGNORE INTO project_label(project_id, label_id)
                 SELECT NEW.id, label.id FROM json_each(NEW.labels_json) entry JOIN label
                   ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                   AND label.deleted_at IS NULL;
             END;

CREATE TRIGGER project_labels_update AFTER UPDATE OF labels_json ON project BEGIN
               INSERT OR IGNORE INTO label(id, workspace_id, name, color)
                 SELECT lower(hex(randomblob(16))), NEW.workspace_id, trim(value), '#999aa3'
                 FROM json_each(NEW.labels_json) WHERE length(trim(value)) BETWEEN 1 AND 80;
               DELETE FROM project_label WHERE project_id = NEW.id;
               INSERT OR IGNORE INTO project_label(project_id, label_id)
                 SELECT NEW.id, label.id FROM json_each(NEW.labels_json) entry JOIN label
                   ON label.workspace_id = NEW.workspace_id AND lower(label.name) = lower(trim(entry.value))
                   AND label.deleted_at IS NULL;
             END;

CREATE INDEX project_lifecycle ON project(workspace_id, deleted_at, archived_at, name);

CREATE INDEX milestone_lifecycle ON milestone(project_id, deleted_at, sort_order);

CREATE TABLE mutation_receipt (
               mutation_id TEXT PRIMARY KEY,
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               operation TEXT NOT NULL,
               request_hash TEXT NOT NULL,
               result_json TEXT NOT NULL,
               workspace_revision INTEGER NOT NULL,
               created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             , effects_json TEXT);

CREATE INDEX mutation_receipt_workspace ON mutation_receipt(workspace_id, created_at);

CREATE TRIGGER issue_version_insert AFTER INSERT ON issue BEGIN
               UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
             END;

CREATE TRIGGER issue_version_update AFTER UPDATE ON issue
             WHEN NEW.version = OLD.version BEGIN
               UPDATE issue SET version = OLD.version + 1 WHERE id = NEW.id;
               UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
             END;

CREATE TRIGGER project_version_insert AFTER INSERT ON project BEGIN
               UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
             END;

CREATE TRIGGER project_version_update AFTER UPDATE ON project
             WHEN NEW.version = OLD.version BEGIN
               UPDATE project SET version = OLD.version + 1 WHERE id = NEW.id;
               UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
             END;

CREATE TABLE milestone_detached_issue (
               milestone_id TEXT NOT NULL REFERENCES milestone(id) ON DELETE CASCADE,
               issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
               detached_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
               PRIMARY KEY(milestone_id, issue_id)
             );

CREATE TRIGGER saved_view_version_update AFTER UPDATE OF definition_json ON saved_view
                 WHEN NEW.version = OLD.version BEGIN
                   UPDATE saved_view SET version = OLD.version + 1 WHERE id = NEW.id;
                 END;

CREATE TRIGGER label_version_update
                 AFTER UPDATE OF name, color, archived_at, deleted_at ON label
                 WHEN NEW.version = OLD.version BEGIN
                   UPDATE label SET version = OLD.version + 1 WHERE id = NEW.id;
                   UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
                 END;

CREATE TRIGGER time_entry_version_update
                 AFTER UPDATE OF stopped_at, duration_seconds, note ON time_entry
                 WHEN NEW.version = OLD.version BEGIN
                   UPDATE time_entry SET version = OLD.version + 1 WHERE id = NEW.id;
                   UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
                 END;

CREATE TRIGGER time_entry_revision_insert
                 AFTER INSERT ON time_entry BEGIN
                   UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
                 END;

CREATE TRIGGER time_entry_revision_delete
                 AFTER DELETE ON time_entry BEGIN
                   UPDATE workspace SET revision = revision + 1 WHERE id = OLD.workspace_id;
                 END;

CREATE TRIGGER label_revision_insert
                 AFTER INSERT ON label BEGIN
                   UPDATE workspace SET revision = revision + 1 WHERE id = NEW.workspace_id;
                 END;

CREATE VIRTUAL TABLE planning_node_search USING fts5(
               node_id UNINDEXED, workspace_id UNINDEXED, name, description
             );

CREATE TRIGGER planning_node_search_insert AFTER INSERT ON planning_node BEGIN
               INSERT INTO planning_node_search(rowid, node_id, workspace_id, name, description)
               SELECT NEW.rowid, NEW.id, project.workspace_id, NEW.name, NEW.description
               FROM project WHERE project.id = NEW.project_id;
             END;

CREATE TRIGGER planning_node_search_update AFTER UPDATE OF name, description ON planning_node BEGIN
               DELETE FROM planning_node_search WHERE rowid = OLD.rowid;
               INSERT INTO planning_node_search(rowid, node_id, workspace_id, name, description)
               SELECT NEW.rowid, NEW.id, project.workspace_id, NEW.name, NEW.description
               FROM project WHERE project.id = NEW.project_id;
             END;

CREATE TRIGGER planning_node_search_delete AFTER DELETE ON planning_node BEGIN
               DELETE FROM planning_node_search WHERE rowid = OLD.rowid;
             END;

CREATE TABLE project_cascade_member (
               cascade_id TEXT NOT NULL,
               project_id TEXT NOT NULL REFERENCES project(id),
               kind TEXT NOT NULL CHECK(kind IN ('issue','node','milestone')),
               member_id TEXT NOT NULL,
               was_archived INTEGER NOT NULL DEFAULT 0,
               PRIMARY KEY(cascade_id, kind, member_id)
             );

CREATE INDEX project_cascade_lookup ON project_cascade_member(project_id, kind, member_id);

CREATE INDEX project_updated_index
               ON project(workspace_id, updated_at DESC, id DESC);

CREATE TRIGGER project_touch_updated
             AFTER UPDATE ON project WHEN NEW.updated_at = OLD.updated_at BEGIN
               UPDATE project SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = NEW.id;
             END;

CREATE TABLE workspace_option (
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               option_key TEXT NOT NULL,
               option_value TEXT NOT NULL,
               PRIMARY KEY(workspace_id, option_key)
             );

CREATE TABLE draft (
               workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
               draft_key TEXT NOT NULL,
               payload_json TEXT NOT NULL,
               version INTEGER NOT NULL DEFAULT 1,
               updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
               PRIMARY KEY(workspace_id, draft_key)
             );

CREATE TABLE issue_status_visit (
               id INTEGER PRIMARY KEY,
               issue_id TEXT NOT NULL REFERENCES issue(id) ON DELETE CASCADE,
               status_name TEXT NOT NULL,
               status_category TEXT NOT NULL,
               entered_at TEXT NOT NULL,
               left_at TEXT,
               mutation_id TEXT
             );

CREATE INDEX issue_status_visit_order ON issue_status_visit(issue_id, entered_at, id);

CREATE TRIGGER issue_status_visit_insert AFTER INSERT ON issue BEGIN
               INSERT INTO issue_status_visit(issue_id, status_name, status_category, entered_at)
               VALUES (NEW.id, NEW.status,
                 COALESCE((SELECT category FROM workflow_status
                           WHERE workspace_id = NEW.workspace_id AND name = NEW.status), 'Backlog'),
                 NEW.created_at);
             END;

CREATE TRIGGER issue_status_visit_change AFTER UPDATE OF status ON issue
             WHEN OLD.status <> NEW.status BEGIN
               UPDATE issue_status_visit SET left_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
               WHERE id = (SELECT id FROM issue_status_visit
                           WHERE issue_id = NEW.id AND left_at IS NULL ORDER BY id DESC LIMIT 1);
               INSERT INTO issue_status_visit(issue_id, status_name, status_category, entered_at)
               VALUES (NEW.id, NEW.status,
                 COALESCE((SELECT category FROM workflow_status
                           WHERE workspace_id = NEW.workspace_id AND name = NEW.status), 'Backlog'),
                 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
             END;

CREATE UNIQUE INDEX one_running_timer_per_issue
            ON time_entry(issue_id) WHERE stopped_at IS NULL AND deleted_at IS NULL;

CREATE TABLE timer_recovery_notice (
               id TEXT PRIMARY KEY,
               entry_id TEXT NOT NULL REFERENCES time_entry(id),
               kind TEXT NOT NULL CHECK (kind IN ('closed', 'long_gap', 'backward_clock', 'sleep_uncertain')),
               gap_seconds INTEGER NOT NULL,
                   gap_ms INTEGER NOT NULL DEFAULT 0,
               created_at INTEGER NOT NULL,
               resolved_at INTEGER,
               UNIQUE(entry_id, created_at)
             );

CREATE INDEX timer_recovery_pending ON timer_recovery_notice(resolved_at);

CREATE TABLE project_resource (
               id TEXT PRIMARY KEY,
               project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
               title TEXT NOT NULL,
               url TEXT NOT NULL,
               sort_order INTEGER NOT NULL,
               version INTEGER NOT NULL DEFAULT 1
             );

CREATE INDEX project_resource_order ON project_resource(project_id, sort_order, id);

PRAGMA application_id = 0x504d4150;
PRAGMA user_version = 1;
