use vstd::prelude::*;

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceOrigin {
    Workspace,
    Generated,
    Included,
    Symlink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryKind {
    ExternalBody,
    External,
    ExternalTypeSpecification,
    AssumeSpecification,
    Assume,
    Admit,
    Axiom,
    Unsafe,
    Foreign,
    IncludedSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditError {
    MissingRegistrationMarker,
    ProhibitedSourceOrigin,
    MalformedSource,
    MalformedLedger,
    MalformedApprovalBaseline,
    DuplicateLedgerId,
    DuplicateApprovalId,
    MissingLedgerEntry,
    MissingApprovalEntry,
    StaleLedgerEntry,
    StaleApprovalEntry,
    ApprovalMetadataMismatch,
}

#[derive(Debug)]
pub struct SourceFile {
    pub path: Vec<u8>,
    pub contents: Vec<u8>,
    pub origin: SourceOrigin,
}

impl SourceFile {
    pub fn new(path: Vec<u8>, contents: Vec<u8>, origin: SourceOrigin) -> (source: Self)
        ensures
            source.path@ == path@,
            source.contents@ == contents@,
            source.origin == origin,
    {
        Self { path, contents, origin }
    }
}

#[derive(Debug)]
pub struct BoundaryOccurrence {
    pub id: Vec<u8>,
    pub source_path: Vec<u8>,
    pub kind: BoundaryKind,
    pub line: usize,
}

impl Clone for BoundaryOccurrence {
    fn clone(&self) -> (copy: Self)
        ensures
            copy.id@ == self.id@,
            copy.source_path@ == self.source_path@,
            copy.kind == self.kind,
            copy.line == self.line,
    {
        Self {
            id: self.id.clone(),
            source_path: self.source_path.clone(),
            kind: self.kind,
            line: self.line,
        }
    }
}

#[derive(Debug)]
pub struct LedgerEntry {
    pub id: Vec<u8>,
    pub source_path: Vec<u8>,
    pub kind: BoundaryKind,
    pub component: Vec<u8>,
    pub reason: Vec<u8>,
    pub assumption: Vec<u8>,
    pub relied_on_property: Vec<u8>,
    pub independent_checks: Vec<u8>,
    pub reviewer: Vec<u8>,
    pub approval: Vec<u8>,
    pub upstream_limitation: Vec<u8>,
    pub review_trigger: Vec<u8>,
}

#[derive(Debug)]
pub struct ApprovalEntry {
    pub id: Vec<u8>,
    pub source_path: Vec<u8>,
    pub kind: BoundaryKind,
    pub component: Vec<u8>,
    pub reason: Vec<u8>,
    pub assumption: Vec<u8>,
    pub relied_on_property: Vec<u8>,
    pub independent_checks: Vec<u8>,
    pub reviewer: Vec<u8>,
    pub approval: Vec<u8>,
    pub upstream_limitation: Vec<u8>,
    pub review_trigger: Vec<u8>,
    pub occurrence_line: usize,
    pub source_bytes: usize,
    pub source_lines: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditSummary {
    pub registered: usize,
    pub external_body_entries: usize,
    pub external_entries: usize,
    pub external_type_specification_entries: usize,
    pub assume_specification_entries: usize,
    pub assume_entries: usize,
    pub admit_entries: usize,
    pub axiom_entries: usize,
    pub unsafe_entries: usize,
    pub foreign_entries: usize,
    pub included_source_entries: usize,
    pub unregistered: usize,
    pub unapproved_growth: usize,
    pub source_files: usize,
}

#[expect(clippy::manual_range_contains, reason = "arithmetic spelling mirrors the Verus specification and proof obligations")]
fn is_identifier_start(byte: u8) -> bool {
    (byte >= b'a' && byte <= b'z') || (byte >= b'A' && byte <= b'Z') || byte == b'_'
}

#[expect(clippy::manual_range_contains, reason = "arithmetic spelling mirrors the Verus specification and proof obligations")]
fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || (byte >= b'0' && byte <= b'9')
}

fn is_marker_id_continue(byte: u8) -> bool {
    is_identifier_continue(byte) || byte == b'-'
}

fn bytes_equal_range(input: &[u8], start: usize, end: usize, expected: &[u8]) -> (equal: bool)
    requires
        start <= end,
        end <= input.len(),
    ensures
        equal == (input@.subrange(start as int, end as int) == expected@),
{
    if end - start != expected.len() {
        assert(input@.subrange(start as int, end as int).len() == end - start);
        return false;
    }
    let mut offset = 0;
    while offset < expected.len()
        invariant
            offset <= expected@.len(),
            end - start == expected@.len(),
            end <= input@.len(),
            forall|prior: int| 0 <= prior < offset ==> input@[start + prior] == expected@[prior],
        decreases expected.len() - offset,
    {
        if input[start + offset] != expected[offset] {
            assert(input@.subrange(start as int, end as int)[offset as int] == input@[start
                + offset]);
            return false;
        }
        offset += 1;
    }
    assert(input@.subrange(start as int, end as int) =~= expected@);
    true
}

fn copy_range(input: &[u8], start: usize, end: usize) -> (copy: Vec<u8>)
    requires
        start <= end,
        end <= input.len(),
    ensures
        copy@ == input@.subrange(start as int, end as int),
{
    let mut copy = Vec::new();
    let mut index = start;
    while index < end
        invariant
            start <= index <= end,
            end <= input@.len(),
            copy@ == input@.subrange(start as int, index as int),
        decreases end - index,
    {
        copy.push(input[index]);
        index += 1;
    }
    copy
}

fn classify_identifier(input: &[u8], start: usize, end: usize) -> (kind: Option<BoundaryKind>)
    requires
        start <= end,
        end <= input.len(),
{
    if bytes_equal_range(input, start, end, b"external_body") {
        Some(BoundaryKind::ExternalBody)
    } else if bytes_equal_range(input, start, end, b"external") {
        Some(BoundaryKind::External)
    } else if bytes_equal_range(input, start, end, b"external_type_specification") {
        Some(BoundaryKind::ExternalTypeSpecification)
    } else if bytes_equal_range(input, start, end, b"assume_specification") {
        Some(BoundaryKind::AssumeSpecification)
    } else if bytes_equal_range(input, start, end, b"assume") {
        Some(BoundaryKind::Assume)
    } else if bytes_equal_range(input, start, end, b"admit") {
        Some(BoundaryKind::Admit)
    } else if bytes_equal_range(input, start, end, b"axiom") {
        Some(BoundaryKind::Axiom)
    } else if bytes_equal_range(input, start, end, b"unsafe") {
        Some(BoundaryKind::Unsafe)
    } else if bytes_equal_range(input, start, end, b"extern") {
        Some(BoundaryKind::Foreign)
    } else if bytes_equal_range(input, start, end, b"include") || bytes_equal_range(
        input,
        start,
        end,
        b"include_str",
    ) || bytes_equal_range(input, start, end, b"include_bytes") {
        Some(BoundaryKind::IncludedSource)
    } else {
        None
    }
}

fn parse_registration_marker(input: &[u8], start: usize, end: usize) -> (id: Option<Vec<u8>>)
    requires
        start <= end,
        end <= input.len(),
{
    let prefix = b"CRUCIBLE-TCB:";
    let mut index = start;
    while index < end && (input[index] == b' ' || input[index] == b'\t')
        invariant
            start <= index <= end,
            end <= input@.len(),
        decreases end - index,
    {
        index += 1;
    }
    if prefix.len() > end - index {
        return None;
    }
    let prefix_end = index + prefix.len();
    if !bytes_equal_range(input, index, prefix_end, prefix) {
        return None;
    }
    index += prefix.len();
    while index < end && (input[index] == b' ' || input[index] == b'\t')
        invariant
            index <= end,
            end <= input@.len(),
        decreases end - index,
    {
        index += 1;
    }
    let id_start = index;
    while index < end && is_marker_id_continue(input[index])
        invariant
            id_start <= index <= end,
            end <= input@.len(),
        decreases end - index,
    {
        index += 1;
    }
    if index == id_start {
        None
    } else {
        Some(copy_range(input, id_start, index))
    }
}

fn scan_one_source(source: &SourceFile) -> (result: Result<Vec<BoundaryOccurrence>, AuditError>) {
    if source.origin != SourceOrigin::Workspace {
        return Err(AuditError::ProhibitedSourceOrigin);
    }
    let input = source.contents.as_slice();
    let mut occurrences = Vec::new();
    let mut index = 0;
    let mut line = 1usize;
    let mut block_depth = 0usize;
    let mut in_string = false;
    let mut in_character = false;
    let mut escaped = false;
    let mut raw_hashes: Option<usize> = None;
    let mut marker: Option<(Vec<u8>, usize)> = None;

    while index < input.len()
        invariant
            index <= input@.len(),
            line >= 1,
        decreases input.len() - index,
    {
        let byte = input[index];

        if block_depth > 0 {
            if byte == b'\n' {
                line = line.checked_add(1).ok_or(AuditError::MalformedSource)?;
                index += 1;
            } else if byte == b'/' && index + 1 < input.len() && input[index + 1] == b'*' {
                block_depth = block_depth.checked_add(1).ok_or(AuditError::MalformedSource)?;
                index += 2;
            } else if byte == b'*' && index + 1 < input.len() && input[index + 1] == b'/' {
                block_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(hash_count) = raw_hashes {
            if byte == b'\n' {
                line = line.checked_add(1).ok_or(AuditError::MalformedSource)?;
                index += 1;
                continue;
            }
            if byte == b'"' {
                let mut cursor = index + 1;
                let mut matched = 0;
                while matched < hash_count && cursor < input.len() && input[cursor] == b'#'
                    invariant
                        index < cursor <= input@.len(),
                        matched <= hash_count,
                        cursor == index + 1 + matched,
                    decreases hash_count - matched,
                {
                    matched += 1;
                    cursor += 1;
                }
                if matched == hash_count {
                    raw_hashes = None;
                    index = cursor;
                    continue;
                }
            }
            index += 1;
            continue;
        }
        if in_string || in_character {
            if byte == b'\n' {
                if in_character {
                    return Err(AuditError::MalformedSource);
                }
                line = line.checked_add(1).ok_or(AuditError::MalformedSource)?;
                index += 1;
                continue;
            }
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if in_string && byte == b'"' {
                in_string = false;
            } else if in_character && byte == b'\'' {
                in_character = false;
            }
            index += 1;
            continue;
        }
        if byte == b'\n' {
            line = line.checked_add(1).ok_or(AuditError::MalformedSource)?;
            index += 1;
        } else if byte == b'/' && index + 1 < input.len() && input[index + 1] == b'/' {
            let comment_start = index + 2;
            let mut comment_end = comment_start;
            while comment_end < input.len() && input[comment_end] != b'\n'
                invariant
                    comment_start <= comment_end <= input@.len(),
                decreases input.len() - comment_end,
            {
                comment_end += 1;
            }
            if let Some(id) = parse_registration_marker(input, comment_start, comment_end) {
                marker = Some((id, line));
            }
            index = comment_end;
        } else if byte == b'/' && index + 1 < input.len() && input[index + 1] == b'*' {
            block_depth = 1;
            index += 2;
        } else if byte == b'r' || (byte == b'b' && input.len() - index > 1 && input[index + 1]
            == b'r') {
            let r_index = if byte == b'r' {
                index
            } else {
                index + 1
            };
            let mut cursor = r_index + 1;
            let mut hash_count = 0usize;
            while cursor < input.len() && input[cursor] == b'#'
                invariant
                    r_index < cursor <= input@.len(),
                    cursor == r_index + 1 + hash_count,
                decreases input.len() - cursor,
            {
                hash_count = hash_count.checked_add(1).ok_or(AuditError::MalformedSource)?;
                cursor += 1;
            }
            if cursor < input.len() && input[cursor] == b'"' {
                raw_hashes = Some(hash_count);
                index = cursor + 1;
            } else {
                let start = index;
                index += 1;
                while index < input.len() && is_identifier_continue(input[index])
                    invariant
                        start < index <= input@.len(),
                    decreases input.len() - index,
                {
                    index += 1;
                }
                if let Some(kind) = classify_identifier(input, start, index) {
                    let id = match &marker {
                        Some((id, marker_line)) if *marker_line <= line && line - *marker_line
                            <= 3 => { id.clone() },
                        _ => return Err(AuditError::MissingRegistrationMarker),
                    };
                    occurrences.push(
                        BoundaryOccurrence { id, source_path: source.path.clone(), kind, line },
                    );
                }
            }
        } else if byte == b'"' {
            in_string = true;
            escaped = false;
            index += 1;
        } else if byte == b'\'' && input.len() - index > 2 && input[index + 2] == b'\'' {
            in_character = true;
            escaped = false;
            index += 1;
        } else if is_identifier_start(byte) {
            let start = index;
            index += 1;
            while index < input.len() && is_identifier_continue(input[index])
                invariant
                    start < index <= input@.len(),
                decreases input.len() - index,
            {
                index += 1;
            }
            if let Some(kind) = classify_identifier(input, start, index) {
                let id = match &marker {
                    Some((id, marker_line)) if *marker_line <= line && line - *marker_line <= 3 => {
                        id.clone()
                    },
                    _ => return Err(AuditError::MissingRegistrationMarker),
                };
                occurrences.push(
                    BoundaryOccurrence { id, source_path: source.path.clone(), kind, line },
                );
            }
        } else {
            index += 1;
        }
    }

    if block_depth != 0 || in_string || in_character || raw_hashes.is_some() {
        Err(AuditError::MalformedSource)
    } else {
        Ok(occurrences)
    }
}

pub fn scan_boundaries(sources: &[SourceFile]) -> (result: Result<
    Vec<BoundaryOccurrence>,
    AuditError,
>) {
    let mut all = Vec::new();
    let mut index = 0;
    while index < sources.len()
        invariant
            index <= sources@.len(),
        decreases sources.len() - index,
    {
        let found = scan_one_source(&sources[index])?;
        let mut found_index = 0;
        while found_index < found.len()
            invariant
                found_index <= found@.len(),
            decreases found.len() - found_index,
        {
            all.push(found[found_index].clone());
            found_index += 1;
        }
        index += 1;
    }
    Ok(all)
}

fn split_fields(input: &[u8], start: usize, end: usize) -> (fields: Vec<(usize, usize)>)
    requires
        start <= end,
        end <= input.len(),
    ensures
        forall|field_index: int|
            0 <= field_index < fields@.len() ==> {
                let field: (usize, usize) = #[trigger] fields@[field_index];
                start <= field.0 <= field.1 <= end
            },
{
    let mut fields = Vec::new();
    let mut field_start = start;
    let mut index = start;
    while index < end
        invariant
            start <= field_start <= index <= end,
            end <= input@.len(),
            forall|prior: int|
                0 <= prior < fields@.len() ==> {
                    let field: (usize, usize) = #[trigger] fields@[prior];
                    start <= field.0 <= field.1 <= end
                },
        decreases end - index,
    {
        if input[index] == b'\t' {
            fields.push((field_start, index));
            field_start = index + 1;
        }
        index += 1;
    }
    fields.push((field_start, end));
    fields
}

fn field_kind(input: &[u8], start: usize, end: usize) -> (kind: Option<BoundaryKind>)
    requires
        start <= end,
        end <= input.len(),
{
    classify_identifier(input, start, end)
}

pub fn parse_ledger(input: &[u8]) -> (result: Result<Vec<LedgerEntry>, AuditError>) {
    let header = b"crucible-tcb-ledger\t1";
    let mut first_end = 0;
    while first_end < input.len() && input[first_end] != b'\n'
        invariant
            first_end <= input@.len(),
        decreases input.len() - first_end,
    {
        first_end += 1;
    }
    if !bytes_equal_range(input, 0, first_end, header) {
        return Err(AuditError::MalformedLedger);
    }
    let mut entries = Vec::new();
    let mut line_start = if first_end < input.len() {
        first_end + 1
    } else {
        first_end
    };
    while line_start < input.len()
        invariant
            line_start <= input@.len(),
        decreases input.len() - line_start,
    {
        let mut line_end = line_start;
        while line_end < input.len() && input[line_end] != b'\n'
            invariant
                line_start <= line_end <= input@.len(),
            decreases input.len() - line_end,
        {
            line_end += 1;
        }
        if line_end == line_start {
            return Err(AuditError::MalformedLedger);
        }
        let fields = split_fields(input, line_start, line_end);
        if fields.len() != 13 {
            return Err(AuditError::MalformedLedger);
        }
        let mut field_index = 0;
        while field_index < fields.len()
            invariant
                field_index <= fields@.len(),
            decreases fields.len() - field_index,
        {
            if fields[field_index].0 == fields[field_index].1 {
                return Err(AuditError::MalformedLedger);
            }
            field_index += 1;
        }
        if !bytes_equal_range(input, fields[0].0, fields[0].1, b"boundary") {
            return Err(AuditError::MalformedLedger);
        }
        let kind = match field_kind(input, fields[3].0, fields[3].1) {
            Some(kind) => kind,
            None => return Err(AuditError::MalformedLedger),
        };
        entries.push(
            LedgerEntry {
                id: copy_range(input, fields[1].0, fields[1].1),
                source_path: copy_range(input, fields[2].0, fields[2].1),
                kind,
                component: copy_range(input, fields[4].0, fields[4].1),
                reason: copy_range(input, fields[5].0, fields[5].1),
                assumption: copy_range(input, fields[6].0, fields[6].1),
                relied_on_property: copy_range(input, fields[7].0, fields[7].1),
                independent_checks: copy_range(input, fields[8].0, fields[8].1),
                reviewer: copy_range(input, fields[9].0, fields[9].1),
                approval: copy_range(input, fields[10].0, fields[10].1),
                upstream_limitation: copy_range(input, fields[11].0, fields[11].1),
                review_trigger: copy_range(input, fields[12].0, fields[12].1),
            },
        );
        line_start =
        if line_end < input.len() {
            line_end + 1
        } else {
            line_end
        };
    }
    Ok(entries)
}

#[expect(clippy::manual_range_contains, reason = "arithmetic spelling mirrors the Verus specification and proof obligations")]
fn parse_bounded_usize(input: &[u8], start: usize, end: usize) -> (value: Option<usize>)
    requires
        start <= end,
        end <= input.len(),
{
    if start == end {
        return None;
    }
    let mut value = 0usize;
    let mut index = start;
    while index < end
        invariant
            start <= index <= end,
            end <= input@.len(),
            value <= 1_000_000_009,
        decreases end - index,
    {
        let byte = input[index];
        if byte < b'0' || byte > b'9' || value > 100_000_000 {
            return None;
        }
        let digit = (byte - b'0') as usize;
        value = value * 10 + digit;
        index += 1;
    }
    Some(value)
}

pub fn parse_approvals(input: &[u8]) -> (result: Result<Vec<ApprovalEntry>, AuditError>) {
    let header = b"crucible-tcb-approvals\t1";
    let mut first_end = 0;
    while first_end < input.len() && input[first_end] != b'\n'
        invariant
            first_end <= input@.len(),
        decreases input.len() - first_end,
    {
        first_end += 1;
    }
    if !bytes_equal_range(input, 0, first_end, header) {
        return Err(AuditError::MalformedApprovalBaseline);
    }
    let mut entries = Vec::new();
    let mut line_start = if first_end < input.len() {
        first_end + 1
    } else {
        first_end
    };
    while line_start < input.len()
        invariant
            line_start <= input@.len(),
        decreases input.len() - line_start,
    {
        let mut line_end = line_start;
        while line_end < input.len() && input[line_end] != b'\n'
            invariant
                line_start <= line_end <= input@.len(),
            decreases input.len() - line_end,
        {
            line_end += 1;
        }
        if line_end == line_start {
            return Err(AuditError::MalformedApprovalBaseline);
        }
        let fields = split_fields(input, line_start, line_end);
        if fields.len() != 16 {
            return Err(AuditError::MalformedApprovalBaseline);
        }
        let mut field_index = 0;
        while field_index < fields.len()
            invariant
                field_index <= fields@.len(),
            decreases fields.len() - field_index,
        {
            if fields[field_index].0 == fields[field_index].1 {
                return Err(AuditError::MalformedApprovalBaseline);
            }
            field_index += 1;
        }
        if !bytes_equal_range(input, fields[0].0, fields[0].1, b"approved") {
            return Err(AuditError::MalformedApprovalBaseline);
        }
        let kind = match field_kind(input, fields[3].0, fields[3].1) {
            Some(kind) => kind,
            None => return Err(AuditError::MalformedApprovalBaseline),
        };
        let occurrence_line = parse_bounded_usize(input, fields[13].0, fields[13].1).ok_or(
            AuditError::MalformedApprovalBaseline,
        )?;
        let source_bytes = parse_bounded_usize(input, fields[14].0, fields[14].1).ok_or(
            AuditError::MalformedApprovalBaseline,
        )?;
        let source_lines = parse_bounded_usize(input, fields[15].0, fields[15].1).ok_or(
            AuditError::MalformedApprovalBaseline,
        )?;
        entries.push(
            ApprovalEntry {
                id: copy_range(input, fields[1].0, fields[1].1),
                source_path: copy_range(input, fields[2].0, fields[2].1),
                kind,
                component: copy_range(input, fields[4].0, fields[4].1),
                reason: copy_range(input, fields[5].0, fields[5].1),
                assumption: copy_range(input, fields[6].0, fields[6].1),
                relied_on_property: copy_range(input, fields[7].0, fields[7].1),
                independent_checks: copy_range(input, fields[8].0, fields[8].1),
                reviewer: copy_range(input, fields[9].0, fields[9].1),
                approval: copy_range(input, fields[10].0, fields[10].1),
                upstream_limitation: copy_range(input, fields[11].0, fields[11].1),
                review_trigger: copy_range(input, fields[12].0, fields[12].1),
                occurrence_line,
                source_bytes,
                source_lines,
            },
        );
        line_start =
        if line_end < input.len() {
            line_end + 1
        } else {
            line_end
        };
    }
    Ok(entries)
}

fn count_source_lines(input: &[u8]) -> (lines: usize) {
    let mut lines = 1usize;
    let mut index = 0;
    while index < input.len()
        invariant
            index <= input@.len(),
            1 <= lines <= index + 1,
        decreases input.len() - index,
    {
        if input[index] == b'\n' {
            if lines == usize::MAX {
                return lines;
            }
            lines += 1;
        }
        index += 1;
    }
    lines
}

fn ledger_matches_approval(ledger: &LedgerEntry, approval: &ApprovalEntry) -> bool {
    ledger.id == approval.id && ledger.source_path == approval.source_path && ledger.kind
        == approval.kind && ledger.component == approval.component && ledger.reason
        == approval.reason && ledger.assumption == approval.assumption && ledger.relied_on_property
        == approval.relied_on_property && ledger.independent_checks == approval.independent_checks
        && ledger.reviewer == approval.reviewer && ledger.approval == approval.approval
        && ledger.upstream_limitation == approval.upstream_limitation && ledger.review_trigger
        == approval.review_trigger
}

fn count_boundary_kind(occurrences: &[BoundaryOccurrence], kind: BoundaryKind) -> (count: usize) {
    let mut index = 0;
    let mut count = 0;
    while index < occurrences.len()
        invariant
            index <= occurrences@.len(),
            count <= index,
        decreases occurrences.len() - index,
    {
        if occurrences[index].kind == kind {
            count += 1;
        }
        index += 1;
    }
    count
}

pub fn reconcile_boundaries(
    sources: &[SourceFile],
    ledger: &[LedgerEntry],
    approvals: &[ApprovalEntry],
) -> (result: Result<AuditSummary, AuditError>) {
    let mut left = 0;
    while left < ledger.len()
        invariant
            left <= ledger@.len(),
        decreases ledger.len() - left,
    {
        let mut right = left + 1;
        while right < ledger.len()
            invariant
                left < ledger@.len(),
                left < right <= ledger@.len(),
            decreases ledger.len() - right,
        {
            if ledger[left].id == ledger[right].id {
                return Err(AuditError::DuplicateLedgerId);
            }
            right += 1;
        }
        left += 1;
    }

    left = 0;
    while left < approvals.len()
        invariant
            left <= approvals@.len(),
        decreases approvals.len() - left,
    {
        let mut right = left + 1;
        while right < approvals.len()
            invariant
                left < approvals@.len(),
                left < right <= approvals@.len(),
            decreases approvals.len() - right,
        {
            if approvals[left].id == approvals[right].id {
                return Err(AuditError::DuplicateApprovalId);
            }
            right += 1;
        }
        left += 1;
    }

    let occurrences = scan_boundaries(sources)?;
    let mut occurrence_index = 0;
    while occurrence_index < occurrences.len()
        invariant
            occurrence_index <= occurrences@.len(),
        decreases occurrences.len() - occurrence_index,
    {
        let occurrence = &occurrences[occurrence_index];
        let mut ledger_index = 0;
        while ledger_index < ledger.len() && ledger[ledger_index].id != occurrence.id
            invariant
                ledger_index <= ledger@.len(),
            decreases ledger.len() - ledger_index,
        {
            ledger_index += 1;
        }
        if ledger_index == ledger.len() {
            return Err(AuditError::MissingLedgerEntry);
        }
        let mut approval_index = 0;
        while approval_index < approvals.len() && approvals[approval_index].id != occurrence.id
            invariant
                approval_index <= approvals@.len(),
            decreases approvals.len() - approval_index,
        {
            approval_index += 1;
        }
        if approval_index == approvals.len() {
            return Err(AuditError::MissingApprovalEntry);
        }
        let ledger_entry = &ledger[ledger_index];
        let approval = &approvals[approval_index];
        if ledger_entry.source_path != occurrence.source_path || ledger_entry.kind
            != occurrence.kind || !ledger_matches_approval(ledger_entry, approval)
            || ledger_entry.approval != b"approved" || approval.occurrence_line != occurrence.line {
            return Err(AuditError::ApprovalMetadataMismatch);
        }
        let mut source_index = 0;
        while source_index < sources.len() && sources[source_index].path != occurrence.source_path
            invariant
                source_index <= sources@.len(),
            decreases sources.len() - source_index,
        {
            source_index += 1;
        }
        if source_index == sources.len() || approvals[approval_index].source_bytes
            != sources[source_index].contents.len() || approvals[approval_index].source_lines
            != count_source_lines(&sources[source_index].contents) {
            return Err(AuditError::ApprovalMetadataMismatch);
        }
        occurrence_index += 1;
    }

    let mut ledger_index = 0;
    while ledger_index < ledger.len()
        invariant
            ledger_index <= ledger@.len(),
        decreases ledger.len() - ledger_index,
    {
        let mut found = false;
        let mut index = 0;
        while index < occurrences.len()
            invariant
                index <= occurrences@.len(),
                ledger_index < ledger@.len(),
            decreases occurrences.len() - index,
        {
            if occurrences[index].id == ledger[ledger_index].id {
                found = true;
            }
            index += 1;
        }
        if !found {
            return Err(AuditError::StaleLedgerEntry);
        }
        ledger_index += 1;
    }

    let mut approval_index = 0;
    while approval_index < approvals.len()
        invariant
            approval_index <= approvals@.len(),
        decreases approvals.len() - approval_index,
    {
        let mut found = false;
        let mut index = 0;
        while index < occurrences.len()
            invariant
                index <= occurrences@.len(),
                approval_index < approvals@.len(),
            decreases occurrences.len() - index,
        {
            if occurrences[index].id == approvals[approval_index].id {
                found = true;
            }
            index += 1;
        }
        if !found {
            return Err(AuditError::StaleApprovalEntry);
        }
        approval_index += 1;
    }

    Ok(
        AuditSummary {
            registered: occurrences.len(),
            external_body_entries: count_boundary_kind(&occurrences, BoundaryKind::ExternalBody),
            external_entries: count_boundary_kind(&occurrences, BoundaryKind::External),
            external_type_specification_entries: count_boundary_kind(
                &occurrences,
                BoundaryKind::ExternalTypeSpecification,
            ),
            assume_specification_entries: count_boundary_kind(
                &occurrences,
                BoundaryKind::AssumeSpecification,
            ),
            assume_entries: count_boundary_kind(&occurrences, BoundaryKind::Assume),
            admit_entries: count_boundary_kind(&occurrences, BoundaryKind::Admit),
            axiom_entries: count_boundary_kind(&occurrences, BoundaryKind::Axiom),
            unsafe_entries: count_boundary_kind(&occurrences, BoundaryKind::Unsafe),
            foreign_entries: count_boundary_kind(&occurrences, BoundaryKind::Foreign),
            included_source_entries: count_boundary_kind(
                &occurrences,
                BoundaryKind::IncludedSource,
            ),
            unregistered: 0,
            unapproved_growth: 0,
            source_files: sources.len(),
        },
    )
}

} // verus!
