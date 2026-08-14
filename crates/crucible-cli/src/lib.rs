#![forbid(unsafe_code)]

mod artifact_store;
mod configuration;
mod inspection;
mod local_run;

pub use artifact_store::*;
pub use configuration::*;
pub use inspection::*;
pub use local_run::*;

#[expect(
    unused_imports,
    reason = "used by Verus proof code after ordinary Rust erasure"
)]
use vstd::assert_seqs_equal;
use vstd::prelude::*;

verus! {

pub const WORKSPACE_APPLICATION_ID: i64 = 0x4352_5543;

pub const WORKSPACE_SCHEMA_VERSION: i64 = 4;

pub const ENGINE_EVENT_SCHEMA_VERSION: u16 = 1;

pub const EVIDENCE_BUNDLE_SCHEMA_VERSION: u16 = 1;

pub const REPORT_SCHEMA_VERSION: u16 = 1;

pub const MAX_CLI_ARGUMENTS: usize = 5;

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
define_byte_literal!(
    run_migration_sql,
    run_migration_sql_spec,
    b"CREATE TABLE id_sequences(name TEXT PRIMARY KEY CHECK(length(name) > 0), next_value INTEGER NOT NULL CHECK(next_value > 0)) STRICT;\nCREATE TABLE target_builds(id TEXT PRIMARY KEY CHECK(length(id) > 0), target_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, manifest_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, identity_digest TEXT NOT NULL CHECK(length(identity_digest) = 71)) STRICT;\nCREATE TABLE capability_manifests(artifact_id TEXT PRIMARY KEY REFERENCES artifacts(id) ON DELETE RESTRICT, backend TEXT NOT NULL CHECK(length(backend) > 0), platform TEXT NOT NULL CHECK(length(platform) > 0)) STRICT;\nCREATE TABLE runs(id TEXT PRIMARY KEY CHECK(length(id) > 0), configuration_source_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, effective_configuration_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, configuration_digest TEXT NOT NULL CHECK(length(configuration_digest) = 71), target_build_id TEXT REFERENCES target_builds(id) ON DELETE RESTRICT, capability_manifest_artifact_id TEXT NOT NULL REFERENCES capability_manifests(artifact_id) ON DELETE RESTRICT, seed TEXT NOT NULL CHECK(length(seed) > 0)) STRICT;\nCREATE TABLE run_attempts(id TEXT PRIMARY KEY CHECK(length(id) > 0), run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE RESTRICT, ordinal INTEGER NOT NULL CHECK(ordinal > 0), status TEXT NOT NULL CHECK(status IN ('reserved','target_prepared','observed','harness_failure')), UNIQUE(run_id, ordinal)) STRICT;\nCREATE TABLE run_effective_controls(run_id TEXT PRIMARY KEY REFERENCES runs(id) ON DELETE RESTRICT, timeout_ms TEXT NOT NULL CHECK(length(timeout_ms) > 0), memory_bytes TEXT NOT NULL CHECK(length(memory_bytes) > 0), max_processes TEXT NOT NULL CHECK(length(max_processes) > 0), max_stream_bytes TEXT NOT NULL CHECK(length(max_stream_bytes) > 0), network_policy TEXT NOT NULL CHECK(network_policy IN ('none','unrestricted-host')), isolation_backend TEXT NOT NULL CHECK(isolation_backend = 'linux-bubblewrap-prlimit-v1'), output_capture_status TEXT NOT NULL CHECK(output_capture_status = 'drain-and-discard')) STRICT;\nCREATE TABLE observations(id TEXT PRIMARY KEY CHECK(length(id) > 0), attempt_id TEXT NOT NULL UNIQUE REFERENCES run_attempts(id) ON DELETE RESTRICT, observation_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, stdout_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, stderr_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, completion_tag INTEGER NOT NULL CHECK(completion_tag > 0), termination_tag INTEGER NOT NULL CHECK(termination_tag > 0)) STRICT;\nCREATE TABLE harness_failures(attempt_id TEXT PRIMARY KEY REFERENCES run_attempts(id) ON DELETE RESTRICT, kind TEXT NOT NULL CHECK(length(kind) > 0), detail_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT) STRICT;\nPRAGMA user_version = 3;"
);
define_byte_literal!(
    run_migration_name,
    run_migration_name_spec,
    b"add-local-run-evidence"
);
define_byte_literal!(
    run_migration_checksum,
    run_migration_checksum_spec,
    b"sha256:1b51cfb89d23f8af6dc67a7c149e3455f888e2f74c280873213c083d9693a21b"
);
define_byte_literal!(
    run_schema_digest,
    run_schema_digest_spec,
    b"sha256:4800eb0d8872dc940eaed3bc590a26605db2e089b5aa1c356c4c70d2c7dda05e"
);
define_byte_literal!(
    domain_migration_sql,
    domain_migration_sql_spec,
    br#"CREATE TABLE projects(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), name TEXT NOT NULL UNIQUE CHECK(length(name) BETWEEN 1 AND 4096), configuration_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE targets(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT, adapter TEXT NOT NULL CHECK(length(adapter) BETWEEN 1 AND 4096), configuration_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE source_snapshots(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT, artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, identity_digest TEXT NOT NULL CHECK(length(identity_digest) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE build_recipes(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), target_id TEXT NOT NULL REFERENCES targets(id) ON DELETE RESTRICT, source_snapshot_id TEXT REFERENCES source_snapshots(id) ON DELETE RESTRICT, recipe_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, identity_digest TEXT NOT NULL CHECK(length(identity_digest) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE build_executions(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), build_recipe_id TEXT NOT NULL REFERENCES build_recipes(id) ON DELETE RESTRICT, status TEXT NOT NULL CHECK(status IN ('reserved','running','succeeded','failed','cancelled')), log_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, output_target_build_id TEXT REFERENCES target_builds(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE deployments(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), target_build_id TEXT NOT NULL REFERENCES target_builds(id) ON DELETE RESTRICT, manifest_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, capability_manifest_artifact_id TEXT NOT NULL REFERENCES capability_manifests(artifact_id) ON DELETE RESTRICT, status TEXT NOT NULL CHECK(status IN ('ready','active','retired','failed'))) STRICT;
CREATE TABLE campaigns(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT, configuration_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, retention_policy TEXT NOT NULL CHECK(retention_policy IN ('retain-every-run','aggregate-checkpoints')), status TEXT NOT NULL CHECK(status IN ('created','active','completed','cancelled','failed')), campaign_seed TEXT NOT NULL CHECK(length(campaign_seed) BETWEEN 1 AND 4096), scheduling_seed TEXT NOT NULL CHECK(length(scheduling_seed) BETWEEN 1 AND 4096), fault_seed TEXT NOT NULL CHECK(length(fault_seed) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE experiments(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE RESTRICT, kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 4096), experiment_seed TEXT NOT NULL CHECK(length(experiment_seed) BETWEEN 1 AND 4096), status TEXT NOT NULL CHECK(status IN ('created','active','completed','cancelled','failed'))) STRICT;
CREATE TABLE scenarios(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE RESTRICT, definition_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, status TEXT NOT NULL CHECK(status IN ('created','active','completed','cancelled','failed'))) STRICT;
CREATE TABLE scenario_participants(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), scenario_id TEXT NOT NULL REFERENCES scenarios(id) ON DELETE RESTRICT, target_build_id TEXT NOT NULL REFERENCES target_builds(id) ON DELETE RESTRICT, role TEXT NOT NULL CHECK(length(role) BETWEEN 1 AND 4096), UNIQUE(scenario_id, role)) STRICT;
CREATE TABLE scenario_steps(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), scenario_id TEXT NOT NULL REFERENCES scenarios(id) ON DELETE RESTRICT, participant_id TEXT REFERENCES scenario_participants(id) ON DELETE RESTRICT, ordinal INTEGER NOT NULL CHECK(ordinal >= 0), action_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, UNIQUE(scenario_id, ordinal), UNIQUE(scenario_id, id)) STRICT;
CREATE TABLE scenario_edges(scenario_id TEXT NOT NULL REFERENCES scenarios(id) ON DELETE RESTRICT, from_step_id TEXT NOT NULL, to_step_id TEXT NOT NULL, relation TEXT NOT NULL CHECK(relation IN ('before','data','cancel','fault')), PRIMARY KEY(scenario_id, from_step_id, to_step_id, relation), CHECK(from_step_id <> to_step_id), FOREIGN KEY(scenario_id, from_step_id) REFERENCES scenario_steps(scenario_id, id) ON DELETE RESTRICT, FOREIGN KEY(scenario_id, to_step_id) REFERENCES scenario_steps(scenario_id, id) ON DELETE RESTRICT) STRICT;
CREATE TABLE oracle_verdicts(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), attempt_id TEXT NOT NULL REFERENCES run_attempts(id) ON DELETE RESTRICT, oracle_id TEXT NOT NULL CHECK(length(oracle_id) BETWEEN 1 AND 4096), verdict TEXT NOT NULL CHECK(verdict IN ('pass','fail','inconclusive','error')), facts_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, hypothesis_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE findings(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT, kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 4096), status TEXT NOT NULL CHECK(status IN ('open','confirmed','fixed','regressed','closed')), canonical_predicate_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE finding_instances(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE RESTRICT, run_attempt_id TEXT NOT NULL REFERENCES run_attempts(id) ON DELETE RESTRICT, observation_id TEXT REFERENCES observations(id) ON DELETE RESTRICT, predicate_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, is_original INTEGER NOT NULL CHECK(is_original IN (0,1))) STRICT;
CREATE TABLE finding_campaigns(finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE RESTRICT, campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE RESTRICT, first_instance_id TEXT NOT NULL REFERENCES finding_instances(id) ON DELETE RESTRICT, PRIMARY KEY(finding_id, campaign_id)) STRICT;
CREATE TABLE finding_transitions(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE RESTRICT, prior_status TEXT NOT NULL CHECK(length(prior_status) BETWEEN 1 AND 4096), next_status TEXT NOT NULL CHECK(length(next_status) BETWEEN 1 AND 4096), evidence_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, ordinal INTEGER NOT NULL CHECK(ordinal > 0), UNIQUE(finding_id, ordinal), CHECK(prior_status <> next_status)) STRICT;
CREATE TABLE evidence_nodes(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 4096), artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, schema_identity TEXT NOT NULL CHECK(length(schema_identity) BETWEEN 1 AND 4096), producer_identity TEXT NOT NULL CHECK(length(producer_identity) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE provenance_edges(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), from_node_id TEXT NOT NULL REFERENCES evidence_nodes(id) ON DELETE RESTRICT, to_node_id TEXT NOT NULL REFERENCES evidence_nodes(id) ON DELETE RESTRICT, relation TEXT NOT NULL CHECK(length(relation) BETWEEN 1 AND 4096), transformation_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, CHECK(from_node_id <> to_node_id)) STRICT;
CREATE TABLE coverage_records(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), attempt_id TEXT NOT NULL REFERENCES run_attempts(id) ON DELETE RESTRICT, provider_identity TEXT NOT NULL CHECK(length(provider_identity) BETWEEN 1 AND 4096), coverage_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, new_features INTEGER NOT NULL CHECK(new_features >= 0)) STRICT;
CREATE TABLE corpus_entries(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE RESTRICT, input_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, ancestry_node_id TEXT REFERENCES evidence_nodes(id) ON DELETE RESTRICT, state TEXT NOT NULL CHECK(state IN ('seed','interesting','coverage','regression','minimized','retired')), UNIQUE(campaign_id, input_artifact_id)) STRICT;
CREATE TABLE patches(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE RESTRICT, patch_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, producer_identity TEXT NOT NULL CHECK(length(producer_identity) BETWEEN 1 AND 4096), status TEXT NOT NULL CHECK(status IN ('candidate','verified','rejected'))) STRICT;
CREATE TABLE verification_runs(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), patch_id TEXT NOT NULL REFERENCES patches(id) ON DELETE RESTRICT, status TEXT NOT NULL CHECK(status IN ('reserved','running','passed','failed','inconclusive','error')), evidence_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, seed TEXT NOT NULL CHECK(length(seed) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE proof_artifacts(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), verification_run_id TEXT NOT NULL REFERENCES verification_runs(id) ON DELETE RESTRICT, artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, proof_kind TEXT NOT NULL CHECK(length(proof_kind) BETWEEN 1 AND 4096), trusted_boundary_digest TEXT NOT NULL CHECK(length(trusted_boundary_digest) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE trusted_boundaries(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), component TEXT NOT NULL CHECK(length(component) BETWEEN 1 AND 4096), kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 4096), assumption_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, approval_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, source_digest TEXT NOT NULL CHECK(length(source_digest) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE plugin_identities(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), manifest_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, capability_manifest_artifact_id TEXT NOT NULL REFERENCES capability_manifests(artifact_id) ON DELETE RESTRICT, implementation_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, status TEXT NOT NULL CHECK(status IN ('enabled','disabled','rejected'))) STRICT;
CREATE TABLE engine_stats(campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE RESTRICT, epoch INTEGER NOT NULL CHECK(epoch >= 0), engine_class TEXT NOT NULL CHECK(engine_class IN ('coverage-fuzzing','property-testing','stateful-testing','metamorphic-testing','fault-injection','symbolic-testing','miscellaneous')), engine_seed TEXT NOT NULL CHECK(length(engine_seed) BETWEEN 1 AND 4096), executions INTEGER NOT NULL CHECK(executions >= 0), cpu_seconds INTEGER NOT NULL CHECK(cpu_seconds >= 0), cpu_nanoseconds INTEGER NOT NULL CHECK(cpu_nanoseconds BETWEEN 0 AND 999999999), new_coverage INTEGER NOT NULL CHECK(new_coverage >= 0), new_findings INTEGER NOT NULL CHECK(new_findings >= 0), unique_states INTEGER NOT NULL CHECK(unique_states >= 0), minimized_findings INTEGER NOT NULL CHECK(minimized_findings >= 0), mutation_score_improvement INTEGER NOT NULL CHECK(mutation_score_improvement >= 0), new_oracle_failures INTEGER NOT NULL CHECK(new_oracle_failures >= 0), corpus_quality_improvement INTEGER NOT NULL CHECK(corpus_quality_improvement >= 0), provenance_credit INTEGER NOT NULL CHECK(provenance_credit >= 0), allocated_slots INTEGER NOT NULL CHECK(allocated_slots >= 0), PRIMARY KEY(campaign_id, epoch, engine_class)) STRICT;
CREATE TABLE run_replay_metadata(run_id TEXT PRIMARY KEY REFERENCES runs(id) ON DELETE RESTRICT, schema_version INTEGER NOT NULL CHECK(schema_version = 1), campaign_seed TEXT NOT NULL CHECK(length(campaign_seed) BETWEEN 1 AND 4096), engine_seed TEXT NOT NULL CHECK(length(engine_seed) BETWEEN 1 AND 4096), experiment_seed TEXT NOT NULL CHECK(length(experiment_seed) BETWEEN 1 AND 4096), scheduling_seed TEXT NOT NULL CHECK(length(scheduling_seed) BETWEEN 1 AND 4096), fault_seed TEXT NOT NULL CHECK(length(fault_seed) BETWEEN 1 AND 4096), engine_seed_status TEXT NOT NULL CHECK(engine_seed_status IN ('supported','unavailable')), engine_checkpoint_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, generated_schedule_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, fault_trace_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, environment_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, failure_predicate_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, engine_version TEXT NOT NULL CHECK(length(engine_version) BETWEEN 1 AND 4096)) STRICT;
CREATE TABLE reproduction_samples(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE RESTRICT, promise TEXT NOT NULL CHECK(promise IN ('finding','experiment','campaign')), attempt_count INTEGER NOT NULL CHECK(attempt_count BETWEEN 1 AND 1000000), observed_failures INTEGER NOT NULL CHECK(observed_failures BETWEEN 0 AND attempt_count), environment_equivalent INTEGER NOT NULL CHECK(environment_equivalent IN (0,1)), schedule_replayed_exactly INTEGER NOT NULL CHECK(schedule_replayed_exactly IN (0,1)), fault_trace_replayed_exactly INTEGER NOT NULL CHECK(fault_trace_replayed_exactly IN (0,1)), classification TEXT NOT NULL CHECK(classification IN ('stable-under-recorded-controls','intermittent-under-recorded-controls','not-observed-in-sample')), evidence_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE minimization_steps(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE RESTRICT, ordinal INTEGER NOT NULL CHECK(ordinal > 0), candidate_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, predicate_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, verdict TEXT NOT NULL CHECK(verdict IN ('accepted','rejected','inconclusive','error')), UNIQUE(finding_id, ordinal)) STRICT;
CREATE TABLE storage_generations(id INTEGER PRIMARY KEY CHECK(id > 0), status TEXT NOT NULL CHECK(status IN ('open','sealed','collecting','collected')), created_epoch INTEGER NOT NULL CHECK(created_epoch >= 0)) STRICT;
CREATE TABLE storage_leases(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), generation_id INTEGER NOT NULL REFERENCES storage_generations(id) ON DELETE RESTRICT, artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, owner_identity TEXT NOT NULL CHECK(length(owner_identity) BETWEEN 1 AND 4096), status TEXT NOT NULL CHECK(status IN ('active','released')), expires_epoch INTEGER NOT NULL CHECK(expires_epoch >= 0)) STRICT;
CREATE TABLE artifact_roots(artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, root_kind TEXT NOT NULL CHECK(root_kind IN ('original-finding','regression','active-campaign','evidence-bundle','manual')), root_id TEXT NOT NULL CHECK(length(root_id) BETWEEN 1 AND 4096), generation_id INTEGER NOT NULL REFERENCES storage_generations(id) ON DELETE RESTRICT, PRIMARY KEY(artifact_id, root_kind, root_id)) STRICT;
CREATE TABLE persistence_batches(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), campaign_id TEXT REFERENCES campaigns(id) ON DELETE RESTRICT, retention_policy TEXT NOT NULL CHECK(retention_policy IN ('retain-every-run','aggregate-checkpoints')), status TEXT NOT NULL CHECK(status IN ('open','committing','committed','failed')), item_count INTEGER NOT NULL CHECK(item_count BETWEEN 0 AND 4096), encoded_bytes INTEGER NOT NULL CHECK(encoded_bytes BETWEEN 0 AND 67108864), generation_id INTEGER NOT NULL REFERENCES storage_generations(id) ON DELETE RESTRICT) STRICT;
CREATE TABLE evidence_bundles(id TEXT PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 4096), manifest_artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE RESTRICT, signature_artifact_id TEXT REFERENCES artifacts(id) ON DELETE RESTRICT, signature_algorithm TEXT, generation_id INTEGER NOT NULL REFERENCES storage_generations(id) ON DELETE RESTRICT, CHECK((signature_artifact_id IS NULL AND signature_algorithm IS NULL) OR (signature_artifact_id IS NOT NULL AND length(signature_algorithm) BETWEEN 1 AND 4096))) STRICT;
PRAGMA user_version = 4;"#
);
define_byte_literal!(
    domain_migration_name,
    domain_migration_name_spec,
    b"add-domain-storage-model"
);
define_byte_literal!(
    domain_migration_checksum,
    domain_migration_checksum_spec,
    b"sha256:44207b6f24d5c73d5dbff923a6402d911be34d2e3d64e2fbd7d427302b491c96"
);
define_byte_literal!(
    domain_schema_digest,
    domain_schema_digest_spec,
    b"sha256:4ad91194bdb8f3865e36c94f4d9c762dcb3dc4e9ba2602f49947ca165599a304"
);
define_byte_literal!(metadata_key, metadata_key_spec, b"format");
define_byte_literal!(
    metadata_value,
    metadata_value_spec,
    b"crucible-workspace-v1"
);
define_string_literal!(init_literal, init_literal_spec, ['i', 'n', 'i', 't']);
define_string_literal!(run_literal, run_literal_spec, ['r', 'u', 'n']);
define_string_literal!(build_literal, build_literal_spec, ['b', 'u', 'i', 'l', 'd']);
define_string_literal!(fuzz_literal, fuzz_literal_spec, ['f', 'u', 'z', 'z']);
define_string_literal!(
    replay_literal,
    replay_literal_spec,
    ['r', 'e', 'p', 'l', 'a', 'y']
);
define_string_literal!(
    minimize_literal,
    minimize_literal_spec,
    ['m', 'i', 'n', 'i', 'm', 'i', 'z', 'e']
);
define_string_literal!(
    findings_literal,
    findings_literal_spec,
    ['f', 'i', 'n', 'd', 'i', 'n', 'g', 's']
);
define_string_literal!(
    report_literal,
    report_literal_spec,
    ['r', 'e', 'p', 'o', 'r', 't']
);
define_string_literal!(
    capabilities_literal,
    capabilities_literal_spec,
    ['c', 'a', 'p', 'a', 'b', 'i', 'l', 'i', 't', 'i', 'e', 's']
);
define_string_literal!(proof_literal, proof_literal_spec, ['p', 'r', 'o', 'o', 'f']);
define_string_literal!(tcb_literal, tcb_literal_spec, ['t', 'c', 'b']);
define_string_literal!(
    plugins_literal,
    plugins_literal_spec,
    ['p', 'l', 'u', 'g', 'i', 'n', 's']
);
define_string_literal!(
    inspect_literal,
    inspect_literal_spec,
    ['i', 'n', 's', 'p', 'e', 'c', 't']
);
define_string_literal!(check_literal, check_literal_spec, ['c', 'h', 'e', 'c', 'k']);
define_string_literal!(gc_literal, gc_literal_spec, ['g', 'c']);
define_string_literal!(
    patch_option_literal,
    patch_option_literal_spec,
    ['-', '-', 'p', 'a', 't', 'c', 'h']
);
define_string_literal!(
    format_option_literal,
    format_option_literal_spec,
    ['-', '-', 'f', 'o', 'r', 'm', 'a', 't']
);
define_string_literal!(human_literal, human_literal_spec, ['h', 'u', 'm', 'a', 'n']);
define_string_literal!(json_literal, json_literal_spec, ['j', 's', 'o', 'n']);
define_string_literal!(jsonl_literal, jsonl_literal_spec, ['j', 's', 'o', 'n', 'l']);
define_string_literal!(sarif_literal, sarif_literal_spec, ['s', 'a', 'r', 'i', 'f']);
define_string_literal!(junit_literal, junit_literal_spec, ['j', 'u', 'n', 'i', 't']);
define_string_literal!(
    evidence_literal,
    evidence_literal_spec,
    ['e', 'v', 'i', 'd', 'e', 'n', 'c', 'e']
);
define_string_literal!(
    bundle_literal,
    bundle_literal_spec,
    ['b', 'u', 'n', 'd', 'l', 'e']
);
define_string_literal!(
    internal_local_supervisor_literal,
    internal_local_supervisor_literal_spec,
    [
        '_', '_', 'c', 'r', 'u', 'c', 'i', 'b', 'l', 'e', '-', 'i', 'n', 't', 'e', 'r', 'n', 'a',
        'l', '-', 'l', 'o', 'c', 'a', 'l', '-', 's', 'u', 'p', 'e', 'r', 'v', 'i', 's', 'o', 'r',
        '-', 'v', '1'
    ]
);
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
    config_literal,
    config_literal_spec,
    ['c', 'o', 'n', 'f', 'i', 'g']
);
define_string_literal!(
    validate_literal,
    validate_literal_spec,
    ['v', 'a', 'l', 'i', 'd', 'a', 't', 'e']
);
define_string_literal!(
    canonicalize_literal,
    canonicalize_literal_spec,
    ['c', 'a', 'n', 'o', 'n', 'i', 'c', 'a', 'l', 'i', 'z', 'e']
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
    pub run_schema_digest: Vec<u8>,
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
    pub run_schema_digest: Seq<u8>,
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
            run_schema_digest: self.run_schema_digest@,
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
    MigrateV2,
    MigrateV3,
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
        && snapshot.artifact_imports_table_sql == Seq::<u8>::empty() && snapshot.run_schema_digest
        == Seq::<u8>::empty() && snapshot.migration_row_count == 1 && snapshot.migrations == seq![
        MigrationRecordView {
            version: 1,
            name: migration_name_spec(),
            checksum: migration_checksum_spec(),
        },
    ]
}

pub open spec fn database_snapshot_is_exact_v2_spec(snapshot: DatabaseSnapshotView) -> bool {
    database_snapshot_has_common_identity_spec(snapshot) && snapshot.schema_version == 2
        && snapshot.schema_object_count == 4 && snapshot.artifacts_table_kind == table_kind_spec()
        && snapshot.artifacts_table_sql == artifacts_table_sql_spec()
        && snapshot.artifact_imports_table_kind == table_kind_spec()
        && snapshot.artifact_imports_table_sql == artifact_imports_table_sql_spec()
        && snapshot.run_schema_digest == Seq::<u8>::empty() && snapshot.migration_row_count == 2
        && snapshot.migrations == seq![
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

pub open spec fn database_snapshot_is_exact_v3_spec(snapshot: DatabaseSnapshotView) -> bool {
    database_snapshot_has_common_identity_spec(snapshot) && snapshot.schema_version == 3
        && snapshot.schema_object_count == 12 && snapshot.artifacts_table_kind == table_kind_spec()
        && snapshot.artifacts_table_sql == artifacts_table_sql_spec()
        && snapshot.artifact_imports_table_kind == table_kind_spec()
        && snapshot.artifact_imports_table_sql == artifact_imports_table_sql_spec()
        && snapshot.run_schema_digest == run_schema_digest_spec() && snapshot.migration_row_count
        == 3 && snapshot.migrations == seq![
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
        MigrationRecordView {
            version: 3,
            name: run_migration_name_spec(),
            checksum: run_migration_checksum_spec(),
        },
    ]
}

pub open spec fn database_snapshot_is_exact_spec(snapshot: DatabaseSnapshotView) -> bool {
    database_snapshot_has_common_identity_spec(snapshot) && snapshot.schema_version
        == WORKSPACE_SCHEMA_VERSION && snapshot.schema_object_count == 47
        && snapshot.artifacts_table_kind == table_kind_spec() && snapshot.artifacts_table_sql
        == artifacts_table_sql_spec() && snapshot.artifact_imports_table_kind == table_kind_spec()
        && snapshot.artifact_imports_table_sql == artifact_imports_table_sql_spec()
        && snapshot.run_schema_digest == domain_schema_digest_spec() && snapshot.migration_row_count
        == 4 && snapshot.migrations == seq![
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
        MigrationRecordView {
            version: 3,
            name: run_migration_name_spec(),
            checksum: run_migration_checksum_spec(),
        },
        MigrationRecordView {
            version: 4,
            name: domain_migration_name_spec(),
            checksum: domain_migration_checksum_spec(),
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
        || !snapshot.artifact_imports_table_sql.is_empty() || !snapshot.run_schema_digest.is_empty()
        || snapshot.migration_row_count != 1 || snapshot.migrations.len() != 1 {
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
    assert(snapshot.run_schema_digest@ == Seq::<u8>::empty());
    assert(database_snapshot_is_exact_v1_spec(snapshot@));
    true
}

pub fn database_snapshot_is_exact_v2(snapshot: &DatabaseSnapshot) -> (exact: bool)
    ensures
        exact == database_snapshot_is_exact_v2_spec(snapshot@),
{
    reveal(database_snapshot_is_exact_v2_spec);
    reveal(database_snapshot_has_common_identity_spec);
    let table = table_kind();
    let artifacts_sql = artifacts_table_sql();
    let imports_sql = artifact_imports_table_sql();
    if !database_snapshot_has_common_identity(snapshot) || snapshot.schema_version != 2
        || snapshot.schema_object_count != 4 || !bytes_equal(
        snapshot.artifacts_table_kind.as_slice(),
        table.as_slice(),
    ) || !bytes_equal(snapshot.artifacts_table_sql.as_slice(), artifacts_sql.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_kind.as_slice(), table.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_sql.as_slice(), imports_sql.as_slice())
        || !snapshot.run_schema_digest.is_empty() || snapshot.migration_row_count != 2
        || snapshot.migrations.len() != 2 {
        assert(!database_snapshot_is_exact_v2_spec(snapshot@)) by {
            if database_snapshot_is_exact_v2_spec(snapshot@) {
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
        assert(!database_snapshot_is_exact_v2_spec(snapshot@)) by {
            if database_snapshot_is_exact_v2_spec(snapshot@) {
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
    assert(snapshot.run_schema_digest@ == Seq::<u8>::empty());
    true
}

pub fn database_snapshot_is_exact_v3(snapshot: &DatabaseSnapshot) -> (exact: bool)
    ensures
        exact == database_snapshot_is_exact_v3_spec(snapshot@),
{
    reveal(database_snapshot_is_exact_v3_spec);
    reveal(database_snapshot_has_common_identity_spec);
    let table = table_kind();
    let artifacts_sql = artifacts_table_sql();
    let imports_sql = artifact_imports_table_sql();
    let expected_run_schema = run_schema_digest();
    if !database_snapshot_has_common_identity(snapshot) || snapshot.schema_version != 3
        || snapshot.schema_object_count != 12 || !bytes_equal(
        snapshot.artifacts_table_kind.as_slice(),
        table.as_slice(),
    ) || !bytes_equal(snapshot.artifacts_table_sql.as_slice(), artifacts_sql.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_kind.as_slice(), table.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_sql.as_slice(), imports_sql.as_slice())
        || !bytes_equal(snapshot.run_schema_digest.as_slice(), expected_run_schema.as_slice())
        || snapshot.migration_row_count != 3 || snapshot.migrations.len() != 3 {
        assert(!database_snapshot_is_exact_v3_spec(snapshot@)) by {
            if database_snapshot_is_exact_v3_spec(snapshot@) {
                assert(database_snapshot_has_common_identity_spec(snapshot@));
                assert(snapshot.migrations.deep_view().len() == 3);
                assert(snapshot.migrations.len() == snapshot.migrations.deep_view().len());
            }
        };
        return false;
    }
    let first_name = migration_name();
    let first_checksum = migration_checksum();
    let second_name = artifact_migration_name();
    let second_checksum = artifact_migration_checksum();
    let third_name = run_migration_name();
    let third_checksum = run_migration_checksum();
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
    let third_matches = migration_record_matches(
        &snapshot.migrations[2],
        3,
        third_name.as_slice(),
        third_checksum.as_slice(),
    );
    if !first_matches || !second_matches || !third_matches {
        assert(!database_snapshot_is_exact_v3_spec(snapshot@)) by {
            if database_snapshot_is_exact_v3_spec(snapshot@) {
                assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
                assert(snapshot.migrations.deep_view()[1] == snapshot.migrations[1]@);
                assert(snapshot.migrations.deep_view()[2] == snapshot.migrations[2]@);
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
                assert(migration_record_matches_spec(
                    snapshot.migrations[2]@,
                    3,
                    run_migration_name_spec(),
                    run_migration_checksum_spec(),
                ));
            }
        };
        return false;
    }
    assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
    assert(snapshot.migrations.deep_view()[1] == snapshot.migrations[1]@);
    assert(snapshot.migrations.deep_view()[2] == snapshot.migrations[2]@);
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
        MigrationRecordView {
            version: 3,
            name: run_migration_name_spec(),
            checksum: run_migration_checksum_spec(),
        },
    ]);
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
    let expected_domain_schema = domain_schema_digest();
    if !database_snapshot_has_common_identity(snapshot) || snapshot.schema_version
        != WORKSPACE_SCHEMA_VERSION || snapshot.schema_object_count != 47 || !bytes_equal(
        snapshot.artifacts_table_kind.as_slice(),
        table.as_slice(),
    ) || !bytes_equal(snapshot.artifacts_table_sql.as_slice(), artifacts_sql.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_kind.as_slice(), table.as_slice())
        || !bytes_equal(snapshot.artifact_imports_table_sql.as_slice(), imports_sql.as_slice())
        || !bytes_equal(snapshot.run_schema_digest.as_slice(), expected_domain_schema.as_slice())
        || snapshot.migration_row_count != 4 || snapshot.migrations.len() != 4 {
        assert(!database_snapshot_is_exact_spec(snapshot@)) by {
            if database_snapshot_is_exact_spec(snapshot@) {
                assert(database_snapshot_has_common_identity_spec(snapshot@));
                assert(snapshot.migrations.deep_view().len() == 4);
                assert(snapshot.migrations.len() == snapshot.migrations.deep_view().len());
            }
        };
        return false;
    }
    let first_name = migration_name();
    let first_checksum = migration_checksum();
    let second_name = artifact_migration_name();
    let second_checksum = artifact_migration_checksum();
    let third_name = run_migration_name();
    let third_checksum = run_migration_checksum();
    let fourth_name = domain_migration_name();
    let fourth_checksum = domain_migration_checksum();
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
    let third_matches = migration_record_matches(
        &snapshot.migrations[2],
        3,
        third_name.as_slice(),
        third_checksum.as_slice(),
    );
    let fourth_matches = migration_record_matches(
        &snapshot.migrations[3],
        4,
        fourth_name.as_slice(),
        fourth_checksum.as_slice(),
    );
    if !first_matches || !second_matches || !third_matches || !fourth_matches {
        assert(!database_snapshot_is_exact_spec(snapshot@)) by {
            if database_snapshot_is_exact_spec(snapshot@) {
                assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
                assert(snapshot.migrations.deep_view()[1] == snapshot.migrations[1]@);
                assert(snapshot.migrations.deep_view()[2] == snapshot.migrations[2]@);
                assert(snapshot.migrations.deep_view()[3] == snapshot.migrations[3]@);
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
                assert(migration_record_matches_spec(
                    snapshot.migrations[2]@,
                    3,
                    run_migration_name_spec(),
                    run_migration_checksum_spec(),
                ));
                assert(migration_record_matches_spec(
                    snapshot.migrations[3]@,
                    4,
                    domain_migration_name_spec(),
                    domain_migration_checksum_spec(),
                ));
            }
        };
        return false;
    }
    assert(snapshot.migrations.deep_view()[0] == snapshot.migrations[0]@);
    assert(snapshot.migrations.deep_view()[1] == snapshot.migrations[1]@);
    assert(snapshot.migrations.deep_view()[2] == snapshot.migrations[2]@);
    assert(snapshot.migrations.deep_view()[3] == snapshot.migrations[3]@);
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
        MigrationRecordView {
            version: 3,
            name: run_migration_name_spec(),
            checksum: run_migration_checksum_spec(),
        },
        MigrationRecordView {
            version: 4,
            name: domain_migration_name_spec(),
            checksum: domain_migration_checksum_spec(),
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
            } else if database_snapshot_is_exact_v2_spec(database) {
                Ok(InitializationDecision::MigrateV2)
            } else if database_snapshot_is_exact_v3_spec(database) {
                Ok(InitializationDecision::MigrateV3)
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
            } else if database_snapshot_is_exact_v2(database) {
                Ok(InitializationDecision::MigrateV2)
            } else if database_snapshot_is_exact_v3(database) {
                Ok(InitializationDecision::MigrateV3)
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
    Build(String),
    Run(String),
    Fuzz(String),
    Replay(String, String),
    Minimize(String, String),
    Findings(String),
    Inspect(String, String),
    VerifyFinding(String, String, String),
    Report(String, ReportFormat, String),
    Capabilities(String),
    Proof(String),
    Tcb(String),
    Plugins(String),
    InternalLocalSupervisor,
    ArtifactImport(String, String),
    ArtifactVerify(String, String),
    ArtifactCheck(String),
    ArtifactGc(String),
    ConfigValidate(String),
    ConfigCanonicalize(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportFormat {
    Human,
    Json,
    JsonLines,
    Sarif,
    Junit,
    EvidenceGraph,
    BundleManifest,
}

#[verifier::ext_equal]
pub enum CliActionView {
    Init(Seq<char>),
    Build(Seq<char>),
    Run(Seq<char>),
    Fuzz(Seq<char>),
    Replay(Seq<char>, Seq<char>),
    Minimize(Seq<char>, Seq<char>),
    Findings(Seq<char>),
    Inspect(Seq<char>, Seq<char>),
    VerifyFinding(Seq<char>, Seq<char>, Seq<char>),
    Report(Seq<char>, ReportFormat, Seq<char>),
    Capabilities(Seq<char>),
    Proof(Seq<char>),
    Tcb(Seq<char>),
    Plugins(Seq<char>),
    InternalLocalSupervisor,
    ArtifactImport(Seq<char>, Seq<char>),
    ArtifactVerify(Seq<char>, Seq<char>),
    ArtifactCheck(Seq<char>),
    ArtifactGc(Seq<char>),
    ConfigValidate(Seq<char>),
    ConfigCanonicalize(Seq<char>),
}

impl View for CliAction {
    type V = CliActionView;

    open spec fn view(&self) -> CliActionView {
        match self {
            CliAction::Init(path) => CliActionView::Init(path@),
            CliAction::Build(path) => CliActionView::Build(path@),
            CliAction::Run(path) => CliActionView::Run(path@),
            CliAction::Fuzz(path) => CliActionView::Fuzz(path@),
            CliAction::Replay(subject, root) => CliActionView::Replay(subject@, root@),
            CliAction::Minimize(subject, root) => CliActionView::Minimize(subject@, root@),
            CliAction::Findings(root) => CliActionView::Findings(root@),
            CliAction::Inspect(run_id, root) => CliActionView::Inspect(run_id@, root@),
            CliAction::VerifyFinding(subject, patch, root) => {
                CliActionView::VerifyFinding(subject@, patch@, root@)
            },
            CliAction::Report(subject, format, root) => {
                CliActionView::Report(subject@, *format, root@)
            },
            CliAction::Capabilities(root) => CliActionView::Capabilities(root@),
            CliAction::Proof(root) => CliActionView::Proof(root@),
            CliAction::Tcb(root) => CliActionView::Tcb(root@),
            CliAction::Plugins(root) => CliActionView::Plugins(root@),
            CliAction::InternalLocalSupervisor => CliActionView::InternalLocalSupervisor,
            CliAction::ArtifactImport(source, root) => CliActionView::ArtifactImport(
                source@,
                root@,
            ),
            CliAction::ArtifactVerify(id, root) => CliActionView::ArtifactVerify(id@, root@),
            CliAction::ArtifactCheck(root) => CliActionView::ArtifactCheck(root@),
            CliAction::ArtifactGc(root) => CliActionView::ArtifactGc(root@),
            CliAction::ConfigValidate(path) => CliActionView::ConfigValidate(path@),
            CliAction::ConfigCanonicalize(path) => CliActionView::ConfigCanonicalize(path@),
        }
    }
}

pub open spec fn report_format_spec(value: Seq<char>) -> Option<ReportFormat> {
    if value == human_literal_spec() {
        Some(ReportFormat::Human)
    } else if value == json_literal_spec() {
        Some(ReportFormat::Json)
    } else if value == jsonl_literal_spec() {
        Some(ReportFormat::JsonLines)
    } else if value == sarif_literal_spec() {
        Some(ReportFormat::Sarif)
    } else if value == junit_literal_spec() {
        Some(ReportFormat::Junit)
    } else if value == evidence_literal_spec() {
        Some(ReportFormat::EvidenceGraph)
    } else if value == bundle_literal_spec() {
        Some(ReportFormat::BundleManifest)
    } else {
        None
    }
}

#[expect(
    clippy::ptr_arg,
    reason = "the executable view is coupled directly to the verified String view"
)]
fn parse_report_format(value: &String) -> (result: Option<ReportFormat>)
    ensures
        result == report_format_spec(value@),
{
    if value.clone() == human_literal() {
        Some(ReportFormat::Human)
    } else if value.clone() == json_literal() {
        Some(ReportFormat::Json)
    } else if value.clone() == jsonl_literal() {
        Some(ReportFormat::JsonLines)
    } else if value.clone() == sarif_literal() {
        Some(ReportFormat::Sarif)
    } else if value.clone() == junit_literal() {
        Some(ReportFormat::Junit)
    } else if value.clone() == evidence_literal() {
        Some(ReportFormat::EvidenceGraph)
    } else if value.clone() == bundle_literal() {
        Some(ReportFormat::BundleManifest)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliParseError {
    UnsupportedArguments,
}

pub open spec fn parse_cli_args_one_spec(args: Seq<Seq<char>>) -> Result<
    CliActionView,
    CliParseError,
>
    recommends
        args.len() == 1,
{
    if args[0] == init_literal_spec() {
        Ok(CliActionView::Init(current_directory_literal_spec()))
    } else if args[0] == internal_local_supervisor_literal_spec() {
        Ok(CliActionView::InternalLocalSupervisor)
    } else if args[0] == findings_literal_spec() {
        Ok(CliActionView::Findings(current_directory_literal_spec()))
    } else if args[0] == capabilities_literal_spec() {
        Ok(CliActionView::Capabilities(current_directory_literal_spec()))
    } else if args[0] == proof_literal_spec() {
        Ok(CliActionView::Proof(current_directory_literal_spec()))
    } else if args[0] == tcb_literal_spec() {
        Ok(CliActionView::Tcb(current_directory_literal_spec()))
    } else if args[0] == plugins_literal_spec() {
        Ok(CliActionView::Plugins(current_directory_literal_spec()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

pub open spec fn parse_cli_args_two_spec(args: Seq<Seq<char>>) -> Result<
    CliActionView,
    CliParseError,
>
    recommends
        args.len() == 2,
{
    if args[0] == init_literal_spec() {
        Ok(CliActionView::Init(args[1]))
    } else if args[0] == build_literal_spec() {
        Ok(CliActionView::Build(args[1]))
    } else if args[0] == run_literal_spec() {
        Ok(CliActionView::Run(args[1]))
    } else if args[0] == fuzz_literal_spec() {
        Ok(CliActionView::Fuzz(args[1]))
    } else if args[0] == replay_literal_spec() {
        Ok(CliActionView::Replay(args[1], current_directory_literal_spec()))
    } else if args[0] == minimize_literal_spec() {
        Ok(CliActionView::Minimize(args[1], current_directory_literal_spec()))
    } else if args[0] == findings_literal_spec() {
        Ok(CliActionView::Findings(args[1]))
    } else if args[0] == inspect_literal_spec() {
        Ok(CliActionView::Inspect(args[1], current_directory_literal_spec()))
    } else if args[0] == report_literal_spec() {
        Ok(CliActionView::Report(args[1], ReportFormat::Human, current_directory_literal_spec()))
    } else if args[0] == capabilities_literal_spec() {
        Ok(CliActionView::Capabilities(args[1]))
    } else if args[0] == proof_literal_spec() {
        Ok(CliActionView::Proof(args[1]))
    } else if args[0] == tcb_literal_spec() {
        Ok(CliActionView::Tcb(args[1]))
    } else if args[0] == plugins_literal_spec() {
        Ok(CliActionView::Plugins(args[1]))
    } else if args[0] == artifact_literal_spec() && args[1] == check_literal_spec() {
        Ok(CliActionView::ArtifactCheck(current_directory_literal_spec()))
    } else if args[0] == artifact_literal_spec() && args[1] == gc_literal_spec() {
        Ok(CliActionView::ArtifactGc(current_directory_literal_spec()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

pub open spec fn parse_cli_args_three_spec(args: Seq<Seq<char>>) -> Result<
    CliActionView,
    CliParseError,
>
    recommends
        args.len() == 3,
{
    if args[0] == replay_literal_spec() {
        Ok(CliActionView::Replay(args[1], args[2]))
    } else if args[0] == minimize_literal_spec() {
        Ok(CliActionView::Minimize(args[1], args[2]))
    } else if args[0] == inspect_literal_spec() {
        Ok(CliActionView::Inspect(args[1], args[2]))
    } else if args[0] == artifact_literal_spec() && args[1] == import_literal_spec() {
        Ok(CliActionView::ArtifactImport(args[2], current_directory_literal_spec()))
    } else if args[0] == artifact_literal_spec() && args[1] == verify_literal_spec() {
        Ok(CliActionView::ArtifactVerify(args[2], current_directory_literal_spec()))
    } else if args[0] == artifact_literal_spec() && args[1] == check_literal_spec() {
        Ok(CliActionView::ArtifactCheck(args[2]))
    } else if args[0] == artifact_literal_spec() && args[1] == gc_literal_spec() {
        Ok(CliActionView::ArtifactGc(args[2]))
    } else if args[0] == config_literal_spec() && args[1] == validate_literal_spec() {
        Ok(CliActionView::ConfigValidate(args[2]))
    } else if args[0] == config_literal_spec() && args[1] == canonicalize_literal_spec() {
        Ok(CliActionView::ConfigCanonicalize(args[2]))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

pub open spec fn parse_cli_args_four_spec(args: Seq<Seq<char>>) -> Result<
    CliActionView,
    CliParseError,
>
    recommends
        args.len() == 4,
{
    if args[0] == verify_literal_spec() && args[2] == patch_option_literal_spec() {
        Ok(CliActionView::VerifyFinding(args[1], args[3], current_directory_literal_spec()))
    } else if args[0] == report_literal_spec() && args[2] == format_option_literal_spec() {
        match report_format_spec(args[3]) {
            Some(format) => Ok(
                CliActionView::Report(args[1], format, current_directory_literal_spec()),
            ),
            None => Err(CliParseError::UnsupportedArguments),
        }
    } else if args[0] == artifact_literal_spec() && args[1] == import_literal_spec() {
        Ok(CliActionView::ArtifactImport(args[2], args[3]))
    } else if args[0] == artifact_literal_spec() && args[1] == verify_literal_spec() {
        Ok(CliActionView::ArtifactVerify(args[2], args[3]))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

pub open spec fn parse_cli_args_five_spec(args: Seq<Seq<char>>) -> Result<
    CliActionView,
    CliParseError,
>
    recommends
        args.len() == 5,
{
    if args[0] == verify_literal_spec() && args[2] == patch_option_literal_spec() {
        Ok(CliActionView::VerifyFinding(args[1], args[3], args[4]))
    } else if args[0] == report_literal_spec() && args[2] == format_option_literal_spec() {
        match report_format_spec(args[3]) {
            Some(format) => Ok(CliActionView::Report(args[1], format, args[4])),
            None => Err(CliParseError::UnsupportedArguments),
        }
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

pub open spec fn parse_cli_args_spec(args: Seq<Seq<char>>) -> Result<CliActionView, CliParseError> {
    if args.len() == 1 {
        parse_cli_args_one_spec(args)
    } else if args.len() == 2 {
        parse_cli_args_two_spec(args)
    } else if args.len() == 3 {
        parse_cli_args_three_spec(args)
    } else if args.len() == 4 {
        parse_cli_args_four_spec(args)
    } else if args.len() == 5 {
        parse_cli_args_five_spec(args)
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

fn parse_cli_args_one(args: &[String]) -> (result: Result<CliAction, CliParseError>)
    requires
        args.len() == 1,
    ensures
        match (&result, parse_cli_args_one_spec(args.deep_view())) {
            (Ok(action), Ok(expected)) => action@ == expected,
            (Err(error), Err(expected)) => *error == expected,
            _ => false,
        },
{
    let init = init_literal();
    if args[0] == init {
        Ok(CliAction::Init(current_directory_literal()))
    } else if args[0] == internal_local_supervisor_literal() {
        Ok(CliAction::InternalLocalSupervisor)
    } else if args[0] == findings_literal() {
        Ok(CliAction::Findings(current_directory_literal()))
    } else if args[0] == capabilities_literal() {
        Ok(CliAction::Capabilities(current_directory_literal()))
    } else if args[0] == proof_literal() {
        Ok(CliAction::Proof(current_directory_literal()))
    } else if args[0] == tcb_literal() {
        Ok(CliAction::Tcb(current_directory_literal()))
    } else if args[0] == plugins_literal() {
        Ok(CliAction::Plugins(current_directory_literal()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

fn parse_cli_args_two(args: &[String]) -> (result: Result<CliAction, CliParseError>)
    requires
        args.len() == 2,
    ensures
        match (&result, parse_cli_args_two_spec(args.deep_view())) {
            (Ok(action), Ok(expected)) => action@ == expected,
            (Err(error), Err(expected)) => *error == expected,
            _ => false,
        },
{
    if args[0] == init_literal() {
        Ok(CliAction::Init(args[1].clone()))
    } else if args[0] == build_literal() {
        Ok(CliAction::Build(args[1].clone()))
    } else if args[0] == run_literal() {
        Ok(CliAction::Run(args[1].clone()))
    } else if args[0] == fuzz_literal() {
        Ok(CliAction::Fuzz(args[1].clone()))
    } else if args[0] == replay_literal() {
        Ok(CliAction::Replay(args[1].clone(), current_directory_literal()))
    } else if args[0] == minimize_literal() {
        Ok(CliAction::Minimize(args[1].clone(), current_directory_literal()))
    } else if args[0] == findings_literal() {
        Ok(CliAction::Findings(args[1].clone()))
    } else if args[0] == inspect_literal() {
        Ok(CliAction::Inspect(args[1].clone(), current_directory_literal()))
    } else if args[0] == report_literal() {
        Ok(CliAction::Report(args[1].clone(), ReportFormat::Human, current_directory_literal()))
    } else if args[0] == capabilities_literal() {
        Ok(CliAction::Capabilities(args[1].clone()))
    } else if args[0] == proof_literal() {
        Ok(CliAction::Proof(args[1].clone()))
    } else if args[0] == tcb_literal() {
        Ok(CliAction::Tcb(args[1].clone()))
    } else if args[0] == plugins_literal() {
        Ok(CliAction::Plugins(args[1].clone()))
    } else if args[0] == artifact_literal() && args[1] == check_literal() {
        Ok(CliAction::ArtifactCheck(current_directory_literal()))
    } else if args[0] == artifact_literal() && args[1] == gc_literal() {
        Ok(CliAction::ArtifactGc(current_directory_literal()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

fn parse_cli_args_three(args: &[String]) -> (result: Result<CliAction, CliParseError>)
    requires
        args.len() == 3,
    ensures
        match (&result, parse_cli_args_three_spec(args.deep_view())) {
            (Ok(action), Ok(expected)) => action@ == expected,
            (Err(error), Err(expected)) => *error == expected,
            _ => false,
        },
{
    if args[0] == replay_literal() {
        Ok(CliAction::Replay(args[1].clone(), args[2].clone()))
    } else if args[0] == minimize_literal() {
        Ok(CliAction::Minimize(args[1].clone(), args[2].clone()))
    } else if args[0] == inspect_literal() {
        Ok(CliAction::Inspect(args[1].clone(), args[2].clone()))
    } else if args[0] == artifact_literal() && args[1] == import_literal() {
        Ok(CliAction::ArtifactImport(args[2].clone(), current_directory_literal()))
    } else if args[0] == artifact_literal() && args[1] == verify_literal() {
        Ok(CliAction::ArtifactVerify(args[2].clone(), current_directory_literal()))
    } else if args[0] == artifact_literal() && args[1] == check_literal() {
        Ok(CliAction::ArtifactCheck(args[2].clone()))
    } else if args[0] == artifact_literal() && args[1] == gc_literal() {
        Ok(CliAction::ArtifactGc(args[2].clone()))
    } else if args[0] == config_literal() && args[1] == validate_literal() {
        Ok(CliAction::ConfigValidate(args[2].clone()))
    } else if args[0] == config_literal() && args[1] == canonicalize_literal() {
        Ok(CliAction::ConfigCanonicalize(args[2].clone()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

fn parse_cli_args_four(args: &[String]) -> (result: Result<CliAction, CliParseError>)
    requires
        args.len() == 4,
    ensures
        match (&result, parse_cli_args_four_spec(args.deep_view())) {
            (Ok(action), Ok(expected)) => action@ == expected,
            (Err(error), Err(expected)) => *error == expected,
            _ => false,
        },
{
    if args[0] == verify_literal() && args[2] == patch_option_literal() {
        Ok(CliAction::VerifyFinding(args[1].clone(), args[3].clone(), current_directory_literal()))
    } else if args[0] == report_literal() && args[2] == format_option_literal() {
        match parse_report_format(&args[3]) {
            Some(format) => Ok(
                CliAction::Report(args[1].clone(), format, current_directory_literal()),
            ),
            None => Err(CliParseError::UnsupportedArguments),
        }
    } else if args[0] == artifact_literal() && args[1] == import_literal() {
        Ok(CliAction::ArtifactImport(args[2].clone(), args[3].clone()))
    } else if args[0] == artifact_literal() && args[1] == verify_literal() {
        Ok(CliAction::ArtifactVerify(args[2].clone(), args[3].clone()))
    } else {
        Err(CliParseError::UnsupportedArguments)
    }
}

fn parse_cli_args_five(args: &[String]) -> (result: Result<CliAction, CliParseError>)
    requires
        args.len() == 5,
    ensures
        match (&result, parse_cli_args_five_spec(args.deep_view())) {
            (Ok(action), Ok(expected)) => action@ == expected,
            (Err(error), Err(expected)) => *error == expected,
            _ => false,
        },
{
    if args[0] == verify_literal() && args[2] == patch_option_literal() {
        Ok(CliAction::VerifyFinding(args[1].clone(), args[3].clone(), args[4].clone()))
    } else if args[0] == report_literal() && args[2] == format_option_literal() {
        match parse_report_format(&args[3]) {
            Some(format) => Ok(CliAction::Report(args[1].clone(), format, args[4].clone())),
            None => Err(CliParseError::UnsupportedArguments),
        }
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
    match args.len() {
        1 => parse_cli_args_one(args),
        2 => parse_cli_args_two(args),
        3 => parse_cli_args_three(args),
        4 => parse_cli_args_four(args),
        5 => parse_cli_args_five(args),
        _ => Err(CliParseError::UnsupportedArguments),
    }
}

} // verus!
