#![forbid(unsafe_code)]

mod artifact_store;

pub use artifact_store::*;

#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use vstd::assert_seqs_equal;
use vstd::prelude::*;

verus! {

pub const WORKSPACE_APPLICATION_ID: i64 = 0x4352_5543;

pub const WORKSPACE_SCHEMA_VERSION: i64 = 2;

pub const MAX_CLI_ARGUMENTS: usize = 4;

pub const MAX_CLI_ARGUMENT_BYTES: usize = 4096;

} // verus!
macro_rules! define_byte_literal {
    ($exec_name:ident, $spec_name:ident, $value:literal) => {
        verus! {

        pub open spec fn $spec_name() -> Seq<u8> {
            $value@
        }

        pub fn $exec_name() -> (value: Vec<u8>)
            ensures
                value@ == $spec_name(),
        {
            vstd::slice::slice_to_vec($value)
        }

        } // verus!
    };
}

macro_rules! define_string_literal {
    ($exec_name:ident, $spec_name:ident, [$($character:literal),+ $(,)?]) => {
        verus! {

        pub open spec fn $spec_name() -> Seq<char> {
            seq![$($character),+]
        }

        pub fn $exec_name() -> (value: String)
            ensures
                value@ == $spec_name(),
        {
            let mut value = String::new();
            $(value.push($character);)+
            value
        }

        } // verus!
    };
}

define_byte_literal!(table_kind, table_kind_spec, b"table");
define_byte_literal!(wal_mode, wal_mode_spec, b"wal");
define_byte_literal!(quick_check_ok, quick_check_ok_spec, b"ok");
define_byte_literal!(
    migration_table_sql,
    migration_table_sql_spec,
    b"CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY CHECK(version > 0), name TEXT NOT NULL UNIQUE CHECK(length(name) > 0), checksum TEXT NOT NULL CHECK(length(checksum) = 71)) STRICT"
);
define_byte_literal!(
    metadata_table_sql,
    metadata_table_sql_spec,
    b"CREATE TABLE workspace_metadata(key TEXT PRIMARY KEY CHECK(length(key) > 0), value TEXT NOT NULL) STRICT"
);
define_byte_literal!(migration_name, migration_name_spec, b"initialize-workspace");
define_byte_literal!(
    migration_checksum,
    migration_checksum_spec,
    b"sha256:a6793465a272d41191c763e4460c035f7862da2ede3e84c280c3f2b9a8da8d36"
);
define_byte_literal!(
    artifacts_table_sql,
    artifacts_table_sql_spec,
    b"CREATE TABLE artifacts(id TEXT PRIMARY KEY CHECK(length(id) = 71), algorithm TEXT NOT NULL CHECK(algorithm = 'sha256'), digest TEXT NOT NULL UNIQUE CHECK(length(digest) = 64), size_bytes INTEGER NOT NULL CHECK(size_bytes >= 0), media_type TEXT) STRICT"
);
define_byte_literal!(
    artifact_imports_table_sql,
    artifact_imports_table_sql_spec,
    b"CREATE TABLE artifact_imports(artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, source_path BLOB NOT NULL CHECK(length(source_path) > 0), PRIMARY KEY(artifact_id, source_path)) STRICT"
);
define_byte_literal!(
    artifact_migration_sql,
    artifact_migration_sql_spec,
    b"CREATE TABLE artifacts(id TEXT PRIMARY KEY CHECK(length(id) = 71), algorithm TEXT NOT NULL CHECK(algorithm = 'sha256'), digest TEXT NOT NULL UNIQUE CHECK(length(digest) = 64), size_bytes INTEGER NOT NULL CHECK(size_bytes >= 0), media_type TEXT) STRICT;\nCREATE TABLE artifact_imports(artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, source_path BLOB NOT NULL CHECK(length(source_path) > 0), PRIMARY KEY(artifact_id, source_path)) STRICT;\nPRAGMA user_version = 2;"
);
define_byte_literal!(
    artifact_migration_name,
    artifact_migration_name_spec,
    b"add-artifact-store"
);
define_byte_literal!(
    artifact_migration_checksum,
    artifact_migration_checksum_spec,
    b"sha256:cc2f9596d293417355a5cfc08e3ecd508bf2fca45c6e6ecacd302e507535e326"
);
define_byte_literal!(metadata_key, metadata_key_spec, b"format");
define_byte_literal!(
    metadata_value,
    metadata_value_spec,
    b"crucible-workspace-v1"
);
define_string_literal!(init_literal, init_literal_spec, ['i', 'n', 'i', 't']);
define_string_literal!(
    artifact_literal,
    artifact_literal_spec,
    ['a', 'r', 't', 'i', 'f', 'a', 'c', 't']
);
define_string_literal!(
    import_literal,
    import_literal_spec,
    ['i', 'm', 'p', 'o', 'r', 't']
);
define_string_literal!(
    verify_literal,
    verify_literal_spec,
    ['v', 'e', 'r', 'i', 'f', 'y']
);
define_string_literal!(
    current_directory_literal,
    current_directory_literal_spec,
    ['.']
);

verus! {

broadcast use vstd::string::group_string_axioms;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathKind {
    Missing,
    File,
    Directory,
    Symlink,
    Other,
}

pub open spec fn same_path_kind_spec(left: PathKind, right: PathKind) -> bool {
    left == right
}

#[expect(
    clippy::match_like_matches_macro,
    reason = "the exhaustive match is the executable witness Verus relates to spec equality"
)]
pub fn same_path_kind(left: PathKind, right: PathKind) -> (same: bool)
    ensures
        same == same_path_kind_spec(left, right),
{
    match (left, right) {
        (PathKind::Missing, PathKind::Missing)
        | (PathKind::File, PathKind::File)
        | (PathKind::Directory, PathKind::Directory)
        | (PathKind::Symlink, PathKind::Symlink)
        | (PathKind::Other, PathKind::Other) => true,
        _ => false,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationRecord {
    pub version: i64,
    pub name: Vec<u8>,
    pub checksum: Vec<u8>,
}

#[verifier::ext_equal]
pub struct MigrationRecordView {
    pub version: i64,
    pub name: Seq<u8>,
    pub checksum: Seq<u8>,
}

impl View for MigrationRecord {
    type V = MigrationRecordView;

    open spec fn view(&self) -> MigrationRecordView {
        MigrationRecordView { version: self.version, name: self.name@, checksum: self.checksum@ }
    }
}

impl DeepView for MigrationRecord {
    type V = MigrationRecordView;

    open spec fn deep_view(&self) -> MigrationRecordView {
        self@
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceMetadata {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[verifier::ext_equal]
pub struct WorkspaceMetadataView {
    pub key: Seq<u8>,
    pub value: Seq<u8>,
}

impl View for WorkspaceMetadata {
    type V = WorkspaceMetadataView;

    open spec fn view(&self) -> WorkspaceMetadataView {
        WorkspaceMetadataView { key: self.key@, value: self.value@ }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DatabaseSnapshot {
    pub application_id: i64,
    pub schema_version: i64,
    pub journal_mode: Vec<u8>,
    pub synchronous: i64,
    pub quick_check: Vec<u8>,
    pub schema_object_count: u64,
    pub migrations_table_kind: Vec<u8>,
    pub migrations_table_sql: Vec<u8>,
    pub metadata_table_kind: Vec<u8>,
    pub metadata_table_sql: Vec<u8>,
    pub artifacts_table_kind: Vec<u8>,
    pub artifacts_table_sql: Vec<u8>,
    pub artifact_imports_table_kind: Vec<u8>,
    pub artifact_imports_table_sql: Vec<u8>,
    pub migration_row_count: u64,
    pub migrations: Vec<MigrationRecord>,
    pub metadata_row_count: u64,
    pub metadata: Option<WorkspaceMetadata>,
}

#[verifier::ext_equal]
pub struct DatabaseSnapshotView {
    pub application_id: i64,
    pub schema_version: i64,
    pub journal_mode: Seq<u8>,
    pub synchronous: i64,
    pub quick_check: Seq<u8>,
    pub schema_object_count: u64,
    pub migrations_table_kind: Seq<u8>,
    pub migrations_table_sql: Seq<u8>,
    pub metadata_table_kind: Seq<u8>,
    pub metadata_table_sql: Seq<u8>,
    pub artifacts_table_kind: Seq<u8>,
    pub artifacts_table_sql: Seq<u8>,
    pub artifact_imports_table_kind: Seq<u8>,
    pub artifact_imports_table_sql: Seq<u8>,
    pub migration_row_count: u64,
    pub migrations: Seq<MigrationRecordView>,
    pub metadata_row_count: u64,
    pub metadata: Option<WorkspaceMetadataView>,
}

impl View for DatabaseSnapshot {
    type V = DatabaseSnapshotView;

    open spec fn view(&self) -> DatabaseSnapshotView {
        DatabaseSnapshotView {
            application_id: self.application_id,
            schema_version: self.schema_version,
            journal_mode: self.journal_mode@,
            synchronous: self.synchronous,
            quick_check: self.quick_check@,
            schema_object_count: self.schema_object_count,
            migrations_table_kind: self.migrations_table_kind@,
            migrations_table_sql: self.migrations_table_sql@,
            metadata_table_kind: self.metadata_table_kind@,
            metadata_table_sql: self.metadata_table_sql@,
            artifacts_table_kind: self.artifacts_table_kind@,
            artifacts_table_sql: self.artifacts_table_sql@,
            artifact_imports_table_kind: self.artifact_imports_table_kind@,
            artifact_imports_table_sql: self.artifact_imports_table_sql@,
            migration_row_count: self.migration_row_count,
            migrations: self.migrations.deep_view(),
            metadata_row_count: self.metadata_row_count,
            metadata: match &self.metadata {
                Some(record) => Some(record@),
                None => None,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub root_kind: PathKind,
    pub state_kind: PathKind,
    pub state_entry_count: u64,
    pub corpus_kind: PathKind,
    pub corpus_entry_count: u64,
    pub seeds_kind: PathKind,
    pub interesting_kind: PathKind,
    pub coverage_kind: PathKind,
    pub regression_kind: PathKind,
    pub minimized_kind: PathKind,
    pub findings_kind: PathKind,
    pub objects_kind: PathKind,
    pub runs_kind: PathKind,
    pub reports_kind: PathKind,
    pub database_kind: PathKind,
    pub database_wal_kind: PathKind,
    pub database_shm_kind: PathKind,
    pub database: Option<DatabaseSnapshot>,
}

#[verifier::ext_equal]
pub struct WorkspaceSnapshotView {
    pub root_kind: PathKind,
    pub state_kind: PathKind,
    pub state_entry_count: u64,
    pub corpus_kind: PathKind,
    pub corpus_entry_count: u64,
    pub seeds_kind: PathKind,
    pub interesting_kind: PathKind,
    pub coverage_kind: PathKind,
    pub regression_kind: PathKind,
    pub minimized_kind: PathKind,
    pub findings_kind: PathKind,
    pub objects_kind: PathKind,
    pub runs_kind: PathKind,
    pub reports_kind: PathKind,
    pub database_kind: PathKind,
    pub database_wal_kind: PathKind,
    pub database_shm_kind: PathKind,
    pub database: Option<DatabaseSnapshotView>,
}

impl View for WorkspaceSnapshot {
    type V = WorkspaceSnapshotView;

    open spec fn view(&self) -> WorkspaceSnapshotView {
        WorkspaceSnapshotView {
            root_kind: self.root_kind,
            state_kind: self.state_kind,
            state_entry_count: self.state_entry_count,
            corpus_kind: self.corpus_kind,
            corpus_entry_count: self.corpus_entry_count,
            seeds_kind: self.seeds_kind,
            interesting_kind: self.interesting_kind,
            coverage_kind: self.coverage_kind,
            regression_kind: self.regression_kind,
            minimized_kind: self.minimized_kind,
            findings_kind: self.findings_kind,
            objects_kind: self.objects_kind,
            runs_kind: self.runs_kind,
            reports_kind: self.reports_kind,
            database_kind: self.database_kind,
            database_wal_kind: self.database_wal_kind,
            database_shm_kind: self.database_shm_kind,
            database: match &self.database {
                Some(database) => Some(database@),
                None => None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitializationDecision {
    Create,
    MigrateV1,
    Reuse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitializationError {
    UnsafeRoot,
    OccupiedState,
    IncompatibleDatabase,
}

pub open spec fn migration_record_matches_spec(
    record: MigrationRecordView,
    version: i64,
    name: Seq<u8>,
    checksum: Seq<u8>,
) -> bool {
    record.version == version && record.name == name && record.checksum == checksum
}

pub open spec fn database_snapshot_has_common_identity_spec(
    snapshot: DatabaseSnapshotView,
) -> bool {
    snapshot.application_id == WORKSPACE_APPLICATION_ID && snapshot.journal_mode == wal_mode_spec()
        && snapshot.synchronous == 2 && snapshot.quick_check == quick_check_ok_spec()
        && snapshot.migrations_table_kind == table_kind_spec() && snapshot.migrations_table_sql
        == migration_table_sql_spec() && snapshot.metadata_table_kind == table_kind_spec()
        && snapshot.metadata_table_sql == metadata_table_sql_spec() && snapshot.metadata_row_count
        == 1 && snapshot.metadata == Some(
        WorkspaceMetadataView { key: metadata_key_spec(), value: metadata_value_spec() },
    )
}

pub open spec fn database_snapshot_is_exact_v1_spec(snapshot: DatabaseSnapshotView) -> bool {
    database_snapshot_has_common_identity_spec(snapshot) && snapshot.schema_version == 1
        && snapshot.schema_object_count == 2 && snapshot.artifacts_table_kind == Seq::<u8>::empty()
        && snapshot.artifacts_table_sql == Seq::<u8>::empty()
        && snapshot.artifact_imports_table_kind == Seq::<u8>::empty()
        && snapshot.artifact_imports_table_sql == Seq::<u8>::empty() && snapshot.migration_row_count
        == 1 && snapshot.migrations == seq![
        MigrationRecordView {
            version: 1,
            name: migration_name_spec(),
            checksum: migration_checksum_spec(),
        },
    ]
}

pub open spec fn database_snapshot_is_exact_spec(snapshot: DatabaseSnapshotView) -> bool {
    database_snapshot_has_common_identity_spec(snapshot) && snapshot.schema_version
        == WORKSPACE_SCHEMA_VERSION && snapshot.schema_object_count == 4
        && snapshot.artifacts_table_kind == table_kind_spec() && snapshot.artifacts_table_sql
        == artifacts_table_sql_spec() && snapshot.artifact_imports_table_kind == table_kind_spec()
        && snapshot.artifact_imports_table_sql == artifact_imports_table_sql_spec()
        && snapshot.migration_row_count == 2 && snapshot.migrations == seq![
        MigrationRecordView {
            version: 1,
            name: migration_name_spec(),
            checksum: migration_checksum_spec(),
        },
        MigrationRecordView {
            version: 2,
            name: artifact_migration_name_spec(),
            checksum: artifact_migration_checksum_spec(),
        },
    ]
}

fn bytes_equal(left: &[u8], right: &[u8]) -> (equal: bool)
    ensures
        equal == (left@ == right@),
{
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len()
        invariant
            index <= left@.len(),
            left@.len() == right@.len(),
            forall|prior: int| 0 <= prior < index ==> left@[prior] == right@[prior],
        decreases left.len() - index,
    {
        if left[index] != right[index] {
            assert(left@ != right@);
            return false;
        }
        index += 1;
    }
    proof {
        assert_seqs_equal!(left@ == right@, position => {
            assert(position < index);
        });
    }
    true
}

fn migration_record_matches(
    record: &MigrationRecord,
    version: i64,
    name: &[u8],
    checksum: &[u8],
) -> (matches: bool)
    ensures
        matches == migration_record_matches_spec(record@, version, name@, checksum@),
{
    record.version == version && bytes_equal(record.name.as_slice(), name) && bytes_equal(
        record.checksum.as_slice(),
        checksum,
    )
}

fn database_snapshot_has_common_identity(snapshot: &DatabaseSnapshot) -> (exact: bool)
    ensures
        exact == database_snapshot_has_common_identity_spec(snapshot@),
{
    reveal(database_snapshot_has_common_identity_spec);
    let wal = wal_mode();
    let quick = quick_check_ok();
    let table = table_kind();
    let migrations_sql = migration_table_sql();
    let metadata_sql = metadata_table_sql();
    let expected_metadata_key = metadata_key();
    let expected_metadata_value = metadata_value();
    if snapshot.application_id != WORKSPACE_APPLICATION_ID || !bytes_equal(
        snapshot.journal_mode.as_slice(),
        wal.as_slice(),
    ) || snapshot.synchronous != 2 || !bytes_equal(
        snapshot.quick_check.as_slice(),
        quick.as_slice(),
    ) || !bytes_equal(snapshot.migrations_table_kind.as_slice(), table.as_slice()) || !bytes_equal(
        snapshot.migrations_table_sql.as_slice(),
        migrations_sql.as_slice(),
    ) || !bytes_equal(snapshot.metadata_table_kind.as_slice(), table.as_slice()) || !bytes_equal(
        snapshot.metadata_table_sql.as_slice(),
        metadata_sql.as_slice(),
    ) || snapshot.metadata_row_count != 1 {
        return false;
    }
    let metadata = match &snapshot.metadata {
        Some(metadata) => metadata,
        None => return false,
    };
    bytes_equal(metadata.key.as_slice(), expected_metadata_key.as_slice()) && bytes_equal(
        metadata.value.as_slice(),
        expected_metadata_value.as_slice(),
    )
}

pub fn database_snapshot_is_exact_v1(snapshot: &DatabaseSnapshot) -> (exact: bool)
    ensures
        exact == database_snapshot_is_exact_v1_spec(snapshot@),
{
    reveal(database_snapshot_is_exact_v1_spec);
    reveal(database_snapshot_has_common_identity_spec);
    if !database_snapshot_has_common_identity(snapshot) || snapshot.schema_version != 1
        || snapshot.schema_object_count != 2 || !snapshot.artifacts_table_kind.is_empty()
        || !snapshot.artifacts_table_sql.is_empty()
        || !snapshot.artifact_imports_table_kind.is_empty()
        || !snapshot.artifact_imports_table_sql.is_empty() || snapshot.migration_row_count != 1
        || snapshot.migrations.len() != 1 {
        assert(!database_snapshot_is_exact_v1_spec(snapshot@)) by {
            if database_snapshot_is_exact_v1_spec(snapshot@) {
                assert(database_snapshot_has_common_identity_spec(snapshot@));
                assert(snapshot.migrations.deep_view().len() == 1);
                assert(snapshot.migrations.len() == snapshot.migrations.deep_view().len());
            }
        };
        return false;
    }
    let expected_name = migration_name();
    let expected_checksum = migration_checksum();
    let matches = migration_record_matches(
        &snapshot.migrations[0],
        1,
        expected_name.as_slice(),
        expected_checksum.as_slice(),
    );
    if !matches {
        assert(!database_snapshot_is_exact_v1_spec(snapshot@)) by {
            if database_snapshot_is_exact_v1_spec(snapshot@) {
                assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
                assert(migration_record_matches_spec(
                    snapshot.migrations[0]@,
                    1,
                    migration_name_spec(),
                    migration_checksum_spec(),
                ));
            }
        };
        return false;
    }
    assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
    assert(snapshot.migrations.deep_view() =~= seq![
        MigrationRecordView {
            version: 1,
            name: migration_name_spec(),
            checksum: migration_checksum_spec(),
        },
    ]);
    assert(database_snapshot_has_common_identity_spec(snapshot@));
    assert(snapshot.artifacts_table_kind@ == Seq::<u8>::empty());
    assert(snapshot.artifacts_table_sql@ == Seq::<u8>::empty());
    assert(snapshot.artifact_imports_table_kind@ == Seq::<u8>::empty());
    assert(snapshot.artifact_imports_table_sql@ == Seq::<u8>::empty());
    assert(database_snapshot_is_exact_v1_spec(snapshot@));
    true
}

pub fn database_snapshot_is_exact(snapshot: &DatabaseSnapshot) -> (exact: bool)
    ensures
        exact == database_snapshot_is_exact_spec(snapshot@),
{
    reveal(database_snapshot_is_exact_spec);
    reveal(database_snapshot_has_common_identity_spec);
    let table = table_kind();
    let artifacts_sql = artifacts_table_sql();
    let imports_sql = artifact_imports_table_sql();
    if !database_snapshot_has_common_identity(snapshot) || snapshot.schema_version
        != WORKSPACE_SCHEMA_VERSION || snapshot.schema_object_count != 4 || !bytes_equal(
        snapshot.artifacts_table_kind.as_slice(),
        table.as_slice(),
    ) || !bytes_equal(snapshot.artifacts_table_sql.as_slice(), artifacts_sql.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_kind.as_slice(), table.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_sql.as_slice(), imports_sql.as_slice())
        || snapshot.migration_row_count != 2 || snapshot.migrations.len() != 2 {
        assert(!database_snapshot_is_exact_spec(snapshot@)) by {
            if database_snapshot_is_exact_spec(snapshot@) {
                assert(database_snapshot_has_common_identity_spec(snapshot@));
                assert(snapshot.migrations.deep_view().len() == 2);
                assert(snapshot.migrations.len() == snapshot.migrations.deep_view().len());
            }
        };
        return false;
    }
    let first_name = migration_name();
    let first_checksum = migration_checksum();
    let second_name = artifact_migration_name();
    let second_checksum = artifact_migration_checksum();
    let first_matches = migration_record_matches(
        &snapshot.migrations[0],
        1,
        first_name.as_slice(),
        first_checksum.as_slice(),
    );
    let second_matches = migration_record_matches(
        &snapshot.migrations[1],
        2,
        second_name.as_slice(),
        second_checksum.as_slice(),
    );
    if !first_matches || !second_matches {
        assert(!database_snapshot_is_exact_spec(snapshot@)) by {
            if database_snapshot_is_exact_spec(snapshot@) {
                assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
                assert(snapshot.migrations.deep_view()[1] == snapshot.migrations[1]@);
                assert(migration_record_matches_spec(
                    snapshot.migrations[0]@,
                    1,
                    migration_name_spec(),
                    migration_checksum_spec(),
                ));
                assert(migration_record_matches_spec(
                    snapshot.migrations[1]@,
                    2,
                    artifact_migration_name_spec(),
                    artifact_migration_checksum_spec(),
                ));
            }
        };
        return false;
    }
    assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
    assert(snapshot.migrations.deep_view()[1] == snapshot.migrations[1]@);
    assert(snapshot.migrations.deep_view() =~= seq![
        MigrationRecordView {
            version: 1,
            name: migration_name_spec(),
            checksum: migration_checksum_spec(),
        },
        MigrationRecordView {
            version: 2,
            name: artifact_migration_name_spec(),
            checksum: artifact_migration_checksum_spec(),
        },
    ]);
    true
}

pub open spec fn workspace_layout_is_exact_spec(snapshot: WorkspaceSnapshotView) -> bool {
    snapshot.root_kind == PathKind::Directory && snapshot.state_kind == PathKind::Directory
        && snapshot.state_entry_count == 8 && snapshot.corpus_kind == PathKind::Directory
        && snapshot.corpus_entry_count == 5 && snapshot.seeds_kind == PathKind::Directory
        && snapshot.interesting_kind == PathKind::Directory && snapshot.coverage_kind
        == PathKind::Directory && snapshot.regression_kind == PathKind::Directory
        && snapshot.minimized_kind == PathKind::Directory && snapshot.findings_kind
        == PathKind::Directory && snapshot.objects_kind == PathKind::Directory && snapshot.runs_kind
        == PathKind::Directory && snapshot.reports_kind == PathKind::Directory
        && snapshot.database_kind == PathKind::File && snapshot.database_wal_kind == PathKind::File
        && snapshot.database_shm_kind == PathKind::File
}

pub fn workspace_layout_is_exact(snapshot: &WorkspaceSnapshot) -> (exact: bool)
    ensures
        exact == workspace_layout_is_exact_spec(snapshot@),
{
    reveal(workspace_layout_is_exact_spec);
    same_path_kind(snapshot.root_kind, PathKind::Directory) && same_path_kind(
        snapshot.state_kind,
        PathKind::Directory,
    ) && snapshot.state_entry_count == 8 && same_path_kind(
        snapshot.corpus_kind,
        PathKind::Directory,
    ) && snapshot.corpus_entry_count == 5 && same_path_kind(
        snapshot.seeds_kind,
        PathKind::Directory,
    ) && same_path_kind(snapshot.interesting_kind, PathKind::Directory) && same_path_kind(
        snapshot.coverage_kind,
        PathKind::Directory,
    ) && same_path_kind(snapshot.regression_kind, PathKind::Directory) && same_path_kind(
        snapshot.minimized_kind,
        PathKind::Directory,
    ) && same_path_kind(snapshot.findings_kind, PathKind::Directory) && same_path_kind(
        snapshot.objects_kind,
        PathKind::Directory,
    ) && same_path_kind(snapshot.runs_kind, PathKind::Directory) && same_path_kind(
        snapshot.reports_kind,
        PathKind::Directory,
    ) && same_path_kind(snapshot.database_kind, PathKind::File) && same_path_kind(
        snapshot.database_wal_kind,
        PathKind::File,
    ) && same_path_kind(snapshot.database_shm_kind, PathKind::File)
}

pub open spec fn decide_workspace_initialization_spec(snapshot: WorkspaceSnapshotView) -> Result<
    InitializationDecision,
    InitializationError,
> {
    if snapshot.root_kind != PathKind::Directory {
        Err(InitializationError::UnsafeRoot)
    } else if snapshot.state_kind == PathKind::Missing {
        Ok(InitializationDecision::Create)
    } else if workspace_layout_is_exact_spec(snapshot) {
        match snapshot.database {
            Some(database) => if database_snapshot_is_exact_spec(database) {
                Ok(InitializationDecision::Reuse)
            } else if database_snapshot_is_exact_v1_spec(database) {
                Ok(InitializationDecision::MigrateV1)
            } else {
                Err(InitializationError::IncompatibleDatabase)
            },
            None => Err(InitializationError::IncompatibleDatabase),
        }
    } else {
        Err(InitializationError::OccupiedState)
    }
}

pub fn decide_workspace_initialization(snapshot: &WorkspaceSnapshot) -> (decision: Result<
    InitializationDecision,
    InitializationError,
>)
    ensures
        decision == decide_workspace_initialization_spec(snapshot@),
{
    reveal(decide_workspace_initialization_spec);
    if !same_path_kind(snapshot.root_kind, PathKind::Directory) {
        Err(InitializationError::UnsafeRoot)
    } else if same_path_kind(snapshot.state_kind, PathKind::Missing) {
        Ok(InitializationDecision::Create)
    } else if workspace_layout_is_exact(snapshot) {
        match &snapshot.database {
            Some(database) => if database_snapshot_is_exact(database) {
                Ok(InitializationDecision::Reuse)
            } else if database_snapshot_is_exact_v1(database) {
                Ok(InitializationDecision::MigrateV1)
            } else {
                Err(InitializationError::IncompatibleDatabase)
            },
            None => Err(InitializationError::IncompatibleDatabase),
        }
    } else {
        Err(InitializationError::OccupiedState)
    }
}

pub proof fn lemma_exact_database_snapshot_is_unique(
    left: DatabaseSnapshot,
    right: DatabaseSnapshot,
)
    requires
        database_snapshot_is_exact_spec(left@),
        database_snapshot_is_exact_spec(right@),
    ensures
        left@ == right@,
{
    assert(left@ =~= right@);
}

pub proof fn lemma_database_profiles_are_disjoint(snapshot: DatabaseSnapshot)
    requires
        database_snapshot_is_exact_v1_spec(snapshot@),
    ensures
        !database_snapshot_is_exact_spec(snapshot@),
{
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliAction {
    Init(String),
    ArtifactImport(String, String),
    ArtifactVerify(String, String),
}

#[verifier::ext_equal]
pub enum CliActionView {
    Init(Seq<char>),
    ArtifactImport(Seq<char>, Seq<char>),
    ArtifactVerify(Seq<char>, Seq<char>),
}

impl View for CliAction {
    type V = CliActionView;

    open spec fn view(&self) -> CliActionView {
        match self {
            CliAction::Init(path) => CliActionView::Init(path@),
            CliAction::ArtifactImport(source, root) => CliActionView::ArtifactImport(
                source@,
                root@,
            ),
            CliAction::ArtifactVerify(id, root) => CliActionView::ArtifactVerify(id@, root@),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliParseError {
    UnsupportedArguments,
}

pub open spec fn parse_cli_args_spec(args: Seq<Seq<char>>) -> Result<CliActionView, CliParseError> {
    if args.len() == 1 && args[0] == init_literal_spec() {
        Ok(CliActionView::Init(current_directory_literal_spec()))
    } else if args.len() == 2 && args[0] == init_literal_spec() {
        Ok(CliActionView::Init(args[1]))
    } else if args.len() == 3 && args[0] == artifact_literal_spec() && args[1]
        == import_literal_spec() {
        Ok(CliActionView::ArtifactImport(args[2], current_directory_literal_spec()))
    } else if args.len() == 4 && args[0] == artifact_literal_spec() && args[1]
        == import_literal_spec() {
        Ok(CliActionView::ArtifactImport(args[2], args[3]))
    } else if args.len() == 3 && args[0] == artifact_literal_spec() && args[1]
        == verify_literal_spec() {
        Ok(CliActionView::ArtifactVerify(args[2], current_directory_literal_spec()))
    } else if args.len() == 4 && args[0] == artifact_literal_spec() && args[1]
        == verify_literal_spec() {
        Ok(CliActionView::ArtifactVerify(args[2], args[3]))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

pub fn parse_cli_args(args: &[String]) -> (result: Result<CliAction, CliParseError>)
    ensures
        match (&result, parse_cli_args_spec(args.deep_view())) {
            (Ok(action), Ok(expected)) => action@ == expected,
            (Err(error), Err(expected)) => *error == expected,
            _ => false,
        },
{
    let init = init_literal();
    if args.len() == 1 && args[0] == init {
        Ok(CliAction::Init(current_directory_literal()))
    } else if args.len() == 2 && args[0] == init {
        Ok(CliAction::Init(args[1].clone()))
    } else if args.len() == 3 && args[0] == artifact_literal() && args[1] == import_literal() {
        Ok(CliAction::ArtifactImport(args[2].clone(), current_directory_literal()))
    } else if args.len() == 4 && args[0] == artifact_literal() && args[1] == import_literal() {
        Ok(CliAction::ArtifactImport(args[2].clone(), args[3].clone()))
    } else if args.len() == 3 && args[0] == artifact_literal() && args[1] == verify_literal() {
        Ok(CliAction::ArtifactVerify(args[2].clone(), current_directory_literal()))
    } else if args.len() == 4 && args[0] == artifact_literal() && args[1] == verify_literal() {
        Ok(CliAction::ArtifactVerify(args[2].clone(), args[3].clone()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

} // verus!
