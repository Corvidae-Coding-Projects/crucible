use crucible_core::{ContentDigest, MAX_PERSISTENCE_BATCH_BYTES, MAX_PERSISTENCE_BATCH_ITEMS};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct Workspace {
    root: PathBuf,
}

impl Workspace {
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "crucible-storage-maintenance-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create workspace root");
        let output = Command::new(env!("CARGO_BIN_EXE_crucible"))
            .arg("init")
            .arg(&root)
            .output()
            .expect("initialize workspace");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { root }
    }

    fn command(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_crucible"))
            .args(arguments)
            .current_dir(&self.root)
            .output()
            .expect("run Crucible command")
    }

    fn database(&self) -> Connection {
        Connection::open(self.root.join(".crucible/database.sqlite")).expect("open database")
    }

    fn object_path_for_bytes(&self, bytes: &[u8]) -> PathBuf {
        let id = ContentDigest::from_bytes(bytes)
            .expect("bounded bytes")
            .into_artifact_id();
        let digest = id.as_str().strip_prefix("sha256:").expect("SHA-256 ID");
        self.root
            .join(".crucible/objects/sha256")
            .join(&digest[0..2])
            .join(&digest[2..4])
            .join(digest)
    }

    fn write_orphan(&self, bytes: &[u8]) -> PathBuf {
        let path = self.object_path_for_bytes(bytes);
        std::fs::create_dir_all(path.parent().expect("object parent")).expect("create shard");
        std::fs::write(&path, bytes).expect("write orphan object");
        path
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn assert_success(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("UTF-8 command output")
}

#[test]
fn integrity_check_and_conservative_gc_handle_unreferenced_complete_objects() {
    let workspace = Workspace::new();
    let source = workspace.root.join("retained.bin");
    std::fs::write(&source, b"retained").expect("write source");
    let imported = workspace.command(&["artifact", "import", source.to_str().expect("UTF-8")]);
    assert_success(&imported);
    let orphan = workspace.write_orphan(b"completed but unreferenced");

    let check = assert_success(&workspace.command(&["artifact", "check"]));
    assert!(check.contains("verified=1"), "{check}");
    assert!(check.contains("orphaned=1"), "{check}");

    let gc = assert_success(&workspace.command(&["artifact", "gc"]));
    assert!(gc.contains("collected=1"), "{gc}");
    assert!(gc.contains("preserved=1"), "{gc}");
    assert!(!orphan.exists());
    assert!(workspace.object_path_for_bytes(b"retained").exists());
}

#[test]
fn integrity_check_rejects_corrupt_referenced_contents() {
    let workspace = Workspace::new();
    let source = workspace.root.join("corrupt.bin");
    std::fs::write(&source, b"authentic").expect("write source");
    assert_success(&workspace.command(&["artifact", "import", source.to_str().expect("UTF-8")]));
    std::fs::write(workspace.object_path_for_bytes(b"authentic"), b"tampered")
        .expect("corrupt object");

    let check = workspace.command(&["artifact", "check"]);
    assert!(!check.status.success());
    assert!(check.stdout.is_empty());
    assert!(String::from_utf8_lossy(&check.stderr).contains("integrity failure"));
}

#[test]
fn active_publication_lease_blocks_collection_and_persistence_batches_stay_bounded() {
    let workspace = Workspace::new();
    let source = workspace.root.join("batched.bin");
    std::fs::write(&source, b"batch evidence").expect("write source");
    assert_success(&workspace.command(&["artifact", "import", source.to_str().expect("UTF-8")]));

    let connection = workspace.database();
    let batch: (i64, i64, String) = connection
        .query_row(
            "SELECT item_count, encoded_bytes, status FROM persistence_batches ORDER BY rowid DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("persistence batch");
    assert!(batch.0 > 0 && batch.0 <= MAX_PERSISTENCE_BATCH_ITEMS as i64);
    assert!(batch.1 > 0 && batch.1 <= MAX_PERSISTENCE_BATCH_BYTES as i64);
    assert_eq!(batch.2, "committed");

    let generation: i64 = connection
        .query_row(
            "SELECT id FROM storage_generations WHERE status = 'open'",
            [],
            |row| row.get(0),
        )
        .expect("open generation");
    connection
        .execute(
            "INSERT INTO storage_leases(id, generation_id, artifact_id, owner_identity, status, expires_epoch)
             VALUES ('test-active-lease', ?1, NULL, 'storage-maintenance-test', 'active', 9223372036854775807)",
            params![generation],
        )
        .expect("insert active lease");
    drop(connection);
    let orphan = workspace.write_orphan(b"leased publication window");

    let gc = workspace.command(&["artifact", "gc"]);
    assert!(!gc.status.success());
    assert!(String::from_utf8_lossy(&gc.stderr).contains("active publication lease"));
    assert!(orphan.exists());
}

#[test]
fn object_store_paths_are_never_followed_through_symlinks() {
    let workspace = Workspace::new();
    let algorithm = workspace.root.join(".crucible/objects/sha256");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(Path::new("/tmp"), &algorithm).expect("create hostile symlink");
        let check = workspace.command(&["artifact", "check"]);
        assert!(!check.status.success());
        assert!(String::from_utf8_lossy(&check.stderr).contains("unsafe workspace"));
    }
}
