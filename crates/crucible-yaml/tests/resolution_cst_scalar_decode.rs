use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, canonical_block_scalar_limits,
    canonical_completed_token_limits, canonical_cst_limits, canonical_plain_scalar_limits,
    canonical_quoted_scalar_limits, canonical_structural_layout_limits,
    canonical_structural_scan_limits, decode_profile1, decode_profile1_cst_node_scalar,
    parse_profile1_cst, scan_profile1_block_scalars, scan_profile1_completed_tokens,
    scan_profile1_plain_scalars, scan_profile1_quoted_scalars, scan_profile1_structural_lexemes,
    AtomizeLimits, AtomizedSource, BlockScalarSource, BomPolicy, CompletedTokenSource, CstNodeKind,
    CstNodeStyle, CstScalarDecodeErrorKind, CstScalarDecodeLimits, CstSource, DecodeLimits,
    PlainScalarSource, QuotedScalarSource, MAX_PROFILE1_DECODED_SCALARS,
    MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES,
};

struct Parsed {
    atoms: AtomizedSource,
    quoted: QuotedScalarSource,
    plain: PlainScalarSource,
    block: BlockScalarSource,
    tokens: CompletedTokenSource,
    cst: CstSource,
}

fn parse(input: &[u8]) -> Parsed {
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
    let cst = parse_profile1_cst(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        &tokens,
        canonical_cst_limits(),
    )
    .expect("canonical CST");
    Parsed {
        atoms,
        quoted,
        plain,
        block,
        tokens,
        cst,
    }
}

fn unlimited() -> CstScalarDecodeLimits {
    CstScalarDecodeLimits::new(MAX_PROFILE1_DECODED_SCALAR_CONTENT_CODE_POINTS)
}

fn text(value: &crucible_yaml::DecodedCstScalar) -> String {
    value
        .decoded()
        .map(|decoded| {
            decoded
                .content()
                .iter()
                .map(|item| char::from_u32(item.code_point()).expect("Unicode scalar"))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_cst_scalar_style_dispatches_to_its_verified_decoder() {
    let parsed = parse(
        b"plain: alpha\nsingle: 'a''b'\ndouble: \"c\\nd\"\nliteral: |\n  e\nfolded: >\n  f\nempty:\nsequence: [g]\n",
    );
    let expected = [
        (CstNodeStyle::Plain, "alpha"),
        (CstNodeStyle::SingleQuoted, "a'b"),
        (CstNodeStyle::DoubleQuoted, "c\nd"),
        (CstNodeStyle::Literal, "e\n"),
        (CstNodeStyle::Folded, "f\n"),
    ];
    for (style, expected_text) in expected {
        let (node_index, decoded) = parsed
            .cst
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.style() == style && node.kind() == CstNodeKind::Scalar)
            .find_map(|(index, _)| {
                let decoded = decode_profile1_cst_node_scalar(
                    &parsed.atoms,
                    &parsed.quoted,
                    &parsed.plain,
                    &parsed.block,
                    &parsed.tokens,
                    &parsed.cst,
                    index as u64,
                    unlimited(),
                )
                .expect("authenticated scalar decodes")
                .expect("scalar node returns a record");
                (text(&decoded) == expected_text).then_some((index as u64, decoded))
            })
            .expect("fixture style and content exist");
        assert_eq!(decoded.node_index(), node_index);
        assert_eq!(decoded.style(), style);
        assert!(decoded.token_index().is_some());
        assert_eq!(text(&decoded), expected_text);
    }

    let empty_index = parsed
        .cst
        .nodes()
        .iter()
        .position(|node| node.kind() == CstNodeKind::Empty)
        .expect("fixture empty node") as u64;
    let empty = decode_profile1_cst_node_scalar(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        empty_index,
        CstScalarDecodeLimits::new(0),
    )
    .expect("empty scalar consumes no content budget")
    .expect("empty node is a semantic scalar");
    assert_eq!(empty.style(), CstNodeStyle::Empty);
    assert!(empty.token_index().is_none());
    assert!(empty.decoded().is_none());

    let collection_index = parsed
        .cst
        .nodes()
        .iter()
        .position(|node| node.kind() == CstNodeKind::Sequence)
        .expect("fixture collection") as u64;
    assert!(decode_profile1_cst_node_scalar(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        collection_index,
        unlimited(),
    )
    .expect("collections are not scalar decoder errors")
    .is_none());
}

#[test]
fn limits_indices_and_cross_source_authentication_have_exact_diagnostics() {
    let parsed = parse(b"value: \"alpha\"\n");
    let node_index = parsed
        .cst
        .nodes()
        .iter()
        .position(|node| node.style() == CstNodeStyle::DoubleQuoted)
        .expect("double scalar") as u64;
    let full = decode_profile1_cst_node_scalar(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        node_index,
        unlimited(),
    )
    .expect("full decode")
    .expect("scalar");
    let excluded_byte = full.decoded().expect("decoded content").content()[2].byte_start();
    let error = decode_profile1_cst_node_scalar(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        node_index,
        CstScalarDecodeLimits::new(2),
    )
    .expect_err("third character exceeds caller limit");
    assert_eq!(error.kind(), CstScalarDecodeErrorKind::ContentLimitExceeded);
    assert_eq!(error.byte_offset(), excluded_byte);

    let error = decode_profile1_cst_node_scalar(
        &parsed.atoms,
        &parsed.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        parsed.cst.nodes().len() as u64,
        unlimited(),
    )
    .expect_err("node index is checked");
    assert_eq!(error.kind(), CstScalarDecodeErrorKind::NodeIndexOutOfRange);
    assert_eq!(error.byte_offset(), parsed.atoms.source_len_bytes());

    let other = parse(b"other: 'different'\n");
    let error = decode_profile1_cst_node_scalar(
        &parsed.atoms,
        &other.quoted,
        &parsed.plain,
        &parsed.block,
        &parsed.tokens,
        &parsed.cst,
        node_index,
        unlimited(),
    )
    .expect_err("selected quoted evidence must match the atom source");
    assert_eq!(error.kind(), CstScalarDecodeErrorKind::InputQuotedMismatch);
}
