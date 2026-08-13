use rusqlite::Connection;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
#[expect(
    unused_imports,
    reason = "Verus requires the prelude crate marker even when this Rust test names no vstd item"
)]
use vstd::prelude::*;

const APPLICATION_ID: i64 = 0x4352_5543;
const SCHEMA_VERSION: i64 = 3;

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "crucible-{label}-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("create isolated temporary directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let temporary_root = std::env::temp_dir();
        assert!(self.path.starts_with(&temporary_root));
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run(arguments: &[&str], current_directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(arguments)
        .current_dir(current_directory)
        .output()
        .expect("run crucible")
}

fn directory_names(path: &Path) -> Vec<OsString> {
    let mut names: Vec<OsString> = std::fs::read_dir(path)
        .expect("read directory")
        .map(|entry| entry.expect("read directory entry").file_name())
        .collect();
    names.sort();
    names
}

fn assert_valid_initialized_workspace(root: &Path) {
    let state = root.join(".crucible");
    for relative in [
        "corpus/seeds",
        "corpus/interesting",
        "corpus/coverage",
        "corpus/regression",
        "corpus/minimized",
        "findings",
        "objects",
        "runs",
        "reports",
    ] {
        assert!(state.join(relative).is_dir(), "missing {relative}");
    }

    let database = state.join("database.sqlite");
    assert!(database.is_file());
    let connection = Connection::open(database).expect("open initialized database");
    let application_id: i64 = connection
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .expect("read application id");
    let user_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .expect("read schema version");
    let quick_check: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .expect("run integrity check");
    let migrations: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE
                (version = 1 AND name = 'initialize-workspace') OR
                (version = 2 AND name = 'add-artifact-store') OR
                (version = 3 AND name = 'add-local-run-evidence')",
            [],
            |row| row.get(0),
        )
        .expect("read migration history");
    let artifact_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name IN ('artifacts', 'artifact_imports')",
            [],
            |row| row.get(0),
        )
        .expect("read artifact tables");
    let format: String = connection
        .query_row(
            "SELECT value FROM workspace_metadata WHERE key = 'format'",
            [],
            |row| row.get(0),
        )
        .expect("read workspace format");

    assert_eq!(application_id, APPLICATION_ID);
    assert_eq!(user_version, SCHEMA_VERSION);
    assert_eq!(quick_check, "ok");
    assert_eq!(migrations, 3);
    assert_eq!(artifact_tables, 2);
    assert_eq!(format, "crucible-workspace-v1");
}

fn create_version_one_workspace(root: &Path) {
    let state = root.join(".crucible");
    for relative in [
        "corpus/seeds",
        "corpus/interesting",
        "corpus/coverage",
        "corpus/regression",
        "corpus/minimized",
        "findings",
        "objects",
        "runs",
        "reports",
    ] {
        std::fs::create_dir_all(state.join(relative)).expect("create version-one layout");
    }
    let database = state.join("database.sqlite");
    let connection = Connection::open(&database).expect("create version-one database");
    connection
        .execute_batch(
            "PRAGMA application_id = 1129469251;
             PRAGMA user_version = 1;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY CHECK(version > 0), name TEXT NOT NULL UNIQUE CHECK(length(name) > 0), checksum TEXT NOT NULL CHECK(length(checksum) = 71)) STRICT;
             CREATE TABLE workspace_metadata(key TEXT PRIMARY KEY CHECK(length(key) > 0), value TEXT NOT NULL) STRICT;
             INSERT INTO schema_migrations(version, name, checksum) VALUES (1, 'initialize-workspace', 'sha256:a6793465a272d41191c763e4460c035f7862da2ede3e84c280c3f2b9a8da8d36');
             INSERT INTO workspace_metadata(key, value) VALUES ('format', 'crucible-workspace-v1');",
        )
        .expect("create version-one schema");
}

#[test]
fn init_migrates_an_exact_version_one_workspace_monotonically() {
    let temporary = TemporaryDirectory::new("migrate-v1");
    create_version_one_workspace(temporary.path());
    let database = temporary.path().join(".crucible/database.sqlite");

    let result = run(&["init"], temporary.path());

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_valid_initialized_workspace(temporary.path());
    let connection = Connection::open(database).expect("open migrated database");
    assert_eq!(
        connection
            .query_row(
                "SELECT checksum FROM schema_migrations WHERE version = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .expect("read preserved first migration"),
        "sha256:a6793465a272d41191c763e4460c035f7862da2ede3e84c280c3f2b9a8da8d36"
    );
}

#[test]
fn concurrent_version_one_migration_is_idempotent() {
    let temporary = TemporaryDirectory::new("concurrent-migrate-v1");
    create_version_one_workspace(temporary.path());
    let mut children = Vec::new();
    for _ in 0..8 {
        children.push(
            Command::new(env!("CARGO_BIN_EXE_crucible"))
                .arg("init")
                .current_dir(temporary.path())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("start concurrent migration"),
        );
    }

    for child in children {
        let output = child
            .wait_with_output()
            .expect("wait for concurrent migration");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_valid_initialized_workspace(temporary.path());
}

#[test]
fn init_creates_a_valid_versioned_workspace_and_is_idempotent() {
    let temporary = TemporaryDirectory::new("init");
    let root = temporary.path().join("workspace");

    let first = run(
        &["init", root.to_str().expect("UTF-8 path")],
        temporary.path(),
    );
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_valid_initialized_workspace(&root);

    let second = run(
        &["init", root.to_str().expect("UTF-8 path")],
        temporary.path(),
    );
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_valid_initialized_workspace(&root);
}

#[test]
fn init_without_a_path_uses_the_current_directory() {
    let temporary = TemporaryDirectory::new("current-directory");
    let result = run(&["init"], temporary.path());

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_valid_initialized_workspace(temporary.path());
}

#[test]
fn init_refuses_to_overwrite_a_non_directory_state_path() {
    let temporary = TemporaryDirectory::new("occupied");
    let state = temporary.path().join(".crucible");
    std::fs::write(&state, b"owned by another application").expect("write occupied state path");

    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
    assert_eq!(
        std::fs::read(state).expect("preserve occupied path"),
        b"owned by another application"
    );
}

#[test]
fn init_refuses_an_incompatible_database_without_rewriting_its_identity() {
    let temporary = TemporaryDirectory::new("incompatible-database");
    let state = temporary.path().join(".crucible");
    std::fs::create_dir(&state).expect("create state directory");
    let database = state.join("database.sqlite");
    let connection = Connection::open(&database).expect("create incompatible database");
    connection
        .execute_batch("PRAGMA application_id = 7; PRAGMA user_version = 99;")
        .expect("set incompatible identity");
    drop(connection);

    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
    let connection = Connection::open(database).expect("reopen incompatible database");
    let application_id: i64 = connection
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .expect("read preserved application id");
    let user_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .expect("read preserved schema version");
    assert_eq!(application_id, 7);
    assert_eq!(user_version, 99);
}

#[test]
fn init_rejects_extended_or_altered_database_schema() {
    let temporary = TemporaryDirectory::new("extended-database");
    let first = run(&["init"], temporary.path());
    assert!(first.status.success());
    let database = temporary.path().join(".crucible/database.sqlite");
    let connection = Connection::open(&database).expect("open initialized database");
    connection
        .execute_batch(
            "INSERT INTO workspace_metadata(key, value) VALUES ('foreign', 'state');
             CREATE TABLE foreign_table(value TEXT) STRICT;
             ALTER TABLE workspace_metadata ADD COLUMN foreign_value TEXT;",
        )
        .expect("extend database");
    drop(connection);

    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
    let connection = Connection::open(database).expect("reopen rejected database");
    let foreign_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name = 'foreign_table'",
            [],
            |row| row.get(0),
        )
        .expect("foreign table remains untouched");
    assert_eq!(foreign_tables, 1);
}

#[test]
fn init_rejects_foreign_views_that_mimic_workspace_rows() {
    let temporary = TemporaryDirectory::new("forged-database");
    let state = temporary.path().join(".crucible");
    std::fs::create_dir(&state).expect("create state directory");
    for relative in [
        "corpus",
        "corpus/seeds",
        "corpus/interesting",
        "corpus/coverage",
        "corpus/regression",
        "corpus/minimized",
        "findings",
        "objects",
        "runs",
        "reports",
    ] {
        std::fs::create_dir(state.join(relative)).expect("create forged layout");
    }
    let database = state.join("database.sqlite");
    let connection = Connection::open(database).expect("create forged database");
    connection
        .execute_batch(
            "PRAGMA application_id = 1129469251;
             PRAGMA user_version = 1;
             CREATE VIEW schema_migrations AS
                 SELECT 1 AS version, 'initialize-workspace' AS name;
             CREATE VIEW workspace_metadata AS
                 SELECT 'format' AS key, 'crucible-workspace-v1' AS value;",
        )
        .expect("create forged views");
    drop(connection);

    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
}

#[test]
fn init_rejects_an_occupied_state_directory_without_adopting_it() {
    let temporary = TemporaryDirectory::new("occupied-directory");
    let state = temporary.path().join(".crucible");
    std::fs::create_dir(&state).expect("create occupied state directory");
    std::fs::write(state.join("owner.txt"), b"foreign state").expect("write foreign state");

    let before = directory_names(&state);
    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
    assert_eq!(directory_names(&state), before);
    assert_eq!(
        std::fs::read(state.join("owner.txt")).expect("read preserved state"),
        b"foreign state"
    );
}

#[test]
fn failed_init_does_not_add_layout_to_an_incompatible_state() {
    let temporary = TemporaryDirectory::new("failure-atomicity");
    let state = temporary.path().join(".crucible");
    std::fs::create_dir(&state).expect("create state directory");
    let database = state.join("database.sqlite");
    let connection = Connection::open(database).expect("create incompatible database");
    connection
        .execute_batch("PRAGMA application_id = 7; PRAGMA user_version = 99;")
        .expect("set incompatible identity");
    drop(connection);
    let before = directory_names(&state);

    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
    assert_eq!(directory_names(&state), before);
}

#[cfg(unix)]
#[test]
fn init_refuses_a_state_symlink_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let temporary = TemporaryDirectory::new("symlink");
    let outside = temporary.path().join("outside");
    std::fs::create_dir(&outside).expect("create external target");
    symlink(&outside, temporary.path().join(".crucible")).expect("create state symlink");

    let result = run(&["init"], temporary.path());

    assert!(!result.status.success());
    assert_eq!(
        std::fs::read_dir(&outside)
            .expect("read untouched target")
            .count(),
        0
    );
}

#[cfg(unix)]
#[test]
fn init_refuses_a_selected_root_symlink_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let temporary = TemporaryDirectory::new("root-symlink");
    let outside = temporary.path().join("outside");
    let selected = temporary.path().join("selected");
    std::fs::create_dir(&outside).expect("create external target");
    symlink(&outside, &selected).expect("create selected-root symlink");

    let result = run(
        &["init", selected.to_str().expect("UTF-8 selected path")],
        temporary.path(),
    );

    assert!(!result.status.success());
    assert_eq!(directory_names(&outside), Vec::<OsString>::new());
}

#[cfg(unix)]
#[test]
fn init_refuses_an_intermediate_symlink_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let temporary = TemporaryDirectory::new("ancestor-symlink");
    let outside = temporary.path().join("outside");
    let intermediate = temporary.path().join("intermediate");
    let selected = intermediate.join("selected");
    std::fs::create_dir(&outside).expect("create external target");
    symlink(&outside, &intermediate).expect("create intermediate symlink");

    let result = run(
        &["init", selected.to_str().expect("UTF-8 selected path")],
        temporary.path(),
    );

    assert!(!result.status.success());
    assert_eq!(directory_names(&outside), Vec::<OsString>::new());
}

#[cfg(unix)]
#[test]
fn non_utf8_path_is_a_typed_failure_not_a_panic() {
    use std::os::unix::ffi::OsStringExt;

    let temporary = TemporaryDirectory::new("non-utf8");
    let mut selected = temporary.path().as_os_str().to_os_string().into_vec();
    selected.extend_from_slice(b"/\xff");
    let result = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .arg("init")
        .arg(OsString::from_vec(selected))
        .current_dir(temporary.path())
        .output()
        .expect("run crucible with non-UTF-8 path");

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(stderr.contains("path is not valid UTF-8"), "{stderr}");
}

#[test]
fn concurrent_first_initialization_is_idempotent() {
    let temporary = TemporaryDirectory::new("concurrent-init");
    let selected = temporary.path().join("workspace");
    std::fs::create_dir(&selected).expect("create selected root");
    let mut children = Vec::new();
    for _ in 0..8 {
        children.push(
            Command::new(env!("CARGO_BIN_EXE_crucible"))
                .arg("init")
                .arg(&selected)
                .current_dir(temporary.path())
                .spawn()
                .expect("start concurrent init"),
        );
    }

    for child in children {
        let status = child.wait_with_output().expect("wait for concurrent init");
        assert!(
            status.status.success(),
            "{}",
            String::from_utf8_lossy(&status.stderr)
        );
    }
    assert_valid_initialized_workspace(&selected);
}

#[test]
fn unsupported_arguments_fail_with_usage() {
    let temporary = TemporaryDirectory::new("arguments");
    let result = run(&["init", "one", "two"], temporary.path());

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("usage: crucible init [path]"));
    assert!(!temporary.path().join(".crucible").exists());
}
