#![cfg(target_os = "linux")]

use rusqlite::Connection;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
#[expect(
    unused_imports,
    reason = "Verus requires the prelude crate marker even when this black-box test names no vstd item"
)]
use vstd::prelude::*;

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[test]
fn every_declared_known_defect_target_is_rediscovered_under_bounded_controls() {
    let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "crucible-harness-selftest-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create root");
    let initialized = Command::new(env!("CARGO_BIN_EXE_crucible"))
        .args(["init", root.to_str().expect("UTF-8 root")])
        .current_dir(&root)
        .output()
        .expect("initialize workspace");
    assert!(initialized.status.success());

    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = std::fs::read_to_string(repository.join("testdata/targets/expected.tsv"))
        .expect("read fixture manifest");
    assert!(manifest.len() <= 4_096);
    let fixtures = manifest.lines().skip(1).collect::<Vec<_>>();
    assert_eq!(fixtures.len(), 11);
    for (index, record) in fixtures.iter().enumerate() {
        let fields = record.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        let source = repository.join("testdata/targets").join(fields[0]);
        let target = root.join(format!("fixture-{index}.sh"));
        std::fs::copy(source, &target).expect("copy fixture");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700))
            .expect("make fixture executable");
        let configuration = root.join(format!("fixture-{index}.yaml"));
        std::fs::write(
            &configuration,
            format!(
                r#"version: 1
language: {{profile: crucible-yaml-1}}
project: {{name: selftest-{index}}}
target: {{adapter: cli, command: "{}", args: []}}
execution: {{timeout_ms: 250, memory_mb: 128, max_processes: 4, max_output_mb: 1, network: false, required_capabilities: [process_group_termination, resource_limits, network_isolation, private_working_directory]}}
oracles: {{process_exit: {{allowed_codes: [0], timeout_is_failure: true}}}}
inputs: {{corpus: []}}
engines: {{fuzz: {{enabled: false, modes: [], native_backends: []}}, property: {{enabled: false}}, differential: {{enabled: false}}, metamorphic: {{enabled: false}}, fault: {{enabled: false}}, concurrency: {{enabled: false}}, symbolic: {{enabled: false}}, mutation: {{enabled: false}}}}
sanitizers: {{address: false, undefined: false, thread: false, memory: false, leak: false}}
campaign: {{duration: 1s, workers: 1, seed: {}}}
storage: {{root: .crucible}}
verification: {{verus: {{required: true, deny_unregistered_assumptions: true, deny_unapproved_tcb_growth: true}}}}
"#,
                target.display(),
                index + 1,
            ),
        )
        .expect("write fixture configuration");
        let run = Command::new(env!("CARGO_BIN_EXE_crucible"))
            .args(["run", configuration.to_str().expect("UTF-8 configuration")])
            .current_dir(&root)
            .output()
            .expect("execute fixture");
        assert!(
            run.status.success(),
            "fixture {} ({}) was not observed: {}",
            fields[0],
            fields[1],
            String::from_utf8_lossy(&run.stderr)
        );
        let connection =
            Connection::open(root.join(".crucible/database.sqlite")).expect("open database");
        let verdicts: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM oracle_verdicts ov
                 JOIN run_attempts ra ON ra.id = ov.attempt_id
                 JOIN runs r ON r.id = ra.run_id
                 JOIN finding_instances fi ON fi.run_attempt_id = ra.id
                 JOIN findings f ON f.id = fi.finding_id
                 JOIN projects p ON p.id = f.project_id
                 WHERE ov.verdict = 'fail' AND p.name = ?1",
                [format!("selftest-{index}")],
                |row| row.get(0),
            )
            .expect("query verdict");
        assert_eq!(verdicts, 1, "fixture {} was not rediscovered", fields[0]);
    }
    let connection =
        Connection::open(root.join(".crucible/database.sqlite")).expect("open final database");
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM findings", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        11
    );
    drop(connection);
    std::fs::remove_dir_all(root).expect("remove workspace");
}
