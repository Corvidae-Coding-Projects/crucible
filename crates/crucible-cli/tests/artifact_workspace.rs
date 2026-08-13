use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
#[allow(unused_imports)]
use vstd::prelude::*;

const ABC_ID: &str = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const MAX_LOCAL_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "crucible-artifact-{label}-{}-{sequence}",
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
        assert!(self.path.starts_with(std::env::temp_dir()));
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

fn initialize(root: &Path, current_directory: &Path) {
    let output = run(
        &["init", root.to_str().expect("UTF-8 workspace path")],
        current_directory,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn object_path(root: &Path, id: &str) -> PathBuf {
    let digest = id.strip_prefix("sha256:").expect("SHA-256 artifact ID");
    root.join(".crucible/objects/sha256")
        .join(&digest[0..2])
        .join(&digest[2..4])
        .join(digest)
}

fn database(root: &Path) -> Connection {
    Connection::open(root.join(".crucible/database.sqlite")).expect("open workspace database")
}

fn scalar_i64(connection: &Connection, sql: &str) -> i64 {
    connection
        .query_row(sql, [], |row| row.get(0))
        .expect("query scalar")
}

#[test]
fn artifact_import_publishes_verified_bytes_and_a_database_reference() {
    let temporary = TemporaryDirectory::new("import");
    let root = temporary.path().join("workspace");
    initialize(&root, temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write seed");

    let output = run(
        &[
            "artifact",
            "import",
            source.to_str().expect("UTF-8 source path"),
            root.to_str().expect("UTF-8 workspace path"),
        ],
        temporary.path(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 output"),
        format!("{ABC_ID}\n")
    );
    assert_eq!(
        std::fs::read(object_path(&root, ABC_ID)).expect("read object"),
        b"abc"
    );

    let connection = database(&root);
    let artifact: (String, String, String, i64, Option<String>) = connection
        .query_row(
            "SELECT id, algorithm, digest, size_bytes, media_type FROM artifacts",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("read artifact row");
    assert_eq!(artifact.0, ABC_ID);
    assert_eq!(artifact.1, "sha256");
    assert_eq!(artifact.2, ABC_ID.trim_start_matches("sha256:"));
    assert_eq!(artifact.3, 3);
    assert_eq!(artifact.4, None);
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM artifact_imports"),
        1
    );
    let provenance: Vec<u8> = connection
        .query_row("SELECT source_path FROM artifact_imports", [], |row| {
            row.get(0)
        })
        .expect("read source provenance");
    assert_eq!(
        provenance,
        source.to_str().expect("UTF-8 source path").as_bytes()
    );
}

#[test]
fn duplicate_contents_store_once_and_retain_each_source_provenance() {
    let temporary = TemporaryDirectory::new("deduplicate");
    initialize(temporary.path(), temporary.path());
    let first = temporary.path().join("first.bin");
    let second = temporary.path().join("second.bin");
    std::fs::write(&first, b"abc").expect("write first source");
    std::fs::write(&second, b"abc").expect("write second source");

    for source in [&first, &second, &first] {
        let output = run(
            &["artifact", "import", source.to_str().expect("UTF-8 source")],
            temporary.path(),
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("{ABC_ID}\n")
        );
    }

    let connection = database(temporary.path());
    assert_eq!(scalar_i64(&connection, "SELECT COUNT(*) FROM artifacts"), 1);
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM artifact_imports"),
        2
    );
    assert_eq!(
        std::fs::read(object_path(temporary.path(), ABC_ID)).expect("read deduplicated object"),
        b"abc"
    );
}

#[test]
fn provenance_uses_one_canonical_absolute_path_for_equivalent_spellings() {
    let temporary = TemporaryDirectory::new("canonical-provenance");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");

    for spelling in [
        "seed.bin",
        "./seed.bin",
        source.to_str().expect("UTF-8 source"),
    ] {
        let output = run(&["artifact", "import", spelling], temporary.path());
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let connection = database(temporary.path());
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM artifact_imports"),
        1
    );
    let provenance: Vec<u8> = connection
        .query_row("SELECT source_path FROM artifact_imports", [], |row| {
            row.get(0)
        })
        .expect("read canonical provenance");
    assert_eq!(
        provenance,
        source.to_str().expect("UTF-8 source").as_bytes()
    );
}

#[test]
fn stored_artifact_survives_source_removal_and_corruption_is_detected() {
    let temporary = TemporaryDirectory::new("verify");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("ephemeral.bin");
    std::fs::write(&source, b"abc").expect("write source");
    let imported = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );
    assert!(imported.status.success());
    std::fs::remove_file(&source).expect("remove original source");

    let verified = run(&["artifact", "verify", ABC_ID], temporary.path());
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&verified.stdout),
        format!("verified {ABC_ID}\n")
    );

    std::fs::write(object_path(temporary.path(), ABC_ID), b"abd").expect("corrupt object");
    let corrupted = run(&["artifact", "verify", ABC_ID], temporary.path());
    assert!(!corrupted.status.success());
    assert!(String::from_utf8_lossy(&corrupted.stderr).contains("artifact integrity check failed"));
}

#[test]
fn artifact_verify_rejects_a_database_digest_that_disagrees_with_the_id() {
    let temporary = TemporaryDirectory::new("database-digest");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");
    let imported = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );
    assert!(imported.status.success());

    let connection = database(temporary.path());
    connection
        .execute(
            "UPDATE artifacts SET digest = ?1 WHERE id = ?2",
            ["0".repeat(64), ABC_ID.to_owned()],
        )
        .expect("corrupt stored digest");
    drop(connection);

    let verified = run(&["artifact", "verify", ABC_ID], temporary.path());
    assert!(!verified.status.success());
    assert!(String::from_utf8_lossy(&verified.stderr).contains("artifact integrity check failed"));
}

#[cfg(unix)]
#[test]
fn artifact_verify_refuses_symlinked_object_fanout_directories() {
    use std::os::unix::fs::symlink;

    let temporary = TemporaryDirectory::new("object-parent-symlink");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");
    let imported = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );
    assert!(imported.status.success());

    let algorithm = temporary.path().join(".crucible/objects/sha256");
    let outside = temporary.path().join("relocated-sha256");
    std::fs::rename(&algorithm, &outside).expect("move algorithm directory outside store");
    symlink(&outside, &algorithm).expect("replace algorithm directory with symlink");

    let verified = run(&["artifact", "verify", ABC_ID], temporary.path());
    assert!(!verified.status.success());
    assert!(String::from_utf8_lossy(&verified.stderr).contains("artifact integrity check failed"));
}

#[test]
fn malformed_artifact_ids_cannot_select_object_store_paths() {
    let temporary = TemporaryDirectory::new("invalid-id");
    initialize(temporary.path(), temporary.path());
    let outside = temporary.path().join("outside");
    std::fs::write(&outside, b"untouched").expect("write outside sentinel");

    for id in ["../outside", "sha256:../../outside", "sha256:BA78"] {
        let output = run(&["artifact", "verify", id], temporary.path());
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid artifact ID"));
    }
    assert_eq!(std::fs::read(outside).expect("read sentinel"), b"untouched");
}

#[cfg(unix)]
#[test]
fn artifact_import_refuses_symlink_sources() {
    use std::os::unix::fs::symlink;

    let temporary = TemporaryDirectory::new("source-symlink");
    initialize(temporary.path(), temporary.path());
    let target = temporary.path().join("target.bin");
    let source = temporary.path().join("source.bin");
    std::fs::write(&target, b"abc").expect("write target");
    symlink(&target, &source).expect("create source symlink");

    let output = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsafe artifact source"));
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifacts"
        ),
        0
    );
}

#[test]
fn artifact_import_rejects_oversized_sources_before_reading_them() {
    let temporary = TemporaryDirectory::new("source-cap");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("sparse.bin");
    let file = std::fs::File::create(&source).expect("create sparse source");
    file.set_len(MAX_LOCAL_ARTIFACT_BYTES + 1)
        .expect("set sparse source length");
    drop(file);

    let output = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("67108864-byte import limit"));
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifacts"
        ),
        0
    );
}

#[test]
fn failed_object_publication_never_commits_a_database_reference() {
    let temporary = TemporaryDirectory::new("atomicity");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");
    std::fs::write(
        temporary.path().join(".crucible/objects/sha256"),
        b"occupied",
    )
    .expect("occupy algorithm directory");

    let output = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );

    assert!(!output.status.success());
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifacts"
        ),
        0
    );
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifact_imports"
        ),
        0
    );
}

#[test]
fn import_recovers_after_a_complete_object_was_published_before_database_commit() {
    let temporary = TemporaryDirectory::new("restart-after-object");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");
    let target = object_path(temporary.path(), ABC_ID);
    std::fs::create_dir_all(target.parent().expect("object parent"))
        .expect("create interrupted publication fanout");
    std::fs::write(&target, b"abc").expect("simulate synced published object");
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifacts"
        ),
        0
    );

    let output = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifacts"
        ),
        1
    );
    assert!(run(&["artifact", "verify", ABC_ID], temporary.path())
        .status
        .success());
}

#[test]
fn import_rejects_a_conflicting_object_left_before_database_commit() {
    let temporary = TemporaryDirectory::new("restart-conflicting-object");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");
    let target = object_path(temporary.path(), ABC_ID);
    std::fs::create_dir_all(target.parent().expect("object parent"))
        .expect("create interrupted publication fanout");
    std::fs::write(&target, b"abd").expect("simulate conflicting published object");

    let output = run(
        &["artifact", "import", source.to_str().expect("UTF-8 source")],
        temporary.path(),
    );

    assert!(!output.status.success());
    assert_eq!(
        scalar_i64(
            &database(temporary.path()),
            "SELECT COUNT(*) FROM artifacts"
        ),
        0
    );
    assert_eq!(
        std::fs::read(target).expect("read conflicting object"),
        b"abd"
    );
}

#[test]
fn concurrent_duplicate_imports_are_idempotent() {
    let temporary = TemporaryDirectory::new("concurrent");
    initialize(temporary.path(), temporary.path());
    let source = temporary.path().join("seed.bin");
    std::fs::write(&source, b"abc").expect("write source");
    let mut children = Vec::new();
    for _ in 0..6 {
        children.push(
            Command::new(env!("CARGO_BIN_EXE_crucible"))
                .args(["artifact", "import", source.to_str().expect("UTF-8 source")])
                .current_dir(temporary.path())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("start concurrent import"),
        );
    }

    for child in children {
        let output = child.wait_with_output().expect("wait for import");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("{ABC_ID}\n")
        );
    }
    let connection = database(temporary.path());
    assert_eq!(scalar_i64(&connection, "SELECT COUNT(*) FROM artifacts"), 1);
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM artifact_imports"),
        1
    );
    assert_eq!(
        std::fs::read(object_path(temporary.path(), ABC_ID)).expect("read object"),
        b"abc"
    );
}

#[test]
fn artifact_subcommands_have_exact_argument_shapes() {
    let temporary = TemporaryDirectory::new("arguments");
    initialize(temporary.path(), temporary.path());
    for arguments in [
        vec!["artifact"],
        vec!["artifact", "import"],
        vec!["artifact", "verify"],
        vec!["artifact", "erase", ABC_ID],
        vec!["artifact", "verify", ABC_ID, ".", "extra"],
    ] {
        let output = run(&arguments, temporary.path());
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
    }
}
