use crucible_xtask::{
    parse_approvals, parse_ledger, reconcile_boundaries, scan_boundaries, AuditError, BoundaryKind,
    SourceFile, SourceOrigin,
};

fn joined_bytes(first: &str, second: &str) -> Vec<u8> {
    format!("{first}{second}").into_bytes()
}

fn workspace_source(contents: Vec<u8>) -> SourceFile {
    SourceFile::new(
        b"crates/fixture/src/lib.rs".to_vec(),
        contents,
        SourceOrigin::Workspace,
    )
}

#[test]
fn scanner_finds_registered_external_body() {
    let contents = joined_bytes(
        "// CRUCIBLE-TCB: FIXTURE-001\n#[verifier::exter",
        "nal_body]\nfn boundary() {}\n",
    );
    let sources = vec![workspace_source(contents)];

    let boundaries = scan_boundaries(&sources).expect("registered boundary should scan");
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].kind, BoundaryKind::ExternalBody);
    assert_eq!(boundaries[0].line, 2);
    assert_eq!(boundaries[0].id, b"FIXTURE-001");
}

#[test]
fn scanner_rejects_every_unregistered_boundary_spelling() {
    let direct_external = joined_bytes("#[verifier::ex", "ternal]\nfn one() {}\n");
    let cfg_external = joined_bytes(
        "#[cfg_attr(feature = \"x\", verifier::ex",
        "ternal)]\nfn two() {}\n",
    );
    let commented_assume = joined_bytes("fn three() { ass", "ume /* gap */ (true); }\n");
    let sources = vec![
        workspace_source(direct_external),
        workspace_source(cfg_external),
        workspace_source(commented_assume),
    ];

    assert_eq!(
        scan_boundaries(&sources).unwrap_err(),
        AuditError::MissingRegistrationMarker
    );
}

#[test]
fn scanner_counts_multiple_boundaries_on_one_line() {
    let contents = joined_bytes(
        "// CRUCIBLE-TCB: FIRST\n#[verifier::exter",
        "nal_body] fn one() {} #[verifier::external] fn two() {}\n",
    );
    let sources = vec![workspace_source(contents)];

    let boundaries = scan_boundaries(&sources).expect("both boundaries should be visible");
    assert_eq!(boundaries.len(), 2);
}

#[test]
fn scanner_ignores_boundary_words_inside_comments_and_strings() {
    let contents = joined_bytes(
        "// verifier::exter",
        "nal_body is documentation\nconst TEXT: &str = \"assume external\";\n",
    );
    let sources = vec![workspace_source(contents)];

    let boundaries = scan_boundaries(&sources).expect("comments and strings are not code");
    assert!(boundaries.is_empty());
}

#[test]
fn scanner_rejects_generated_and_symlinked_sources() {
    for origin in [
        SourceOrigin::Generated,
        SourceOrigin::Included,
        SourceOrigin::Symlink,
    ] {
        let sources = vec![SourceFile::new(
            b"generated.rs".to_vec(),
            Vec::new(),
            origin,
        )];
        assert_eq!(
            scan_boundaries(&sources).unwrap_err(),
            AuditError::ProhibitedSourceOrigin
        );
    }
}

#[test]
fn ledger_parser_rejects_missing_required_fields() {
    let malformed = b"crucible-tcb-ledger\t1\nboundary\tONLY-AN-ID\n";
    assert_eq!(
        parse_ledger(malformed).unwrap_err(),
        AuditError::MalformedLedger
    );
}

fn registered_fixture() -> (Vec<SourceFile>, Vec<u8>, Vec<u8>) {
    let contents = joined_bytes(
        "// CRUCIBLE-TCB: FIXTURE-001\n#[verifier::exter",
        "nal_body]\nfn boundary() {}\n",
    );
    let source_len = contents.len();
    let source_lines = contents.iter().filter(|byte| **byte == b'\n').count() + 1;
    let sources = vec![workspace_source(contents)];
    let fields = [
        "FIXTURE-001",
        "crates/fixture/src/lib.rs",
        "external_body",
        "fixture component",
        "host API is not specified",
        "host may return arbitrary data",
        "caller validates all data",
        "tcb_contract",
        "owner-review-1",
        "approved",
        "upstream-verus-issue",
        "next-toolchain-upgrade",
    ];
    let ledger_record = format!("boundary\t{}\n", fields.join("\t"));
    let ledger = format!("crucible-tcb-ledger\t1\n{ledger_record}").into_bytes();
    let approval = format!(
        "crucible-tcb-approvals\t1\napproved\t{}\t2\t{source_len}\t{source_lines}\n",
        fields.join("\t")
    )
    .into_bytes();
    (sources, ledger, approval)
}

#[test]
fn ledger_parser_accepts_and_retains_every_required_field() {
    let (_, ledger, _) = registered_fixture();
    let entries = parse_ledger(&ledger).expect("canonical ledger must parse");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, b"FIXTURE-001");
    assert_eq!(entries[0].approval, b"approved");
    assert!(!entries[0].assumption.is_empty());
    assert!(!entries[0].review_trigger.is_empty());
}

#[test]
fn reconciliation_binds_source_ledger_and_approval_metadata() {
    let (sources, ledger, approvals) = registered_fixture();
    let ledger = parse_ledger(&ledger).expect("ledger");
    let approvals = parse_approvals(&approvals).expect("approvals");
    let summary =
        reconcile_boundaries(&sources, &ledger, &approvals).expect("strict reconciliation");
    assert_eq!(summary.registered, 1);
    assert_eq!(summary.external_body_entries, 1);
    assert_eq!(summary.external_entries, 0);
    assert_eq!(summary.external_type_specification_entries, 0);
    assert_eq!(summary.assume_specification_entries, 0);
    assert_eq!(summary.assume_entries, 0);
    assert_eq!(summary.admit_entries, 0);
    assert_eq!(summary.axiom_entries, 0);
    assert_eq!(summary.unsafe_entries, 0);
    assert_eq!(summary.foreign_entries, 0);
    assert_eq!(summary.included_source_entries, 0);
    assert_eq!(summary.unregistered, 0);
    assert_eq!(summary.unapproved_growth, 0);
}

#[test]
fn reconciliation_rejects_duplicate_ids_and_approval_drift() {
    let (sources, ledger, approvals) = registered_fixture();
    let mut ledger = parse_ledger(&ledger).expect("ledger");
    ledger.push(
        parse_ledger(&registered_fixture().1)
            .expect("second ledger")
            .remove(0),
    );
    let approvals = parse_approvals(&approvals).expect("approvals");
    assert_eq!(
        reconcile_boundaries(&sources, &ledger, &approvals).unwrap_err(),
        AuditError::DuplicateLedgerId
    );

    let (_, ledger, mut approval_bytes) = registered_fixture();
    let last_digit = approval_bytes
        .iter()
        .rposition(|byte| byte.is_ascii_digit())
        .expect("line count");
    approval_bytes[last_digit] = b'9';
    let ledger = parse_ledger(&ledger).expect("ledger");
    let approvals = parse_approvals(&approval_bytes).expect("approvals");
    assert_eq!(
        reconcile_boundaries(&sources, &ledger, &approvals).unwrap_err(),
        AuditError::ApprovalMetadataMismatch
    );
}
