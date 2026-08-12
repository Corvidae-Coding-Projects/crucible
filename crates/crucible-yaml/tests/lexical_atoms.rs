use crucible_yaml::{
    atomize_profile1, classify_lexical_atom, decode_profile1, AtomizeErrorKind, AtomizeLimits,
    AtomizedSource, BomPolicy, DecodeLimits, LexicalAtom, LexicalAtomKind, YamlIndicator,
    MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_SOURCE_BYTES,
};

const PROFILE1_INDICATORS: [(u32, YamlIndicator); 19] = [
    (b'-' as u32, YamlIndicator::BlockSequenceEntry),
    (b'?' as u32, YamlIndicator::ExplicitMappingKey),
    (b':' as u32, YamlIndicator::MappingValue),
    (b',' as u32, YamlIndicator::FlowEntry),
    (b'[' as u32, YamlIndicator::FlowSequenceStart),
    (b']' as u32, YamlIndicator::FlowSequenceEnd),
    (b'{' as u32, YamlIndicator::FlowMappingStart),
    (b'}' as u32, YamlIndicator::FlowMappingEnd),
    (b'#' as u32, YamlIndicator::Comment),
    (b'&' as u32, YamlIndicator::Anchor),
    (b'*' as u32, YamlIndicator::Alias),
    (b'!' as u32, YamlIndicator::Tag),
    (b'|' as u32, YamlIndicator::LiteralBlockScalar),
    (b'>' as u32, YamlIndicator::FoldedBlockScalar),
    (b'\'' as u32, YamlIndicator::SingleQuotedScalar),
    (b'"' as u32, YamlIndicator::DoubleQuotedScalar),
    (b'%' as u32, YamlIndicator::Directive),
    (b'@' as u32, YamlIndicator::ReservedAt),
    (b'`' as u32, YamlIndicator::ReservedGraveAccent),
];

fn decode(input: &[u8]) -> crucible_yaml::DecodedSource {
    decode_profile1(
        input,
        DecodeLimits::new(MAX_PROFILE1_SOURCE_BYTES, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    )
    .expect("valid profile-1 source")
}

fn atomize(source: &crucible_yaml::DecodedSource) -> AtomizedSource {
    atomize_profile1(source, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS))
        .expect("source fits the profile atom cap")
}

fn model_kind(code_point: u32) -> LexicalAtomKind {
    if code_point == b'\n' as u32 {
        LexicalAtomKind::LineFeed
    } else if code_point == b' ' as u32 {
        LexicalAtomKind::Space
    } else if code_point == b'\t' as u32 {
        LexicalAtomKind::Tab
    } else if let Some((_, indicator)) = PROFILE1_INDICATORS
        .iter()
        .find(|(indicator_code_point, _)| *indicator_code_point == code_point)
    {
        LexicalAtomKind::Indicator(*indicator)
    } else {
        LexicalAtomKind::Content
    }
}

fn atom_fact(atom: &LexicalAtom) -> (LexicalAtomKind, u32, (u64, u64, u64, u64, u64, u64)) {
    let span = atom.span();
    (
        atom.kind(),
        atom.code_point(),
        (
            span.start().byte_offset(),
            span.start().line(),
            span.start().column(),
            span.end().byte_offset(),
            span.end().line(),
            span.end().column(),
        ),
    )
}

#[test]
fn every_profile_indicator_whitespace_and_content_class_has_an_exact_atom() {
    let bytes = b"-?:,[]{}#&*!|>'\"%@` \t\r\na\xc3\xa9";
    let decoded = decode(bytes);
    let atomized = atomize(&decoded);

    assert_eq!(atomized.profile_version(), 1);
    assert_eq!(atomized.transformation_version(), 1);
    assert_eq!(atomized.source_len_bytes(), bytes.len() as u64);
    assert_eq!(atomized.bom_bytes(), 0);
    assert_eq!(atomized.atoms().len(), decoded.scalars().len());

    for (atom, scalar) in atomized.atoms().iter().zip(decoded.scalars()) {
        assert_eq!(atom.code_point(), scalar.code_point());
        assert_eq!(atom.kind(), model_kind(atom.code_point()));
        assert_eq!(atom.span(), scalar.span());
    }

    let line_feed = atomized
        .atoms()
        .iter()
        .find(|atom| atom.kind() == LexicalAtomKind::LineFeed)
        .expect("normalized CRLF atom");
    assert_eq!(
        atom_fact(line_feed),
        (LexicalAtomKind::LineFeed, 0x0a, (21, 0, 21, 23, 1, 0))
    );
    assert_eq!(
        atomized.atoms().last().map(LexicalAtom::kind),
        Some(LexicalAtomKind::Content)
    );
}

#[test]
fn normative_indicator_table_is_unique_complete_and_directly_classified() {
    assert_eq!(PROFILE1_INDICATORS.len(), 19);
    for (index, &(code_point, indicator)) in PROFILE1_INDICATORS.iter().enumerate() {
        assert_eq!(
            classify_lexical_atom(code_point),
            LexicalAtomKind::Indicator(indicator)
        );
        assert!(PROFILE1_INDICATORS[..index]
            .iter()
            .all(|(prior_code_point, prior_indicator)| {
                *prior_code_point != code_point && *prior_indicator != indicator
            }));

        let high_bit_mutation = code_point ^ 0x80;
        assert_eq!(
            classify_lexical_atom(high_bit_mutation),
            LexicalAtomKind::Content,
            "ASCII indicator U+{code_point:04X} was overmatched after a one-bit mutation"
        );
    }
}

#[test]
fn bom_metadata_is_retained_and_only_the_leading_bom_is_absent_from_atoms() {
    let decoded = decode(&[0xef, 0xbb, 0xbf, b'x', 0xef, 0xbb, 0xbf]);
    let atomized = atomize(&decoded);
    assert_eq!(atomized.source_len_bytes(), 7);
    assert_eq!(atomized.bom_bytes(), 3);
    assert_eq!(atomized.atoms().len(), 2);
    assert_eq!(atomized.atoms()[0].code_point(), b'x' as u32);
    assert_eq!(atomized.atoms()[0].span().start().byte_offset(), 3);
    assert_eq!(atomized.atoms()[1].code_point(), 0xfeff);
    assert_eq!(atomized.atoms()[1].kind(), LexicalAtomKind::Content);
}

#[test]
fn caller_atom_limits_reject_before_partial_atomization_with_the_next_scalar_offset() {
    let decoded = decode(b"a\r\nb");
    let error = atomize_profile1(&decoded, AtomizeLimits::new(2))
        .expect_err("three normalized scalars exceed a two-atom limit");
    assert_eq!(error.kind(), AtomizeErrorKind::AtomLimitExceeded);
    assert_eq!(error.byte_offset(), 3);

    let exact = atomize_profile1(&decoded, AtomizeLimits::new(3)).expect("exact atom limit");
    assert_eq!(exact.atoms().len(), 3);
}

#[test]
fn zero_bom_multibyte_and_large_input_limit_boundaries_have_exact_offsets() {
    let empty = decode(b"");
    assert!(atomize_profile1(&empty, AtomizeLimits::new(0))
        .expect("empty source fits a zero atom limit")
        .atoms()
        .is_empty());

    let one = decode(b"x");
    let zero_error = atomize_profile1(&one, AtomizeLimits::new(0))
        .expect_err("a nonempty source exceeds a zero atom limit");
    assert_eq!(zero_error.kind(), AtomizeErrorKind::AtomLimitExceeded);
    assert_eq!(zero_error.byte_offset(), 0);

    let after_bom = decode(&[0xef, 0xbb, 0xbf, b'x']);
    let bom_error = atomize_profile1(&after_bom, AtomizeLimits::new(0))
        .expect_err("the first post-BOM scalar is excluded");
    assert_eq!(bom_error.kind(), AtomizeErrorKind::AtomLimitExceeded);
    assert_eq!(bom_error.byte_offset(), 3);

    let multibyte = decode("aé".as_bytes());
    let multibyte_error = atomize_profile1(&multibyte, AtomizeLimits::new(1))
        .expect_err("the multibyte scalar starts at byte one");
    assert_eq!(multibyte_error.kind(), AtomizeErrorKind::AtomLimitExceeded);
    assert_eq!(multibyte_error.byte_offset(), 1);

    let large_bytes = vec![b'a'; MAX_PROFILE1_LEXICAL_ATOMS as usize];
    let large = decode(&large_bytes);
    let preflight_error = atomize_profile1(&large, AtomizeLimits::new(1))
        .expect_err("a tiny caller limit rejects a large source before atom construction");
    assert_eq!(preflight_error.kind(), AtomizeErrorKind::AtomLimitExceeded);
    assert_eq!(preflight_error.byte_offset(), 1);
}

#[test]
fn absolute_atom_cap_is_fixed_and_accepts_its_boundary() {
    assert_eq!(MAX_PROFILE1_LEXICAL_ATOMS, 1024 * 1024);
    assert_eq!(MAX_PROFILE1_LEXICAL_ATOMS, MAX_PROFILE1_DECODED_SCALARS);
    let bytes = vec![b'a'; MAX_PROFILE1_LEXICAL_ATOMS as usize];
    let decoded = decode(&bytes);
    let atomized = atomize_profile1(&decoded, AtomizeLimits::new(u64::MAX))
        .expect("the exact absolute atom cap");
    assert_eq!(atomized.atoms().len() as u64, MAX_PROFILE1_LEXICAL_ATOMS);
}

#[test]
fn atomization_is_deterministic_and_classification_matches_every_unicode_scalar() {
    let fixture = decode(b"key: [a, b] # comment\n");
    assert_eq!(
        atomize_profile1(&fixture, AtomizeLimits::new(128)),
        atomize_profile1(&fixture, AtomizeLimits::new(128))
    );

    const CHUNK_SCALARS: usize = 65_536;
    let mut bytes = Vec::new();
    let mut expected = Vec::new();
    let check_chunk = |bytes: &[u8], expected: &[(u32, LexicalAtomKind)]| {
        let decoded = decode(bytes);
        let atomized = atomize(&decoded);
        assert_eq!(atomized.atoms().len(), expected.len());
        for (atom, expected) in atomized.atoms().iter().zip(expected) {
            let normalized = if expected.0 == 0x0d { 0x0a } else { expected.0 };
            assert_eq!(atom.code_point(), normalized);
            assert_eq!(classify_lexical_atom(normalized), expected.1);
            assert_eq!(atom.kind(), expected.1);
        }
    };

    for code_point in 0..=0x10ffff {
        let Some(character) = char::from_u32(code_point) else {
            continue;
        };
        let mut encoded = [0u8; 4];
        bytes.extend_from_slice(character.encode_utf8(&mut encoded).as_bytes());
        let normalized = if code_point == 0x0d { 0x0a } else { code_point };
        expected.push((normalized, model_kind(normalized)));
        if expected.len() == CHUNK_SCALARS {
            check_chunk(&bytes, &expected);
            bytes.clear();
            expected.clear();
        }
    }
    if !expected.is_empty() {
        check_chunk(&bytes, &expected);
    }
}
