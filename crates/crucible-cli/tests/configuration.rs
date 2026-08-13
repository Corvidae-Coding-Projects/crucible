use crucible_cli::{
    canonical_configuration_limits, validate_configuration, ConfigurationErrorKind,
    ConfigurationLimits, CONFIGURATION_CANONICALIZATION_VERSION, CONFIGURATION_SCHEMA_VERSION,
};

const VALID_CONFIGURATION: &str = r#"version: 1

language:
  profile: crucible-yaml-1

project:
  name: example-parser

target:
  adapter: cli
  command: ./build/example-parser
  args:
    - "{input_file}"

execution:
  timeout_ms: 2000
  memory_mb: 1024
  max_processes: 32
  max_output_mb: 16
  network: false
  required_capabilities:
    - process_group_termination
    - resource_limits

oracles:
  process_exit:
    allowed_codes: [0]
    timeout_is_failure: true

inputs:
  corpus:
    - ./seeds/

engines:
  fuzz:
    enabled: true
    modes:
      - managed
      - native
    native_backends:
      - afl++
      - libfuzzer
      - honggfuzz
  property:
    enabled: true
  differential:
    enabled: false
  metamorphic:
    enabled: true
  fault:
    enabled: true
  concurrency:
    enabled: false
  symbolic:
    enabled: false
  mutation:
    enabled: false

sanitizers:
  address: true
  undefined: true
  thread: false
  memory: false
  leak: true

campaign:
  duration: 8h
  workers: 8
  seed: 123456789

storage:
  root: .crucible

verification:
  verus:
    required: true
    deny_unregistered_assumptions: true
    deny_unapproved_tcb_growth: true
"#;

const CANONICAL_CONFIGURATION: &str = concat!(
    "{\"version\":1,",
    "\"language\":{\"profile\":\"crucible-yaml-1\"},",
    "\"project\":{\"name\":\"example-parser\"},",
    "\"target\":{\"adapter\":\"cli\",\"command\":\"./build/example-parser\",",
    "\"args\":[\"{input_file}\"]},",
    "\"execution\":{\"timeout_ms\":2000,\"memory_mb\":1024,\"max_processes\":32,",
    "\"max_output_mb\":16,\"network\":false,",
    "\"required_capabilities\":[\"process_group_termination\",\"resource_limits\"]},",
    "\"oracles\":{\"process_exit\":{\"allowed_codes\":[0],",
    "\"timeout_is_failure\":true}},",
    "\"inputs\":{\"corpus\":[\"./seeds/\"]},",
    "\"engines\":{\"fuzz\":{\"enabled\":true,\"modes\":[\"managed\",\"native\"],",
    "\"native_backends\":[\"afl++\",\"libfuzzer\",\"honggfuzz\"]},",
    "\"property\":{\"enabled\":true},\"differential\":{\"enabled\":false},",
    "\"metamorphic\":{\"enabled\":true},\"fault\":{\"enabled\":true},",
    "\"concurrency\":{\"enabled\":false},\"symbolic\":{\"enabled\":false},",
    "\"mutation\":{\"enabled\":false}},",
    "\"sanitizers\":{\"address\":true,\"undefined\":true,\"thread\":false,",
    "\"memory\":false,\"leak\":true},",
    "\"campaign\":{\"duration\":\"8h\",\"workers\":8,\"seed\":123456789},",
    "\"storage\":{\"root\":\".crucible\"},",
    "\"verification\":{\"verus\":{\"required\":true,",
    "\"deny_unregistered_assumptions\":true,\"deny_unapproved_tcb_growth\":true}}}\n",
);

#[test]
fn complete_design_configuration_validates_and_has_an_exact_canonical_form() {
    let validated = validate_configuration(
        VALID_CONFIGURATION.as_bytes(),
        canonical_configuration_limits(),
    )
    .expect("design configuration must validate");

    assert_eq!(validated.schema_version(), CONFIGURATION_SCHEMA_VERSION);
    assert_eq!(
        validated.canonicalization_version(),
        CONFIGURATION_CANONICALIZATION_VERSION
    );
    assert_eq!(
        validated.canonical_bytes(),
        CANONICAL_CONFIGURATION.as_bytes()
    );
    assert_eq!(validated.digest().to_hex().len(), 64);

    let reparsed = validate_configuration(
        validated.canonical_bytes(),
        canonical_configuration_limits(),
    )
    .expect("canonical form must parse and validate");
    assert_eq!(reparsed.canonical_bytes(), validated.canonical_bytes());
    assert_eq!(reparsed.digest(), validated.digest());
}

#[test]
fn harmless_order_comments_and_scalar_spelling_do_not_change_the_digest() {
    let equivalent = VALID_CONFIGURATION
        .replace(
            "version: 1\n",
            "# presentation-only comment\nversion: 0x1\n",
        )
        .replace(
            "  timeout_ms: 2000\n  memory_mb: 1024\n",
            "  memory_mb: 1024\n  timeout_ms: 0x7d0\n",
        );
    let left = validate_configuration(
        VALID_CONFIGURATION.as_bytes(),
        canonical_configuration_limits(),
    )
    .unwrap();
    let right =
        validate_configuration(equivalent.as_bytes(), canonical_configuration_limits()).unwrap();
    assert_eq!(left.canonical_bytes(), right.canonical_bytes());
    assert_eq!(left.digest(), right.digest());
}

#[test]
fn unknown_execution_affecting_fields_are_rejected_at_the_key() {
    let input = VALID_CONFIGURATION.replace(
        "  network: false\n",
        "  network: false\n  future_execution_switch: true\n",
    );
    let error = validate_configuration(input.as_bytes(), canonical_configuration_limits())
        .expect_err("unknown execution field must be rejected");
    assert_eq!(error.kind(), ConfigurationErrorKind::UnknownField);
    assert_eq!(
        error.byte_offset(),
        input.find("future_execution_switch").unwrap() as u64
    );
}

#[test]
fn wrong_types_and_unsupported_schema_versions_are_distinct() {
    let wrong_type = VALID_CONFIGURATION.replace("timeout_ms: 2000", "timeout_ms: fast");
    let type_error =
        validate_configuration(wrong_type.as_bytes(), canonical_configuration_limits())
            .unwrap_err();
    assert_eq!(type_error.kind(), ConfigurationErrorKind::WrongValueKind);
    assert_eq!(
        type_error.byte_offset(),
        wrong_type.find("fast").unwrap() as u64
    );

    let future = VALID_CONFIGURATION.replacen("version: 1", "version: 2", 1);
    let version_error =
        validate_configuration(future.as_bytes(), canonical_configuration_limits()).unwrap_err();
    assert_eq!(
        version_error.kind(),
        ConfigurationErrorKind::UnsupportedSchemaVersion
    );
    assert_eq!(
        version_error.byte_offset(),
        future.find('2').unwrap() as u64
    );
}

#[test]
fn native_fuzz_mode_requires_at_least_one_native_backend() {
    let invalid = VALID_CONFIGURATION.replace(
        "    native_backends:\n      - afl++\n      - libfuzzer\n      - honggfuzz\n",
        "    native_backends: []\n",
    );
    let error =
        validate_configuration(invalid.as_bytes(), canonical_configuration_limits()).unwrap_err();
    assert_eq!(error.kind(), ConfigurationErrorKind::CrossFieldInvariant);
    assert_eq!(
        error.byte_offset(),
        invalid.find("native_backends").unwrap() as u64
    );
}

#[test]
fn source_and_canonical_output_limits_fail_at_the_first_excluded_byte() {
    let canonical = canonical_configuration_limits();
    let source_limit = ConfigurationLimits::new(
        VALID_CONFIGURATION.len() as u64 - 1,
        canonical.max_typed_nodes(),
        canonical.max_canonical_bytes(),
        canonical.max_depth(),
    );
    let source_error =
        validate_configuration(VALID_CONFIGURATION.as_bytes(), source_limit).unwrap_err();
    assert_eq!(
        source_error.kind(),
        ConfigurationErrorKind::SourceByteLimitExceeded
    );
    assert_eq!(
        source_error.byte_offset(),
        VALID_CONFIGURATION.len() as u64 - 1
    );

    let output_limit = ConfigurationLimits::new(
        canonical.max_source_bytes(),
        canonical.max_typed_nodes(),
        CANONICAL_CONFIGURATION.len() as u64 - 1,
        canonical.max_depth(),
    );
    let output_error =
        validate_configuration(VALID_CONFIGURATION.as_bytes(), output_limit).unwrap_err();
    assert_eq!(
        output_error.kind(),
        ConfigurationErrorKind::CanonicalByteLimitExceeded
    );
    assert_eq!(
        output_error.canonical_byte_index(),
        Some(CANONICAL_CONFIGURATION.len() as u64 - 1)
    );
}

#[test]
fn supplementary_unicode_is_json_compatible_and_canonically_reparseable() {
    let input = VALID_CONFIGURATION.replace("name: example-parser", "name: \"😀\"");
    let validated =
        validate_configuration(input.as_bytes(), canonical_configuration_limits()).unwrap();

    assert!(validated
        .canonical_bytes()
        .windows("😀".len())
        .any(|window| window == "😀".as_bytes()));
    let decoded: serde_json::Value = serde_json::from_slice(validated.canonical_bytes())
        .expect("canonical YAML v1 must remain a JSON document");
    assert_eq!(decoded["project"]["name"], "😀");

    let reparsed = validate_configuration(
        validated.canonical_bytes(),
        canonical_configuration_limits(),
    )
    .expect("JSON-compatible supplementary Unicode must reparse");
    assert_eq!(reparsed.canonical_bytes(), validated.canonical_bytes());
}

#[test]
fn declared_empty_values_are_preserved_without_undocumented_restrictions() {
    let empty_capability = VALID_CONFIGURATION.replace(
        "  required_capabilities:\n    - process_group_termination\n    - resource_limits\n",
        "  required_capabilities: [\"\"]\n",
    );
    let capability = validate_configuration(
        empty_capability.as_bytes(),
        canonical_configuration_limits(),
    )
    .expect("the declared schema permits an empty capability identity");
    assert!(capability
        .canonical_bytes()
        .windows(b"\"required_capabilities\":[\"\"]".len())
        .any(|window| window == b"\"required_capabilities\":[\"\"]"));

    let empty_corpus =
        VALID_CONFIGURATION.replace("  corpus:\n    - ./seeds/\n", "  corpus: [\"\"]\n");
    validate_configuration(empty_corpus.as_bytes(), canonical_configuration_limits())
        .expect("the declared schema permits an empty corpus path");

    let empty_allowed_codes =
        VALID_CONFIGURATION.replace("allowed_codes: [0]", "allowed_codes: []");
    validate_configuration(
        empty_allowed_codes.as_bytes(),
        canonical_configuration_limits(),
    )
    .expect("an empty allowed-code set intentionally treats every exit as a failure");
}

#[test]
fn caller_lowered_work_budget_charges_uniqueness_comparisons() {
    let one = VALID_CONFIGURATION.replace(
        "  required_capabilities:\n    - process_group_termination\n    - resource_limits\n",
        "  required_capabilities: [a]\n",
    );
    let two = VALID_CONFIGURATION.replace(
        "  required_capabilities:\n    - process_group_termination\n    - resource_limits\n",
        "  required_capabilities: [a, b]\n",
    );
    let one_valid =
        validate_configuration(one.as_bytes(), canonical_configuration_limits()).unwrap();
    let two_valid =
        validate_configuration(two.as_bytes(), canonical_configuration_limits()).unwrap();
    assert!(two_valid.work_count() >= one_valid.work_count() + 4);

    let limited = canonical_configuration_limits().with_max_work(two_valid.work_count() - 1);
    let error = validate_configuration(two.as_bytes(), limited).unwrap_err();
    assert_eq!(error.kind(), ConfigurationErrorKind::WorkLimitExceeded);
}
