use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, parse_profile1_cst,
    scan_profile1_block_scalars, scan_profile1_completed_tokens, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AtomizeLimits, BomPolicy,
    CstErrorKind, CstLimits, CstNodeKind, CstNodeStyle, CstSource, CstWarningKind, DecodeLimits,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
};

fn parse_with_limits(input: &[u8], limits: CstLimits) -> Result<CstSource, (CstErrorKind, u64)> {
    let decoded = decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 bytes");
    let atoms = atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS))
        .expect("bounded atom source");
    let layout = analyze_profile1_layout(&atoms, canonical_structural_layout_limits())
        .expect("canonical layout");
    let structural =
        scan_profile1_structural_lexemes(&atoms, &layout, canonical_structural_scan_limits())
            .expect("canonical structural candidates");
    let quoted = scan_profile1_quoted_scalars(
        &atoms,
        &layout,
        &structural,
        canonical_quoted_scalar_limits(),
    )
    .expect("canonical quoted scalars");
    let plain = scan_profile1_plain_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    )
    .expect("canonical plain scalars");
    let block = scan_profile1_block_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    )
    .expect("canonical block scalars");
    let tokens = scan_profile1_completed_tokens(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    )
    .expect("canonical completed tokens");
    parse_profile1_cst(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        &tokens,
        limits,
    )
    .map_err(|error| (error.kind(), error.byte_offset()))
}

fn parse(input: &[u8]) -> Result<CstSource, (CstErrorKind, u64)> {
    parse_with_limits(input, canonical_cst_limits())
}

fn assert_child_before_parent_tables(cst: &CstSource) {
    assert_eq!(cst.syntax_owners().len(), cst.input_token_count() as usize);
    for (token_index, owner) in cst.syntax_owners().iter().enumerate() {
        if let Some(owner) = owner {
            assert_eq!(owner.token_index(), token_index as u64);
        }
    }
    for (node_index, node) in cst.nodes().iter().enumerate() {
        let start = node.entry_start() as usize;
        let end = node.entry_end() as usize;
        match node.kind() {
            CstNodeKind::Sequence => {
                assert!(start <= end && end <= cst.sequence_entries().len());
                for entry in &cst.sequence_entries()[start..end] {
                    assert!((entry.node_index() as usize) < node_index);
                }
            }
            CstNodeKind::Mapping => {
                assert!(start <= end && end <= cst.mapping_entries().len());
                for entry in &cst.mapping_entries()[start..end] {
                    assert!((entry.key_node_index() as usize) < node_index);
                    assert!((entry.value_node_index() as usize) < node_index);
                }
            }
            _ => assert_eq!((start, end), (0, 0)),
        }
    }
    for entry_index in 0..cst.sequence_entries().len() {
        let entry = &cst.sequence_entries()[entry_index];
        assert!(entry.token_start() < entry.token_end());
        assert!(entry.token_end() <= cst.input_token_count());
        assert_eq!(
            cst.nodes()
                .iter()
                .filter(|node| {
                    node.kind() == CstNodeKind::Sequence
                        && node.entry_start() <= entry_index as u64
                        && (entry_index as u64) < node.entry_end()
                })
                .count(),
            1,
            "sequence entry {entry_index} must have exactly one owning collection",
        );
    }
    for entry_index in 0..cst.mapping_entries().len() {
        let entry = &cst.mapping_entries()[entry_index];
        assert!(entry.token_start() < entry.token_end());
        assert!(entry.token_end() <= cst.input_token_count());
        assert_eq!(
            cst.nodes()
                .iter()
                .filter(|node| {
                    node.kind() == CstNodeKind::Mapping
                        && node.entry_start() <= entry_index as u64
                        && (entry_index as u64) < node.entry_end()
                })
                .count(),
            1,
            "mapping entry {entry_index} must have exactly one owning collection",
        );
    }
}

fn assert_document_regions_partition(cst: &CstSource) {
    for document in cst.documents() {
        assert_eq!(document.token_start(), document.prefix_token_start());
        assert_eq!(document.prefix_token_end(), document.directive_start());
        assert_eq!(
            document.directive_end(),
            document.explicit_start_token_start(),
        );
        assert_eq!(
            document.explicit_start_token_end(),
            document.root_token_start(),
        );
        assert_eq!(
            document.root_token_end(),
            document.explicit_end_token_start()
        );
        assert_eq!(
            document.explicit_end_token_end(),
            document.suffix_token_start(),
        );
        assert_eq!(document.suffix_token_end(), document.token_end());
    }
}

#[test]
fn multidocument_block_flow_and_empty_nodes_form_one_lossless_cst() {
    let input =
        b"%YAML 1.2\n---\nroot:\n  - plain\n  - {flow: \"quoted\", empty:}\n...\n---\n[one, two]\n";
    let cst = parse(input).expect("valid YAML 1.2.2 stream");
    assert_child_before_parent_tables(&cst);
    assert_document_regions_partition(&cst);

    assert_eq!(cst.documents().len(), 2);
    assert_eq!(cst.warnings().len(), 0);
    assert_eq!(cst.directive_count(), 1);
    assert_eq!(cst.maximum_depth(), 3);
    assert_eq!(cst.documents()[0].token_start(), 0);
    assert!(
        cst.documents()[0].explicit_start_token_start()
            < cst.documents()[0].explicit_start_token_end(),
    );
    assert!(
        cst.documents()[0].explicit_end_token_start() < cst.documents()[0].explicit_end_token_end(),
    );
    assert_eq!(
        cst.documents()[1].byte_end(),
        input.len() as u64,
        "the final document retains its complete presentation interval",
    );
    assert!(cst
        .nodes()
        .iter()
        .any(|node| node.kind() == CstNodeKind::Mapping && node.style() == CstNodeStyle::Block));
    assert!(cst
        .nodes()
        .iter()
        .any(|node| node.kind() == CstNodeKind::Sequence && node.style() == CstNodeStyle::Flow));
    assert!(
        cst.nodes()
            .iter()
            .any(|node| node.kind() == CstNodeKind::Empty),
        "the empty flow-mapping value is an anchored CST node",
    );
    for document in cst.documents() {
        assert!((document.root_node_index() as usize) < cst.nodes().len());
    }
}

#[test]
fn directive_state_warnings_and_errors_are_document_local() {
    let input = b"%YAML 1.1\n%FUTURE retained\n--- first\n...\n%YAML 1.3\n--- second\n";
    let cst = parse(input).expect("supported and future-minor directives are retained");
    assert_eq!(cst.directive_count(), 3);
    assert_eq!(
        cst.warnings()
            .iter()
            .map(|warning| warning.kind())
            .collect::<Vec<_>>(),
        vec![
            CstWarningKind::Yaml11Compatibility,
            CstWarningKind::ReservedDirective,
            CstWarningKind::FutureMinorVersion,
        ]
    );

    for (input, kind, marker) in [
        (
            &b"%YAML 1.2\n%YAML 1.2\n--- value\n"[..],
            CstErrorKind::DuplicateYamlDirective,
            &b"%YAML 1.2"[..],
        ),
        (
            &b"%TAG !e! tag:one/\n%TAG !e! tag:two/\n--- value\n"[..],
            CstErrorKind::DuplicateTagHandle,
            &b"%TAG !e! tag:two/"[..],
        ),
        (
            &b"%YAML 2.0\n--- value\n"[..],
            CstErrorKind::UnsupportedYamlMajorVersion,
            &b"2.0"[..],
        ),
    ] {
        let second = if kind == CstErrorKind::DuplicateYamlDirective {
            input
                .windows(marker.len())
                .rposition(|window| window == marker)
                .expect("second directive")
        } else {
            input
                .windows(marker.len())
                .position(|window| window == marker)
                .expect("diagnostic marker")
        };
        assert_eq!(
            parse(input),
            Err((kind, second as u64)),
            "fixture: {input:?}"
        );
    }
}

#[test]
fn property_and_flow_grammar_failures_have_first_impossible_offsets() {
    for (input, kind, marker) in [
        (
            &b"--- &one &two value\n"[..],
            CstErrorKind::DuplicateAnchorProperty,
            &b"&two"[..],
        ),
        (
            &b"--- !one !two value\n"[..],
            CstErrorKind::DuplicateTagProperty,
            &b"!two"[..],
        ),
        (
            &b"--- &property *alias\n"[..],
            CstErrorKind::AliasHasPropertiesOrContent,
            &b"*alias"[..],
        ),
        (
            &b"--- [one,,two]\n"[..],
            CstErrorKind::UnexpectedFlowEntry,
            &b",,"[1..2],
        ),
    ] {
        let offset = if kind == CstErrorKind::UnexpectedFlowEntry {
            input
                .windows(marker.len())
                .rposition(|window| window == marker)
                .expect("second flow entry")
        } else {
            input
                .windows(marker.len())
                .position(|window| window == marker)
                .expect("diagnostic marker")
        };
        assert_eq!(
            parse(input),
            Err((kind, offset as u64)),
            "fixture: {input:?}"
        );
    }
}

#[test]
fn complete_presentation_grammar_matrix_accepts_block_flow_properties_and_empty_forms() {
    let fixtures: &[(&str, usize)] = &[
        ("", 0),
        ("# presentation-only stream\n", 0),
        ("---\n...\n", 1),
        ("---\n---\n", 2),
        ("-\n-\n", 1),
        ("- - one\n  - two\n- key: value\n  other: [one, two,]\n", 1),
        ("map:\n  key:\n    - one\n    - two\n", 1),
        ("? [one, two]\n: {three: four}\n", 1),
        ("?\n  - one\n  - two\n: collection-key-value\n", 1),
        ("? key-with-empty-value\nnext: value\n", 1),
        ("?\n: explicit-empty-key\n", 1),
        ("- ? key\n  : value\n", 1),
        ("{? [a, b] : c, : empty-key, empty-value: ,}\n", 1),
        ("[? key: value, : empty-key, ? empty-value]\n", 1),
        ("[{a: b}, c,]\n", 1),
        ("&root !<tag:example.com,2026:node> [*root]\n", 1),
        ("&empty !example\n", 1),
        ("literal: |\n  retained\nfolded: >\n  folded\n", 1),
        (": empty-key\nempty-value:\n", 1),
    ];

    for (input, expected_documents) in fixtures {
        let cst = parse(input.as_bytes())
            .unwrap_or_else(|error| panic!("valid YAML fixture rejected at {error:?}: {input:?}"));
        assert_eq!(
            cst.documents().len(),
            *expected_documents,
            "fixture: {input:?}",
        );
        assert_child_before_parent_tables(&cst);
    }
}

#[test]
fn extended_yaml_1_2_2_collection_and_property_forms_remain_parseable() {
    let fixtures: &[&str] = &[
        "key:\n- one\n- two\n",
        "- key: value\n- key:\n  - nested\n",
        "? |\n  block key\n: block value\n",
        "[foo: bar, baz, {nested: value}]\n",
        "{foo: bar, baz, quoted: \"value\", \"json\":adjacent}\n",
        "&anchor !local scalar\n",
        "!local &anchor scalar\n",
        "key: &empty\nnext: value\n",
        "key: &sequence\n- one\n- two\n",
        "&collection\n- one\n- two\n",
        "- &mapping\n  key: value\n",
        "- &sequence\n  - nested\n",
        "mapping: &collection\n  one: two\n",
        "--- # explicit empty document\n...\n---\nvalue\n...\n",
        "plain: first line\n  second line\nquoted: \"first\n  second\"\n",
    ];

    for input in fixtures {
        let cst = parse(input.as_bytes()).unwrap_or_else(|error| {
            panic!("valid extended YAML fixture rejected at {error:?}: {input:?}")
        });
        assert_child_before_parent_tables(&cst);
    }
}

#[test]
fn block_context_does_not_launder_multiple_roots_or_nested_implicit_pairs() {
    for (input, kind, offset) in [
        (
            &b"outer: inner: value\n"[..],
            CstErrorKind::InvalidIndentation,
            12,
        ),
        (&b"outer: - item\n"[..], CstErrorKind::UnexpectedToken, 7),
        (
            &b"outer: &anchor inner: value\n"[..],
            CstErrorKind::InvalidIndentation,
            20,
        ),
        (
            &b"key: value\n- item\n"[..],
            CstErrorKind::UnexpectedToken,
            11,
        ),
        (
            &b"- item\nkey: value\n"[..],
            CstErrorKind::UnexpectedToken,
            7,
        ),
    ] {
        assert_eq!(parse(input), Err((kind, offset)), "fixture: {input:?}");
    }
}

#[test]
fn every_cst_limit_is_caller_lowerable_and_reports_the_first_excluded_record() {
    let canonical = canonical_cst_limits();
    let cases = [
        (
            &b"value\n"[..],
            CstLimits::new(
                0,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
            ),
            CstErrorKind::DocumentLimitExceeded,
            0,
        ),
        (
            &b"value\n"[..],
            CstLimits::new(
                u64::MAX,
                0,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
            ),
            CstErrorKind::NodeLimitExceeded,
            0,
        ),
        (
            &b"- value\n"[..],
            CstLimits::new(
                u64::MAX,
                u64::MAX,
                0,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
            ),
            CstErrorKind::SequenceEntryLimitExceeded,
            0,
        ),
        (
            &b"key: value\n"[..],
            CstLimits::new(
                u64::MAX,
                u64::MAX,
                u64::MAX,
                0,
                u64::MAX,
                u64::MAX,
                u64::MAX,
            ),
            CstErrorKind::MappingEntryLimitExceeded,
            0,
        ),
        (
            &b"%YAML 1.2\n--- value\n"[..],
            CstLimits::new(
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                0,
                u64::MAX,
                u64::MAX,
            ),
            CstErrorKind::DirectiveLimitExceeded,
            0,
        ),
        (
            &b"%YAML 1.1\n--- value\n"[..],
            CstLimits::new(
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                0,
                u64::MAX,
            ),
            CstErrorKind::WarningLimitExceeded,
            0,
        ),
        (
            &b"[value]\n"[..],
            CstLimits::new(
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                0,
            ),
            CstErrorKind::DepthLimitExceeded,
            0,
        ),
    ];

    assert_eq!(canonical.max_depth(), 4_096);
    for (input, limits, kind, offset) in cases {
        assert_eq!(
            parse_with_limits(input, limits),
            Err((kind, offset)),
            "fixture: {input:?}"
        );
    }
}

#[test]
fn observed_parser_depth_is_exact_and_caller_lowering_rejects_the_first_excluded_frame() {
    let nested = parse_with_limits(
        b"[[value]]\n",
        CstLimits::new(
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            2,
        ),
    )
    .expect("two collection frames fit the caller depth");
    assert_eq!(nested.maximum_depth(), 2);

    assert_eq!(
        parse_with_limits(
            b"[[value]]\n",
            CstLimits::new(
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                u64::MAX,
                1,
            ),
        ),
        Err((CstErrorKind::DepthLimitExceeded, 1)),
    );
}

#[test]
fn malformed_flow_and_block_frame_transitions_fail_at_the_first_impossible_token() {
    for (input, kind, marker) in [
        (
            &b"[, value]\n"[..],
            CstErrorKind::UnexpectedFlowEntry,
            &b","[..],
        ),
        (
            &b"[\"one\" \"two\"]\n"[..],
            CstErrorKind::MissingFlowEntry,
            &b"\"two\""[..],
        ),
        (
            &b"{key: value,,}\n"[..],
            CstErrorKind::UnexpectedFlowEntry,
            &b",,"[1..2],
        ),
        (
            &b"root:\n  child: value\n stray: invalid\n"[..],
            CstErrorKind::InvalidIndentation,
            &b"stray"[..],
        ),
    ] {
        let offset = if input
            .windows(marker.len())
            .filter(|window| *window == marker)
            .count()
            > 1
        {
            input
                .windows(marker.len())
                .rposition(|window| window == marker)
                .unwrap()
        } else {
            input
                .windows(marker.len())
                .position(|window| window == marker)
                .unwrap()
        };
        assert_eq!(
            parse(input),
            Err((kind, offset as u64)),
            "fixture: {input:?}"
        );
    }
}

#[test]
fn named_tag_handles_are_scoped_to_exactly_one_document() {
    let valid = b"%TAG !e! tag:example.com,2026:/\n--- !e!node value\n";
    assert!(parse(valid).is_ok());

    let leaked = b"%TAG !e! tag:example.com,2026:/\n--- !e!node first\n...\n--- !e!node second\n";
    let second_property = leaked
        .windows(b"!e!node".len())
        .rposition(|window| window == b"!e!node")
        .unwrap();
    assert_eq!(
        parse(leaked),
        Err((CstErrorKind::UndeclaredTagHandle, second_property as u64)),
    );
}

#[test]
fn implicit_flow_keys_are_confined_to_one_source_line() {
    for input in [
        &b"[ foo\n  bar: invalid ]\n"[..],
        &b"{ foo\n  bar: invalid }\n"[..],
    ] {
        assert_eq!(
            parse(input),
            Err((CstErrorKind::MultilineImplicitKey, 11)),
            "the mapping-value indicator is the first token that makes the multiline implicit key impossible: {input:?}",
        );
    }
}

#[test]
fn node_properties_require_separation_before_flow_content() {
    for input in [&b"&a[one]\n"[..], &b"!b[one]\n"[..]] {
        assert_eq!(
            parse(input),
            Err((CstErrorKind::MissingPropertySeparation, 2)),
            "the adjacent flow opener lacks the required property separation: {input:?}",
        );
    }
}
