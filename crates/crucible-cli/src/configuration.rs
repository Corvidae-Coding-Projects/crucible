//! Verus-written production bridge from untrusted Crucible YAML bytes to a canonical effective
//! configuration digest.
//!
//! The generic YAML machines remain in `crucible-yaml`; this module only composes them, applies the
//! concrete configuration-schema version, checks execution-facing invariants, renders the exact
//! canonical YAML form, and authenticates those bytes with the project-owned SHA-256.
use crucible_core::{sha256, Sha256Digest};
use crucible_yaml::{
    analyze_profile1_layout, atomize_profile1, bind_profile1_typed_yaml_value,
    canonical_alias_cycle_limits, canonical_block_scalar_limits, canonical_completed_token_limits,
    canonical_duplicate_key_limits, canonical_lowering_limits, canonical_merge_expansion_limits,
    canonical_plain_scalar_limits, canonical_quoted_scalar_limits, canonical_scalar_key_limits,
    canonical_semantic_node_table_limits, canonical_semantic_scalar_table_limits,
    canonical_semantic_topology_limits, canonical_structural_key_limits,
    canonical_structural_layout_limits, canonical_structural_scan_limits,
    canonical_typed_field_schema_limits, canonical_typed_mapping_field_limits,
    compile_typed_field_schema, compose_profile1_canonical_structural_keys, decode_profile1,
    expand_profile1_merge_keys, lower_profile1_canonical_graph, parse_profile1_cst,
    partition_profile1_typed_mapping_fields, reject_profile1_duplicate_keys,
    scan_profile1_block_scalars, scan_profile1_completed_tokens, scan_profile1_plain_scalars,
    scan_profile1_quoted_scalars, scan_profile1_structural_lexemes, AnchorAliasLimits,
    AtomizeLimits, BomPolicy, CanonicalYamlGraphSource, CstLimits, DecodeLimits,
    ResolvedScalarValue, TypedFieldDefinition, TypedFieldSchema, TypedMappingFieldErrorKind,
    TypedSchemaNode, TypedSchemaValueKind, TypedValueBindingErrorKind, MAX_PROFILE1_ALIAS_BINDINGS,
    MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_CST_DIRECTIVES, MAX_PROFILE1_CST_DOCUMENTS,
    MAX_PROFILE1_CST_MAPPING_ENTRIES, MAX_PROFILE1_CST_NODES, MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
    MAX_PROFILE1_CST_WARNINGS, MAX_PROFILE1_DECODED_SCALARS, MAX_PROFILE1_LEXICAL_ATOMS,
    MAX_PROFILE1_SOURCE_BYTES,
};
use vstd::prelude::*;

macro_rules! define_configuration_field_name {
    ($name:ident, $value:expr) => {
        verus! {

        fn $name() -> (bytes: Vec<u8>) {
            vstd::slice::slice_to_vec($value)
        }

        } // verus!
    };
}

define_configuration_field_name!(field_name_01, b"version");
define_configuration_field_name!(field_name_02, b"language");
define_configuration_field_name!(field_name_03, b"project");
define_configuration_field_name!(field_name_04, b"target");
define_configuration_field_name!(field_name_05, b"execution");
define_configuration_field_name!(field_name_06, b"oracles");
define_configuration_field_name!(field_name_07, b"inputs");
define_configuration_field_name!(field_name_08, b"engines");
define_configuration_field_name!(field_name_09, b"sanitizers");
define_configuration_field_name!(field_name_10, b"campaign");
define_configuration_field_name!(field_name_11, b"storage");
define_configuration_field_name!(field_name_12, b"verification");
define_configuration_field_name!(field_name_13, b"profile");
define_configuration_field_name!(field_name_14, b"name");
define_configuration_field_name!(field_name_15, b"adapter");
define_configuration_field_name!(field_name_16, b"command");
define_configuration_field_name!(field_name_17, b"args");
define_configuration_field_name!(field_name_18, b"timeout_ms");
define_configuration_field_name!(field_name_19, b"memory_mb");
define_configuration_field_name!(field_name_20, b"max_processes");
define_configuration_field_name!(field_name_21, b"max_output_mb");
define_configuration_field_name!(field_name_22, b"network");
define_configuration_field_name!(field_name_23, b"required_capabilities");
define_configuration_field_name!(field_name_24, b"process_exit");
define_configuration_field_name!(field_name_25, b"allowed_codes");
define_configuration_field_name!(field_name_26, b"timeout_is_failure");
define_configuration_field_name!(field_name_27, b"corpus");
define_configuration_field_name!(field_name_28, b"fuzz");
define_configuration_field_name!(field_name_29, b"property");
define_configuration_field_name!(field_name_30, b"differential");
define_configuration_field_name!(field_name_31, b"metamorphic");
define_configuration_field_name!(field_name_32, b"fault");
define_configuration_field_name!(field_name_33, b"concurrency");
define_configuration_field_name!(field_name_34, b"symbolic");
define_configuration_field_name!(field_name_35, b"mutation");
define_configuration_field_name!(field_name_36, b"enabled");
define_configuration_field_name!(field_name_37, b"modes");
define_configuration_field_name!(field_name_38, b"native_backends");
define_configuration_field_name!(field_name_39, b"enabled");
define_configuration_field_name!(field_name_40, b"address");
define_configuration_field_name!(field_name_41, b"undefined");
define_configuration_field_name!(field_name_42, b"thread");
define_configuration_field_name!(field_name_43, b"memory");
define_configuration_field_name!(field_name_44, b"leak");
define_configuration_field_name!(field_name_45, b"duration");
define_configuration_field_name!(field_name_46, b"workers");
define_configuration_field_name!(field_name_47, b"seed");
define_configuration_field_name!(field_name_48, b"root");
define_configuration_field_name!(field_name_49, b"verus");
define_configuration_field_name!(field_name_50, b"required");
define_configuration_field_name!(field_name_51, b"deny_unregistered_assumptions");
define_configuration_field_name!(field_name_52, b"deny_unapproved_tcb_growth");
define_configuration_field_name!(field_name_invalid, b"");

verus! {

pub const CONFIGURATION_SCHEMA_VERSION: u16 = 1;

pub const CONFIGURATION_CANONICALIZATION_VERSION: u16 = 1;

pub const MAX_CONFIGURATION_SOURCE_BYTES: u64 = MAX_PROFILE1_SOURCE_BYTES;

pub const MAX_CONFIGURATION_TYPED_NODES: u64 = 1_048_576;

pub const MAX_CONFIGURATION_CANONICAL_BYTES: u64 = 16 * 1024 * 1024;

pub const MAX_CONFIGURATION_DEPTH: u64 = 4_096;

pub const MAX_CONFIGURATION_RENDER_TASKS: u64 = 4_194_304;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigurationLimits {
    max_source_bytes: u64,
    max_typed_nodes: u64,
    max_canonical_bytes: u64,
    max_depth: u64,
    max_work: u64,
}

#[verifier::ext_equal]
pub struct ConfigurationLimitsView {
    pub max_source_bytes: u64,
    pub max_typed_nodes: u64,
    pub max_canonical_bytes: u64,
    pub max_depth: u64,
    pub max_work: u64,
}

impl View for ConfigurationLimits {
    type V = ConfigurationLimitsView;

    closed spec fn view(&self) -> ConfigurationLimitsView {
        ConfigurationLimitsView {
            max_source_bytes: self.max_source_bytes,
            max_typed_nodes: self.max_typed_nodes,
            max_canonical_bytes: self.max_canonical_bytes,
            max_depth: self.max_depth,
            max_work: self.max_work,
        }
    }
}

impl ConfigurationLimits {
    pub fn new(
        max_source_bytes: u64,
        max_typed_nodes: u64,
        max_canonical_bytes: u64,
        max_depth: u64,
    ) -> (limits: Self)
        ensures
            limits@ == (ConfigurationLimitsView {
                max_source_bytes,
                max_typed_nodes,
                max_canonical_bytes,
                max_depth,
                max_work: MAX_CONFIGURATION_RENDER_TASKS,
            }),
    {
        Self {
            max_source_bytes,
            max_typed_nodes,
            max_canonical_bytes,
            max_depth,
            max_work: MAX_CONFIGURATION_RENDER_TASKS,
        }
    }

    pub fn max_source_bytes(&self) -> (value: u64)
        ensures
            value == self@.max_source_bytes,
    {
        self.max_source_bytes
    }

    pub fn max_typed_nodes(&self) -> (value: u64)
        ensures
            value == self@.max_typed_nodes,
    {
        self.max_typed_nodes
    }

    pub fn max_canonical_bytes(&self) -> (value: u64)
        ensures
            value == self@.max_canonical_bytes,
    {
        self.max_canonical_bytes
    }

    pub fn max_depth(&self) -> (value: u64)
        ensures
            value == self@.max_depth,
    {
        self.max_depth
    }

    pub fn max_work(&self) -> (value: u64)
        ensures
            value == self@.max_work,
    {
        self.max_work
    }

    pub fn with_max_work(self, max_work: u64) -> (limits: Self)
        ensures
            limits@ == (ConfigurationLimitsView {
                max_source_bytes: self@.max_source_bytes,
                max_typed_nodes: self@.max_typed_nodes,
                max_canonical_bytes: self@.max_canonical_bytes,
                max_depth: self@.max_depth,
                max_work,
            }),
    {
        Self {
            max_source_bytes: self.max_source_bytes,
            max_typed_nodes: self.max_typed_nodes,
            max_canonical_bytes: self.max_canonical_bytes,
            max_depth: self.max_depth,
            max_work,
        }
    }
}

pub fn canonical_configuration_limits() -> (limits: ConfigurationLimits)
    ensures
        limits@ == (ConfigurationLimitsView {
            max_source_bytes: MAX_CONFIGURATION_SOURCE_BYTES,
            max_typed_nodes: MAX_CONFIGURATION_TYPED_NODES,
            max_canonical_bytes: MAX_CONFIGURATION_CANONICAL_BYTES,
            max_depth: MAX_CONFIGURATION_DEPTH,
            max_work: MAX_CONFIGURATION_RENDER_TASKS,
        }),
{
    ConfigurationLimits::new(
        MAX_CONFIGURATION_SOURCE_BYTES,
        MAX_CONFIGURATION_TYPED_NODES,
        MAX_CONFIGURATION_CANONICAL_BYTES,
        MAX_CONFIGURATION_DEPTH,
    )
}

pub open spec fn configuration_effective_limit_spec(requested: u64, absolute: u64) -> u64 {
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

fn effective_limit(requested: u64, absolute: u64) -> (value: u64)
    ensures
        value == configuration_effective_limit_spec(requested, absolute),
        value <= absolute,
{
    if requested < absolute {
        requested
    } else {
        absolute
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Structural)]
#[non_exhaustive]
pub enum ConfigurationErrorKind {
    SourceByteLimitExceeded,
    YamlSyntax,
    ExpectedSingleDocument,
    ExpectedRootMapping,
    UnknownField,
    MissingRequiredField,
    WrongValueKind,
    UnsupportedSchemaVersion,
    InvalidLanguageProfile,
    InvalidTargetAdapter,
    InvalidFieldValue,
    IntegerOutOfRange,
    DuplicateSequenceValue,
    CrossFieldInvariant,
    TypedNodeLimitExceeded,
    CanonicalByteLimitExceeded,
    DepthLimitExceeded,
    WorkLimitExceeded,
    HashInputTooLong,
    InternalInvariantViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigurationError {
    kind: ConfigurationErrorKind,
    byte_offset: u64,
    typed_node_index: Option<u64>,
    canonical_byte_index: Option<u64>,
}

#[verifier::ext_equal]
pub struct ConfigurationErrorView {
    pub kind: ConfigurationErrorKind,
    pub byte_offset: u64,
    pub typed_node_index: Option<u64>,
    pub canonical_byte_index: Option<u64>,
}

impl View for ConfigurationError {
    type V = ConfigurationErrorView;

    closed spec fn view(&self) -> ConfigurationErrorView {
        ConfigurationErrorView {
            kind: self.kind,
            byte_offset: self.byte_offset,
            typed_node_index: self.typed_node_index,
            canonical_byte_index: self.canonical_byte_index,
        }
    }
}

impl ConfigurationError {
    fn at(kind: ConfigurationErrorKind, byte_offset: u64) -> (error: Self)
        ensures
            error@ == (ConfigurationErrorView {
                kind,
                byte_offset,
                typed_node_index: None,
                canonical_byte_index: None,
            }),
    {
        Self { kind, byte_offset, typed_node_index: None, canonical_byte_index: None }
    }

    fn typed(kind: ConfigurationErrorKind, byte_offset: u64, index: u64) -> (error: Self)
        ensures
            error@ == (ConfigurationErrorView {
                kind,
                byte_offset,
                typed_node_index: Some(index),
                canonical_byte_index: None,
            }),
    {
        Self { kind, byte_offset, typed_node_index: Some(index), canonical_byte_index: None }
    }

    fn canonical(byte_offset: u64, index: u64) -> (error: Self)
        ensures
            error@ == (ConfigurationErrorView {
                kind: ConfigurationErrorKind::CanonicalByteLimitExceeded,
                byte_offset,
                typed_node_index: None,
                canonical_byte_index: Some(index),
            }),
    {
        Self {
            kind: ConfigurationErrorKind::CanonicalByteLimitExceeded,
            byte_offset,
            typed_node_index: None,
            canonical_byte_index: Some(index),
        }
    }

    pub fn kind(&self) -> (value: ConfigurationErrorKind)
        ensures
            value == self@.kind,
    {
        self.kind
    }

    pub fn byte_offset(&self) -> (value: u64)
        ensures
            value == self@.byte_offset,
    {
        self.byte_offset
    }

    pub fn typed_node_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.typed_node_index,
    {
        self.typed_node_index
    }

    pub fn canonical_byte_index(&self) -> (value: Option<u64>)
        ensures
            value == self@.canonical_byte_index,
    {
        self.canonical_byte_index
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedConfiguration {
    schema_version: u16,
    canonicalization_version: u16,
    source_digest: Sha256Digest,
    canonical_bytes: Vec<u8>,
    digest: Sha256Digest,
    typed_node_count: u64,
    work_count: u64,
}

#[verifier::ext_equal]
pub struct ValidatedConfigurationView {
    pub schema_version: u16,
    pub canonicalization_version: u16,
    pub source_digest: Seq<u8>,
    pub canonical_bytes: Seq<u8>,
    pub digest: Seq<u8>,
    pub typed_node_count: u64,
    pub work_count: u64,
}

impl View for ValidatedConfiguration {
    type V = ValidatedConfigurationView;

    closed spec fn view(&self) -> ValidatedConfigurationView {
        ValidatedConfigurationView {
            schema_version: self.schema_version,
            canonicalization_version: self.canonicalization_version,
            source_digest: self.source_digest@,
            canonical_bytes: self.canonical_bytes@,
            digest: self.digest@,
            typed_node_count: self.typed_node_count,
            work_count: self.work_count,
        }
    }
}

impl ValidatedConfiguration {
    fn new(
        source_digest: Sha256Digest,
        canonical_bytes: Vec<u8>,
        digest: Sha256Digest,
        typed_node_count: u64,
        work_count: u64,
    ) -> (configuration: Self)
        ensures
            configuration@ == (ValidatedConfigurationView {
                schema_version: CONFIGURATION_SCHEMA_VERSION,
                canonicalization_version: CONFIGURATION_CANONICALIZATION_VERSION,
                source_digest: source_digest@,
                canonical_bytes: canonical_bytes@,
                digest: digest@,
                typed_node_count,
                work_count,
            }),
    {
        Self {
            schema_version: CONFIGURATION_SCHEMA_VERSION,
            canonicalization_version: CONFIGURATION_CANONICALIZATION_VERSION,
            source_digest,
            canonical_bytes,
            digest,
            typed_node_count,
            work_count,
        }
    }

    pub fn schema_version(&self) -> (value: u16)
        ensures
            value == self@.schema_version,
    {
        self.schema_version
    }

    pub fn canonicalization_version(&self) -> (value: u16)
        ensures
            value == self@.canonicalization_version,
    {
        self.canonicalization_version
    }

    pub fn source_digest(&self) -> (value: Sha256Digest)
        ensures
            value@ == self@.source_digest,
    {
        self.source_digest
    }

    pub fn canonical_bytes(&self) -> (value: &[u8])
        ensures
            value@ == self@.canonical_bytes,
    {
        self.canonical_bytes.as_slice()
    }

    pub fn digest(&self) -> (value: Sha256Digest)
        ensures
            value@ == self@.digest,
    {
        self.digest
    }

    pub fn typed_node_count(&self) -> (value: u64)
        ensures
            value == self@.typed_node_count,
    {
        self.typed_node_count
    }

    pub fn work_count(&self) -> (value: u64)
        ensures
            value == self@.work_count,
    {
        self.work_count
    }
}

/// Integrity and resource facts currently proved about an accepted configuration.
///
/// The stronger executable-to-pure schema/canonicalization equivalence required by §12.2 remains
/// a separate proof milestone; this predicate is deliberately not named as that semantic proof.
pub open spec fn validated_configuration_integrity_spec(
    input: Seq<u8>,
    limits: ConfigurationLimitsView,
    valid: ValidatedConfigurationView,
) -> bool {
    valid.schema_version == CONFIGURATION_SCHEMA_VERSION && valid.canonicalization_version
        == CONFIGURATION_CANONICALIZATION_VERSION && input.len()
        <= configuration_effective_limit_spec(
        limits.max_source_bytes,
        MAX_CONFIGURATION_SOURCE_BYTES,
    ) && valid.typed_node_count <= configuration_effective_limit_spec(
        limits.max_typed_nodes,
        MAX_CONFIGURATION_TYPED_NODES,
    ) && valid.work_count <= configuration_effective_limit_spec(
        limits.max_work,
        MAX_CONFIGURATION_RENDER_TASKS,
    ) && valid.canonical_bytes.len() <= configuration_effective_limit_spec(
        limits.max_canonical_bytes,
        MAX_CONFIGURATION_CANONICAL_BYTES,
    ) && crucible_core::artifact::sha256_input_supported(input.len() as nat)
        && crucible_core::artifact::sha256_input_supported(valid.canonical_bytes.len() as nat)
        && valid.source_digest == crucible_core::artifact::sha256_spec(input) && valid.digest
        == crucible_core::artifact::sha256_spec(valid.canonical_bytes)
}

pub proof fn lemma_validated_configuration_obeys_absolute_limits(
    input: Seq<u8>,
    limits: ConfigurationLimitsView,
    valid: ValidatedConfigurationView,
)
    requires
        validated_configuration_integrity_spec(input, limits, valid),
    ensures
        input.len() <= MAX_CONFIGURATION_SOURCE_BYTES,
        valid.canonical_bytes.len() <= MAX_CONFIGURATION_CANONICAL_BYTES,
        valid.typed_node_count <= MAX_CONFIGURATION_TYPED_NODES,
        valid.work_count <= MAX_CONFIGURATION_RENDER_TASKS,
{
}

} // verus!
verus! {

fn render_effective_configuration(
    graph: &CanonicalYamlGraphSource,
    schema: &crucible_yaml::CompiledTypedFieldSchema,
    root_node_index: u64,
    max_typed_nodes: u64,
    max_canonical_bytes: u64,
    max_depth: u64,
    max_work: u64,
) -> (result: Result<(Vec<u8>, u64, u64), ConfigurationError>)
    ensures
        match result {
            Ok((bytes, typed_nodes, work)) => bytes@.len() <= max_canonical_bytes && typed_nodes
                <= max_typed_nodes && work <= max_work,
            Err(_) => true,
        },
{
    let mut output = Vec::new();
    let mut state = RenderState::new();
    let mut tasks = Vec::new();
    match push_render_task(
        &mut tasks,
        RenderTask::Value {
            yaml_node_index: root_node_index,
            schema_node_index: schema.root_schema_node_index(),
            field_id: 0,
            depth: 1,
            anchor: 0,
        },
        0,
    ) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    let mut work = 0u64;
    while !tasks.is_empty() && work < max_work
        invariant
            work <= max_work,
        decreases max_work - work,
    {
        let task = match tasks.pop() {
            Some(value) => value,
            None => break,
        };
        work += 1;
        let anchor = task.anchor();
        match task {
            RenderTask::Byte { byte, .. } => {
                match push_output_byte(&mut output, byte, anchor, max_canonical_bytes) {
                    Ok(()) => {},
                    Err(error) => return Err(error),
                }
            },
            RenderTask::FieldName { schema_field_index, field_id, .. } => {
                let fields = schema.schema().fields();
                if schema_field_index >= fields.len() as u64 {
                    return Err(
                        ConfigurationError::at(
                            ConfigurationErrorKind::InternalInvariantViolation,
                            anchor,
                        ),
                    );
                }
                state.field_anchor(field_id, anchor);
                let name = fields[schema_field_index as usize].name();
                match append_quoted_points(&mut output, name, anchor, max_canonical_bytes) {
                    Ok(()) => {},
                    Err(error) => return Err(error),
                }
            },
            RenderTask::Value { yaml_node_index, schema_node_index, field_id, depth, .. } => {
                if depth > max_depth {
                    return Err(
                        ConfigurationError::at(ConfigurationErrorKind::DepthLimitExceeded, anchor),
                    );
                }
                if state.typed_node_count >= max_typed_nodes {
                    return Err(
                        ConfigurationError::typed(
                            ConfigurationErrorKind::TypedNodeLimitExceeded,
                            anchor,
                            state.typed_node_count,
                        ),
                    );
                }
                state.typed_node_count += 1;
                let binding = match bind_profile1_typed_yaml_value(
                    graph,
                    schema,
                    yaml_node_index,
                    schema_node_index,
                ) {
                    Ok(value) => value,
                    Err(error) => return Err(map_binding_error(error)),
                };
                let schema_nodes = schema.schema().nodes();
                let graph_nodes = graph.nodes();
                if schema_node_index >= schema_nodes.len() as u64 {
                    return Err(
                        ConfigurationError::at(
                            ConfigurationErrorKind::InternalInvariantViolation,
                            anchor,
                        ),
                    );
                }
                if yaml_node_index >= graph_nodes.len() as u64 {
                    return Err(
                        ConfigurationError::at(
                            ConfigurationErrorKind::InternalInvariantViolation,
                            anchor,
                        ),
                    );
                }
                let schema_node_position = schema_node_index as usize;
                let yaml_node_position = yaml_node_index as usize;
                assert(schema_node_position < schema_nodes.len());
                assert(yaml_node_position < graph_nodes.len());
                let schema_node = &schema_nodes[schema_node_position];
                let yaml_node = &graph_nodes[yaml_node_position];
                match schema_node.kind() {
                    TypedSchemaValueKind::Mapping => {
                        let partition = match partition_profile1_typed_mapping_fields(
                            graph,
                            schema,
                            yaml_node_index,
                            schema_node_index,
                            canonical_typed_mapping_field_limits(),
                        ) {
                            Ok(value) => value,
                            Err(error) => return Err(map_mapping_error(error)),
                        };
                        match push_output_byte(&mut output, b'{', anchor, max_canonical_bytes) {
                            Ok(()) => {},
                            Err(error) => return Err(error),
                        }
                        match push_render_task(
                            &mut tasks,
                            RenderTask::Byte { byte: b'}', anchor },
                            anchor,
                        ) {
                            Ok(()) => {},
                            Err(error) => return Err(error),
                        }
                        let partition_fields = partition.fields();
                        let schema_fields = schema.schema().fields();
                        let mut index = partition_fields.len();
                        while index > 0
                            invariant
                                index <= partition_fields.len(),
                            decreases index,
                        {
                            index -= 1;
                            let field = &partition_fields[index];
                            let schema_field_index = field.schema_field_index();
                            if schema_field_index >= schema_fields.len() as u64 {
                                return Err(
                                    ConfigurationError::at(
                                        ConfigurationErrorKind::InternalInvariantViolation,
                                        anchor,
                                    ),
                                );
                            }
                            let definition = &schema_fields[schema_field_index as usize];
                            let key_node_index = field.key_yaml_node_index();
                            let key_node_position = key_node_index as usize;
                            let field_anchor = if key_node_position < graph_nodes.len()
                                && key_node_position as u64 == key_node_index {
                                graph_nodes[key_node_position].byte_start()
                            } else {
                                anchor
                            };
                            let value_node_index = field.value_yaml_node_index();
                            let value_node_position = value_node_index as usize;
                            let value_anchor = if value_node_position < graph_nodes.len()
                                && value_node_position as u64 == value_node_index {
                                graph_nodes[value_node_position].byte_start()
                            } else {
                                field_anchor
                            };
                            if depth == u64::MAX {
                                return Err(
                                    ConfigurationError::at(
                                        ConfigurationErrorKind::DepthLimitExceeded,
                                        value_anchor,
                                    ),
                                );
                            }
                            let child_depth = depth + 1;
                            match push_render_task(
                                &mut tasks,
                                RenderTask::Value {
                                    yaml_node_index: value_node_index,
                                    schema_node_index: definition.value_schema_node_index(),
                                    field_id: field.field_id(),
                                    depth: child_depth,
                                    anchor: value_anchor,
                                },
                                value_anchor,
                            ) {
                                Ok(()) => {},
                                Err(error) => return Err(error),
                            }
                            match push_render_task(
                                &mut tasks,
                                RenderTask::Byte { byte: b':', anchor: field_anchor },
                                field_anchor,
                            ) {
                                Ok(()) => {},
                                Err(error) => return Err(error),
                            }
                            match push_render_task(
                                &mut tasks,
                                RenderTask::FieldName {
                                    schema_field_index,
                                    field_id: field.field_id(),
                                    anchor: field_anchor,
                                },
                                field_anchor,
                            ) {
                                Ok(()) => {},
                                Err(error) => return Err(error),
                            }
                            if index > 0 {
                                match push_render_task(
                                    &mut tasks,
                                    RenderTask::Byte { byte: b',', anchor: field_anchor },
                                    field_anchor,
                                ) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                            }
                        }
                    },
                    TypedSchemaValueKind::Sequence => {
                        let item_schema = match schema_node.sequence_item_schema_node_index() {
                            Some(value) => value,
                            None => return Err(
                                ConfigurationError::at(
                                    ConfigurationErrorKind::InternalInvariantViolation,
                                    anchor,
                                ),
                            ),
                        };
                        let start = yaml_node.edge_start();
                        let end = yaml_node.edge_end();
                        let entries = graph.sequence_entries();
                        let start_position = start as usize;
                        let end_position = end as usize;
                        if start_position as u64 != start || end_position as u64 != end
                            || start_position > end_position || end_position > entries.len() {
                            return Err(
                                ConfigurationError::at(
                                    ConfigurationErrorKind::InternalInvariantViolation,
                                    anchor,
                                ),
                            );
                        }
                        match push_output_byte(&mut output, b'[', anchor, max_canonical_bytes) {
                            Ok(()) => {},
                            Err(error) => return Err(error),
                        }
                        match push_render_task(
                            &mut tasks,
                            RenderTask::Byte { byte: b']', anchor },
                            anchor,
                        ) {
                            Ok(()) => {},
                            Err(error) => return Err(error),
                        }
                        let mut index = end_position;
                        while index > start_position
                            invariant
                                start_position <= index <= end_position,
                                end_position <= entries.len(),
                            decreases index - start_position,
                        {
                            index -= 1;
                            let value_node_index = entries[index].value_node_index();
                            let value_node_position = value_node_index as usize;
                            let entry_anchor = if value_node_position < graph_nodes.len()
                                && value_node_position as u64 == value_node_index {
                                graph_nodes[value_node_position].byte_start()
                            } else {
                                anchor
                            };
                            if depth == u64::MAX {
                                return Err(
                                    ConfigurationError::at(
                                        ConfigurationErrorKind::DepthLimitExceeded,
                                        entry_anchor,
                                    ),
                                );
                            }
                            let child_depth = depth + 1;
                            match push_render_task(
                                &mut tasks,
                                RenderTask::Value {
                                    yaml_node_index: value_node_index,
                                    schema_node_index: item_schema,
                                    field_id,
                                    depth: child_depth,
                                    anchor: entry_anchor,
                                },
                                entry_anchor,
                            ) {
                                Ok(()) => {},
                                Err(error) => return Err(error),
                            }
                            if index > start_position {
                                match push_render_task(
                                    &mut tasks,
                                    RenderTask::Byte { byte: b',', anchor: entry_anchor },
                                    entry_anchor,
                                ) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                            }
                        }
                    },
                    TypedSchemaValueKind::Boolean
                    | TypedSchemaValueKind::Integer
                    | TypedSchemaValueKind::String => {
                        let scalar_index = match binding.scalar_index() {
                            Some(value) => value,
                            None => return Err(
                                ConfigurationError::at(
                                    ConfigurationErrorKind::InternalInvariantViolation,
                                    anchor,
                                ),
                            ),
                        };
                        let scalars =
                            graph.input().input().structural_keys().scalar_keys().graph().node_table().scalars().scalars();
                        if scalar_index >= scalars.len() as u64 {
                            return Err(
                                ConfigurationError::at(
                                    ConfigurationErrorKind::InternalInvariantViolation,
                                    anchor,
                                ),
                            );
                        }
                        let scalar = &scalars[scalar_index as usize];
                        match scalar.value() {
                            ResolvedScalarValue::Boolean(value) => {
                                match observe_boolean(&mut state, field_id, *value, anchor) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                                let literal: &[u8] = if *value {
                                    b"true"
                                } else {
                                    b"false"
                                };
                                match append_literal(
                                    &mut output,
                                    literal,
                                    anchor,
                                    max_canonical_bytes,
                                ) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                            },
                            ResolvedScalarValue::Integer(integer) => {
                                match observe_integer(field_id, integer, anchor) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                                match append_integer(
                                    &mut output,
                                    integer,
                                    anchor,
                                    max_canonical_bytes,
                                ) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                            },
                            ResolvedScalarValue::String => {
                                let points = copy_scalar_points(scalar)?;
                                let work_before_observation = work;
                                match observe_string(
                                    &mut state,
                                    field_id,
                                    points.as_slice(),
                                    anchor,
                                    &mut work,
                                    max_work,
                                ) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                                if work < work_before_observation || work > max_work {
                                    return Err(
                                        ConfigurationError::at(
                                            ConfigurationErrorKind::InternalInvariantViolation,
                                            anchor,
                                        ),
                                    );
                                }
                                match append_quoted_points(
                                    &mut output,
                                    points.as_slice(),
                                    anchor,
                                    max_canonical_bytes,
                                ) {
                                    Ok(()) => {},
                                    Err(error) => return Err(error),
                                }
                            },
                            _ => return Err(
                                ConfigurationError::at(
                                    ConfigurationErrorKind::WrongValueKind,
                                    anchor,
                                ),
                            ),
                        }
                    },
                    _ => return Err(
                        ConfigurationError::at(ConfigurationErrorKind::WrongValueKind, anchor),
                    ),
                }
            },
        }
    }
    if !tasks.is_empty() {
        let anchor = match tasks.last() {
            Some(task) => task.anchor(),
            None => 0,
        };
        return Err(ConfigurationError::at(ConfigurationErrorKind::WorkLimitExceeded, anchor));
    }
    match validate_cross_field_state(&state) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    match push_output_byte(&mut output, b'\n', 0, max_canonical_bytes) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    if output.len() as u64 > max_canonical_bytes || state.typed_node_count > max_typed_nodes {
        return Err(ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, 0));
    }
    Ok((output, state.typed_node_count, work))
}

pub fn validate_configuration(input: &[u8], limits: ConfigurationLimits) -> (result: Result<
    ValidatedConfiguration,
    ConfigurationError,
>)
    ensures
        match &result {
            Ok(configuration) => validated_configuration_integrity_spec(
                input@,
                limits@,
                configuration@,
            ),
            Err(_) => true,
        },
{
    let max_source_bytes = effective_limit(
        limits.max_source_bytes(),
        MAX_CONFIGURATION_SOURCE_BYTES,
    );
    let max_typed_nodes = effective_limit(limits.max_typed_nodes(), MAX_CONFIGURATION_TYPED_NODES);
    let max_canonical_bytes = effective_limit(
        limits.max_canonical_bytes(),
        MAX_CONFIGURATION_CANONICAL_BYTES,
    );
    let max_depth = effective_limit(limits.max_depth(), MAX_CONFIGURATION_DEPTH);
    let max_work = effective_limit(limits.max_work(), MAX_CONFIGURATION_RENDER_TASKS);
    if input.len() as u64 > max_source_bytes {
        return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::SourceByteLimitExceeded,
                max_source_bytes,
            ),
        );
    }
    let graph = parse_canonical_graph(input, max_source_bytes, max_depth)?;
    let roots = graph.document_roots();
    if roots.len() != 1 {
        let byte_offset = if roots.len() > 1 {
            roots[1].byte_start()
        } else {
            input.len() as u64
        };
        return Err(
            ConfigurationError::at(ConfigurationErrorKind::ExpectedSingleDocument, byte_offset),
        );
    }
    let root_node_index = roots[0].value_node_index();
    let graph_nodes = graph.nodes();
    if root_node_index >= graph_nodes.len() as u64 {
        return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::ExpectedRootMapping,
                roots[0].byte_start(),
            ),
        );
    }
    let root_node_position = root_node_index as usize;
    assert(root_node_position < graph_nodes.len());
    if graph_nodes[root_node_position].kind() != crucible_yaml::CanonicalYamlNodeKind::Mapping {
        return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::ExpectedRootMapping,
                roots[0].byte_start(),
            ),
        );
    }
    let schema = configuration_schema()?;
    let (canonical_bytes, typed_node_count, work_count) = render_effective_configuration(
        &graph,
        &schema,
        root_node_index,
        max_typed_nodes,
        max_canonical_bytes,
        max_depth,
        max_work,
    )?;
    if canonical_bytes.len() as u64 > max_canonical_bytes || typed_node_count > max_typed_nodes {
        return Err(ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, 0));
    }
    let source_digest = match sha256(input) {
        Ok(value) => value,
        Err(_) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::HashInputTooLong, input.len() as u64),
        ),
    };
    let digest = match sha256(canonical_bytes.as_slice()) {
        Ok(value) => value,
        Err(_) => return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::HashInputTooLong,
                canonical_bytes.len() as u64,
            ),
        ),
    };
    let configuration = ValidatedConfiguration::new(
        source_digest,
        canonical_bytes,
        digest,
        typed_node_count,
        work_count,
    );
    proof {
        reveal(validated_configuration_integrity_spec);
    }
    Ok(configuration)
}

} // verus!
verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderTask {
    Byte { byte: u8, anchor: u64 },
    FieldName { schema_field_index: u64, field_id: u64, anchor: u64 },
    Value { yaml_node_index: u64, schema_node_index: u64, field_id: u64, depth: u64, anchor: u64 },
}

impl RenderTask {
    fn anchor(&self) -> (anchor: u64) {
        match *self {
            RenderTask::Byte { anchor, .. } => anchor,
            RenderTask::FieldName { anchor, .. } => anchor,
            RenderTask::Value { anchor, .. } => anchor,
        }
    }
}

#[derive(Debug)]
struct RenderState {
    typed_node_count: u64,
    fuzz_enabled: Option<bool>,
    fuzz_anchor: u64,
    has_mode: bool,
    has_native_mode: bool,
    modes_anchor: u64,
    has_backend: bool,
    backends_anchor: u64,
    exclusive_sanitizer_count: u64,
    capabilities: Vec<Vec<u32>>,
    modes: Vec<Vec<u32>>,
    backends: Vec<Vec<u32>>,
}

impl RenderState {
    fn new() -> Self {
        Self {
            typed_node_count: 0,
            fuzz_enabled: None,
            fuzz_anchor: 0,
            has_mode: false,
            has_native_mode: false,
            modes_anchor: 0,
            has_backend: false,
            backends_anchor: 0,
            exclusive_sanitizer_count: 0,
            capabilities: Vec::new(),
            modes: Vec::new(),
            backends: Vec::new(),
        }
    }

    fn field_anchor(&mut self, field_id: u64, byte_offset: u64) {
        match field_id {
            36 => self.fuzz_anchor = byte_offset,
            37 => self.modes_anchor = byte_offset,
            38 => self.backends_anchor = byte_offset,
            _ => {},
        }
    }
}

fn points_equal_ascii(points: &[u32], ascii: &[u8]) -> (equal: bool) {
    if points.len() != ascii.len() {
        return false;
    }
    let mut index = 0;
    while index < points.len()
        invariant
            index <= points.len(),
            points.len() == ascii.len(),
        decreases points.len() - index,
    {
        if points[index] != ascii[index] as u32 {
            return false;
        }
        index += 1;
    }
    true
}

fn charge_work(work: &mut u64, max_work: u64, byte_offset: u64) -> (result: Result<
    (),
    ConfigurationError,
>)
    requires
        *old(work) <= max_work,
    ensures
        match result {
            Ok(()) => *final(work) == *old(work) + 1 && *final(work) <= max_work,
            Err(_) => *final(work) == *old(work),
        },
{
    if *work >= max_work {
        return Err(ConfigurationError::at(ConfigurationErrorKind::WorkLimitExceeded, byte_offset));
    }
    *work += 1;
    Ok(())
}

fn points_are_duplicate(
    values: &[Vec<u32>],
    points: &[u32],
    work: &mut u64,
    max_work: u64,
    byte_offset: u64,
) -> (result: Result<bool, ConfigurationError>)
    requires
        *old(work) <= max_work,
    ensures
        *final(work) <= max_work,
{
    let mut index = 0;
    while index < values.len()
        invariant
            index <= values.len(),
            *work <= max_work,
        decreases values.len() - index,
    {
        charge_work(work, max_work, byte_offset)?;
        let prior = values[index].as_slice();
        if prior.len() == points.len() {
            let mut point_index = 0;
            let mut equal = true;
            while point_index < points.len() && equal
                invariant
                    point_index <= points.len(),
                    prior.len() == points.len(),
                    *work <= max_work,
                decreases points.len() - point_index,
            {
                charge_work(work, max_work, byte_offset)?;
                if prior[point_index] != points[point_index] {
                    equal = false;
                }
                point_index += 1;
            }
            if equal {
                return Ok(true);
            }
        }
        index += 1;
    }
    Ok(false)
}

fn duration_is_valid(points: &[u32]) -> (valid: bool) {
    if points.len() < 2 {
        return false;
    }
    let mut digit_end = 0;
    while digit_end < points.len() && 0x30 <= points[digit_end] && points[digit_end] <= 0x39
        invariant
            digit_end <= points.len(),
        decreases points.len() - digit_end,
    {
        digit_end += 1;
    }
    if digit_end == 0 || points[0] == 0x30 {
        return false;
    }
    let suffix_len = points.len() - digit_end;
    if suffix_len == 1 {
        return points[digit_end] == 0x73 || points[digit_end] == 0x6d || points[digit_end] == 0x68
            || points[digit_end] == 0x64;
    }
    suffix_len == 2 && points[digit_end] == 0x6d && points[digit_end + 1] == 0x73
}

fn integer_as_u64(integer: &crucible_yaml::CoreInteger) -> (value: Option<u64>) {
    if integer.negative() {
        return None;
    }
    let limbs = integer.limbs();
    let mut value = 0u64;
    let mut index = limbs.len();
    while index > 0
        invariant
            index <= limbs.len(),
        decreases index,
    {
        index -= 1;
        value = value.checked_mul(crucible_yaml::CORE_INTEGER_MAGNITUDE_RADIX as u64)?;
        value = value.checked_add(limbs[index] as u64)?;
    }
    Some(value)
}

fn signed_exit_code_is_valid(integer: &crucible_yaml::CoreInteger) -> (valid: bool) {
    let limbs = integer.limbs();
    let mut magnitude = 0u64;
    let mut index = limbs.len();
    while index > 0
        invariant
            index <= limbs.len(),
        decreases index,
    {
        index -= 1;
        magnitude =
        match magnitude.checked_mul(crucible_yaml::CORE_INTEGER_MAGNITUDE_RADIX as u64) {
            Some(next) => next,
            None => return false,
        };
        magnitude =
        match magnitude.checked_add(limbs[index] as u64) {
            Some(next) => next,
            None => return false,
        };
    }
    if integer.negative() {
        magnitude <= 2_147_483_648
    } else {
        magnitude <= 2_147_483_647
    }
}

fn copy_scalar_points(scalar: &crucible_yaml::ResolvedScalar) -> (result: Result<
    Vec<u32>,
    ConfigurationError,
>) {
    let decoded = match scalar.presentation().decoded() {
        Some(value) => value,
        None => return Err(
            ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, 0),
        ),
    };
    let content = decoded.content();
    let mut points = Vec::new();
    let mut index = 0;
    while index < content.len()
        invariant
            index <= content.len(),
        decreases content.len() - index,
    {
        points.push(content[index].code_point());
        index += 1;
    }
    Ok(points)
}

fn observe_string(
    state: &mut RenderState,
    field_id: u64,
    points: &[u32],
    byte_offset: u64,
    work: &mut u64,
    max_work: u64,
) -> (result: Result<(), ConfigurationError>)
    requires
        *old(work) <= max_work,
{
    match field_id {
        13 => if !points_equal_ascii(points, b"crucible-yaml-1") {
            return Err(
                ConfigurationError::at(ConfigurationErrorKind::InvalidLanguageProfile, byte_offset),
            );
        },
        14 | 16 | 48 => if points.is_empty() {
            return Err(
                ConfigurationError::at(ConfigurationErrorKind::InvalidFieldValue, byte_offset),
            );
        },
        15 => if !points_equal_ascii(points, b"cli") {
            return Err(
                ConfigurationError::at(ConfigurationErrorKind::InvalidTargetAdapter, byte_offset),
            );
        },
        23 => {
            if points_are_duplicate(
                state.capabilities.as_slice(),
                points,
                work,
                max_work,
                byte_offset,
            )? {
                return Err(
                    ConfigurationError::at(
                        ConfigurationErrorKind::DuplicateSequenceValue,
                        byte_offset,
                    ),
                );
            }
            state.capabilities.push(vstd::slice::slice_to_vec(points));
        },
        37 => {
            if !points_equal_ascii(points, b"managed") && !points_equal_ascii(points, b"native") {
                return Err(
                    ConfigurationError::at(ConfigurationErrorKind::InvalidFieldValue, byte_offset),
                );
            }
            if points_are_duplicate(state.modes.as_slice(), points, work, max_work, byte_offset)? {
                return Err(
                    ConfigurationError::at(
                        ConfigurationErrorKind::DuplicateSequenceValue,
                        byte_offset,
                    ),
                );
            }
            if points_equal_ascii(points, b"native") {
                state.has_native_mode = true;
            }
            state.has_mode = true;
            state.modes.push(vstd::slice::slice_to_vec(points));
        },
        38 => {
            if !points_equal_ascii(points, b"afl++") && !points_equal_ascii(points, b"libfuzzer")
                && !points_equal_ascii(points, b"honggfuzz") {
                return Err(
                    ConfigurationError::at(ConfigurationErrorKind::InvalidFieldValue, byte_offset),
                );
            }
            if points_are_duplicate(
                state.backends.as_slice(),
                points,
                work,
                max_work,
                byte_offset,
            )? {
                return Err(
                    ConfigurationError::at(
                        ConfigurationErrorKind::DuplicateSequenceValue,
                        byte_offset,
                    ),
                );
            }
            state.has_backend = true;
            state.backends.push(vstd::slice::slice_to_vec(points));
        },
        45 if !duration_is_valid(points) => {
            return Err(
                ConfigurationError::at(ConfigurationErrorKind::InvalidFieldValue, byte_offset),
            );
        },
        _ => {},
    }
    Ok(())
}

fn observe_integer(
    field_id: u64,
    integer: &crucible_yaml::CoreInteger,
    byte_offset: u64,
) -> (result: Result<(), ConfigurationError>) {
    match field_id {
        1 => match integer_as_u64(integer) {
            Some(value) if value == CONFIGURATION_SCHEMA_VERSION as u64 => {},
            _ => return Err(
                ConfigurationError::at(
                    ConfigurationErrorKind::UnsupportedSchemaVersion,
                    byte_offset,
                ),
            ),
        },
        18 | 19 | 20 | 21 | 46 => match integer_as_u64(integer) {
            Some(value) if value > 0 => {},
            _ => return Err(
                ConfigurationError::at(ConfigurationErrorKind::IntegerOutOfRange, byte_offset),
            ),
        },
        25 => {
            if !signed_exit_code_is_valid(integer) {
                return Err(
                    ConfigurationError::at(ConfigurationErrorKind::IntegerOutOfRange, byte_offset),
                );
            }
        },
        47 if integer_as_u64(integer).is_none() => {
            return Err(
                ConfigurationError::at(ConfigurationErrorKind::IntegerOutOfRange, byte_offset),
            );
        },
        _ => {},
    }
    Ok(())
}

fn observe_boolean(
    state: &mut RenderState,
    field_id: u64,
    value: bool,
    byte_offset: u64,
) -> (result: Result<(), ConfigurationError>) {
    match field_id {
        36 => state.fuzz_enabled = Some(value),
        40 | 42 | 43 => if value {
            if state.exclusive_sanitizer_count > 0 {
                return Err(
                    ConfigurationError::at(
                        ConfigurationErrorKind::CrossFieldInvariant,
                        byte_offset,
                    ),
                );
            }
            state.exclusive_sanitizer_count += 1;
        },
        50..=52 if !value => {
            return Err(
                ConfigurationError::at(ConfigurationErrorKind::InvalidFieldValue, byte_offset),
            );
        },
        _ => {},
    }
    Ok(())
}

fn validate_cross_field_state(state: &RenderState) -> (result: Result<(), ConfigurationError>) {
    let enabled = match state.fuzz_enabled {
        Some(value) => value,
        None => return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::InternalInvariantViolation,
                state.fuzz_anchor,
            ),
        ),
    };
    if enabled && !state.has_mode {
        return Err(
            ConfigurationError::at(ConfigurationErrorKind::CrossFieldInvariant, state.modes_anchor),
        );
    }
    if !enabled && (state.has_mode || state.has_backend) {
        return Err(
            ConfigurationError::at(ConfigurationErrorKind::CrossFieldInvariant, state.fuzz_anchor),
        );
    }
    if state.has_native_mode && !state.has_backend {
        return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::CrossFieldInvariant,
                state.backends_anchor,
            ),
        );
    }
    if !state.has_native_mode && state.has_backend {
        return Err(
            ConfigurationError::at(
                ConfigurationErrorKind::CrossFieldInvariant,
                state.backends_anchor,
            ),
        );
    }
    Ok(())
}

fn push_output_byte(
    output: &mut Vec<u8>,
    byte: u8,
    byte_offset: u64,
    max_canonical_bytes: u64,
) -> (result: Result<(), ConfigurationError>) {
    if output.len() as u64 >= max_canonical_bytes {
        return Err(ConfigurationError::canonical(byte_offset, output.len() as u64));
    }
    output.push(byte);
    Ok(())
}

fn append_literal(
    output: &mut Vec<u8>,
    literal: &[u8],
    byte_offset: u64,
    max_canonical_bytes: u64,
) -> (result: Result<(), ConfigurationError>) {
    let mut index = 0;
    while index < literal.len()
        invariant
            index <= literal.len(),
        decreases literal.len() - index,
    {
        match push_output_byte(output, literal[index], byte_offset, max_canonical_bytes) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
        index += 1;
    }
    Ok(())
}

fn hex_digit(value: u32) -> (digit: u8)
    requires
        value < 16,
{
    if value < 10 {
        b'0' + value as u8
    } else {
        assert(value - 10 < 6);
        b'a' + (value - 10) as u8
    }
}

fn append_hex(
    output: &mut Vec<u8>,
    value: u32,
    digits: u32,
    byte_offset: u64,
    max_canonical_bytes: u64,
) -> (result: Result<(), ConfigurationError>) {
    if digits > 8 {
        return Err(
            ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, byte_offset),
        );
    }
    let mut remaining = digits;
    while remaining > 0
        invariant
            remaining <= digits,
            digits <= 8,
        decreases remaining,
    {
        remaining -= 1;
        assert(remaining <= 7);
        let shift = remaining * 4;
        assert(shift < 32);
        let nibble = (value >> shift) % 16;
        assert(nibble < 16);
        match push_output_byte(output, hex_digit(nibble), byte_offset, max_canonical_bytes) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn append_quoted_points(
    output: &mut Vec<u8>,
    points: &[u32],
    byte_offset: u64,
    max_canonical_bytes: u64,
) -> (result: Result<(), ConfigurationError>) {
    match push_output_byte(output, 0x22, byte_offset, max_canonical_bytes) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    let mut index = 0;
    while index < points.len()
        invariant
            index <= points.len(),
        decreases points.len() - index,
    {
        let point = points[index];
        let escaped = match point {
            0x08 => Some(0x62u8),
            0x09 => Some(0x74u8),
            0x0a => Some(0x6eu8),
            0x0c => Some(0x66u8),
            0x0d => Some(0x72u8),
            0x22 => Some(0x22u8),
            0x5c => Some(0x5cu8),
            _ => None,
        };
        if let Some(escaped_byte) = escaped {
            match push_output_byte(output, 0x5c, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            match push_output_byte(output, escaped_byte, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
        } else if (0x20..0x7f).contains(&point) {
            match push_output_byte(output, point as u8, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
        } else if point <= 0xffff {
            let prefix: &[u8] = &[0x5c, 0x75];
            match append_literal(output, prefix, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            match append_hex(output, point, 4, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
        } else {
            if point > 0x10ffff {
                return Err(
                    ConfigurationError::at(
                        ConfigurationErrorKind::InternalInvariantViolation,
                        byte_offset,
                    ),
                );
            }
            assert(point <= 0x10ffff);
            let first_value = point / 0x40000;
            let second_value = (point / 0x1000) % 0x40;
            let third_value = (point / 0x40) % 0x40;
            let fourth_value = point % 0x40;
            assert(first_value <= 4);
            assert(second_value < 0x40);
            assert(third_value < 0x40);
            assert(fourth_value < 0x40);
            assert(0xf0 + first_value <= 0xff);
            assert(0x80 + second_value <= 0xff);
            assert(0x80 + third_value <= 0xff);
            assert(0x80 + fourth_value <= 0xff);
            let first = (0xf0u32 + first_value) as u8;
            let second = (0x80u32 + second_value) as u8;
            let third = (0x80u32 + third_value) as u8;
            let fourth = (0x80u32 + fourth_value) as u8;
            match push_output_byte(output, first, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            match push_output_byte(output, second, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            match push_output_byte(output, third, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
            match push_output_byte(output, fourth, byte_offset, max_canonical_bytes) {
                Ok(()) => {},
                Err(error) => return Err(error),
            }
        }
        index += 1;
    }
    push_output_byte(output, 0x22, byte_offset, max_canonical_bytes)
}

fn append_decimal_u32(
    output: &mut Vec<u8>,
    value: u32,
    minimum_digits: u32,
    byte_offset: u64,
    max_canonical_bytes: u64,
) -> (result: Result<(), ConfigurationError>) {
    if minimum_digits > 10 {
        return Err(
            ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, byte_offset),
        );
    }
    let mut remaining = value;
    let mut reversed = Vec::new();
    while remaining >= 10
        invariant
            remaining <= value,
        decreases remaining,
    {
        let digit = remaining % 10;
        assert(digit < 10);
        reversed.push(b'0' + digit as u8);
        remaining /= 10;
    }
    assert(remaining < 10);
    reversed.push(b'0' + remaining as u8);
    let minimum = minimum_digits as usize;
    let mut padding = if reversed.len() < minimum {
        minimum - reversed.len()
    } else {
        0
    };
    while padding > 0
        invariant
            minimum <= 10,
        decreases padding,
    {
        match push_output_byte(output, b'0', byte_offset, max_canonical_bytes) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
        padding -= 1;
    }
    let mut index = reversed.len();
    while index > 0
        invariant
            index <= reversed.len(),
        decreases index,
    {
        index -= 1;
        match push_output_byte(output, reversed[index], byte_offset, max_canonical_bytes) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn append_integer(
    output: &mut Vec<u8>,
    integer: &crucible_yaml::CoreInteger,
    byte_offset: u64,
    max_canonical_bytes: u64,
) -> (result: Result<(), ConfigurationError>) {
    let limbs = integer.limbs();
    if limbs.is_empty() {
        return Err(
            ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, byte_offset),
        );
    }
    if integer.negative() {
        match push_output_byte(output, b'-', byte_offset, max_canonical_bytes) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
    }
    let mut index = limbs.len();
    index -= 1;
    match append_decimal_u32(output, limbs[index], 1, byte_offset, max_canonical_bytes) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }
    while index > 0
        invariant
            index < limbs.len(),
        decreases index,
    {
        index -= 1;
        match append_decimal_u32(output, limbs[index], 9, byte_offset, max_canonical_bytes) {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn push_render_task(tasks: &mut Vec<RenderTask>, task: RenderTask, anchor: u64) -> (result: Result<
    (),
    ConfigurationError,
>) {
    if tasks.len() as u64 >= MAX_CONFIGURATION_RENDER_TASKS {
        return Err(ConfigurationError::at(ConfigurationErrorKind::WorkLimitExceeded, anchor));
    }
    tasks.push(task);
    Ok(())
}

fn map_mapping_error(error: crucible_yaml::TypedMappingFieldError) -> (mapped: ConfigurationError) {
    let kind = match error.kind() {
        TypedMappingFieldErrorKind::UnknownField => ConfigurationErrorKind::UnknownField,
        TypedMappingFieldErrorKind::MissingRequiredField => {
            ConfigurationErrorKind::MissingRequiredField
        },
        TypedMappingFieldErrorKind::MappingKindMismatch
        | TypedMappingFieldErrorKind::MappingKeyNotString
        | TypedMappingFieldErrorKind::ValueKindMismatch => ConfigurationErrorKind::WrongValueKind,
        TypedMappingFieldErrorKind::FieldLimitExceeded
        | TypedMappingFieldErrorKind::KeyCodePointLimitExceeded => {
            ConfigurationErrorKind::TypedNodeLimitExceeded
        },
        _ => ConfigurationErrorKind::InternalInvariantViolation,
    };
    ConfigurationError::at(kind, error.byte_offset())
}

fn map_binding_error(error: crucible_yaml::TypedValueBindingError) -> (mapped: ConfigurationError) {
    let kind = match error.kind() {
        TypedValueBindingErrorKind::YamlValueKindMismatch => ConfigurationErrorKind::WrongValueKind,
        _ => ConfigurationErrorKind::InternalInvariantViolation,
    };
    ConfigurationError::at(kind, error.byte_offset())
}

} // verus!
verus! {

fn ascii_name(field_id: u64) -> (name: Vec<u32>) {
    let bytes = match field_id {
        1 => field_name_01(),
        2 => field_name_02(),
        3 => field_name_03(),
        4 => field_name_04(),
        5 => field_name_05(),
        6 => field_name_06(),
        7 => field_name_07(),
        8 => field_name_08(),
        9 => field_name_09(),
        10 => field_name_10(),
        11 => field_name_11(),
        12 => field_name_12(),
        13 => field_name_13(),
        14 => field_name_14(),
        15 => field_name_15(),
        16 => field_name_16(),
        17 => field_name_17(),
        18 => field_name_18(),
        19 => field_name_19(),
        20 => field_name_20(),
        21 => field_name_21(),
        22 => field_name_22(),
        23 => field_name_23(),
        24 => field_name_24(),
        25 => field_name_25(),
        26 => field_name_26(),
        27 => field_name_27(),
        28 => field_name_28(),
        29 => field_name_29(),
        30 => field_name_30(),
        31 => field_name_31(),
        32 => field_name_32(),
        33 => field_name_33(),
        34 => field_name_34(),
        35 => field_name_35(),
        36 => field_name_36(),
        37 => field_name_37(),
        38 => field_name_38(),
        39 => field_name_39(),
        40 => field_name_40(),
        41 => field_name_41(),
        42 => field_name_42(),
        43 => field_name_43(),
        44 => field_name_44(),
        45 => field_name_45(),
        46 => field_name_46(),
        47 => field_name_47(),
        48 => field_name_48(),
        49 => field_name_49(),
        50 => field_name_50(),
        51 => field_name_51(),
        52 => field_name_52(),
        _ => field_name_invalid(),
    };
    let mut name = Vec::new();
    let mut index = 0;
    while index < bytes.len()
        invariant
            index <= bytes.len(),
        decreases bytes.len() - index,
    {
        name.push(bytes[index] as u32);
        index += 1;
    }
    name
}

fn required_field(
    parent_schema_node_index: u64,
    field_id: u64,
    value_schema_node_index: u64,
) -> (field: TypedFieldDefinition) {
    TypedFieldDefinition::new(
        parent_schema_node_index,
        field_id,
        ascii_name(field_id),
        value_schema_node_index,
        true,
    )
}

fn configuration_schema() -> (result: Result<
    crucible_yaml::CompiledTypedFieldSchema,
    ConfigurationError,
>) {
    let nodes =
        vec![
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 0, 12, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Integer, 0, 0, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 12, 13, None),
        TypedSchemaNode::new(TypedSchemaValueKind::String, 0, 0, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 13, 14, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 14, 17, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Sequence, 0, 0, Some(3)),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 17, 23, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Boolean, 0, 0, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 23, 24, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 24, 26, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Sequence, 0, 0, Some(1)),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 26, 27, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 27, 35, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 35, 38, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 38, 39, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 39, 44, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 44, 47, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 47, 48, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 48, 49, None),
        TypedSchemaNode::new(TypedSchemaValueKind::Mapping, 49, 52, None),
    ];
    let fields =
        vec![
        required_field(0, 1, 1),
        required_field(0, 2, 2),
        required_field(0, 3, 4),
        required_field(0, 4, 5),
        required_field(0, 5, 7),
        required_field(0, 6, 9),
        required_field(0, 7, 12),
        required_field(0, 8, 13),
        required_field(0, 9, 16),
        required_field(0, 10, 17),
        required_field(0, 11, 18),
        required_field(0, 12, 19),
        required_field(2, 13, 3),
        required_field(4, 14, 3),
        required_field(5, 15, 3),
        required_field(5, 16, 3),
        required_field(5, 17, 6),
        required_field(7, 18, 1),
        required_field(7, 19, 1),
        required_field(7, 20, 1),
        required_field(7, 21, 1),
        required_field(7, 22, 8),
        required_field(7, 23, 6),
        required_field(9, 24, 10),
        required_field(10, 25, 11),
        required_field(10, 26, 8),
        required_field(12, 27, 6),
        required_field(13, 28, 14),
        required_field(13, 29, 15),
        required_field(13, 30, 15),
        required_field(13, 31, 15),
        required_field(13, 32, 15),
        required_field(13, 33, 15),
        required_field(13, 34, 15),
        required_field(13, 35, 15),
        required_field(14, 36, 8),
        required_field(14, 37, 6),
        required_field(14, 38, 6),
        required_field(15, 39, 8),
        required_field(16, 40, 8),
        required_field(16, 41, 8),
        required_field(16, 42, 8),
        required_field(16, 43, 8),
        required_field(16, 44, 8),
        required_field(17, 45, 3),
        required_field(17, 46, 1),
        required_field(17, 47, 1),
        required_field(18, 48, 3),
        required_field(19, 49, 20),
        required_field(20, 50, 8),
        required_field(20, 51, 8),
        required_field(20, 52, 8),
    ];
    match compile_typed_field_schema(
        TypedFieldSchema::new(CONFIGURATION_SCHEMA_VERSION, 0, nodes, fields),
        canonical_typed_field_schema_limits(),
    ) {
        Ok(schema) => Ok(schema),
        Err(_) => Err(
            ConfigurationError::at(ConfigurationErrorKind::InternalInvariantViolation, 0),
        ),
    }
}

fn parse_canonical_graph(input: &[u8], max_source_bytes: u64, max_depth: u64) -> (result: Result<
    CanonicalYamlGraphSource,
    ConfigurationError,
>) {
    let decoded = match decode_profile1(
        input,
        DecodeLimits::new(max_source_bytes, MAX_PROFILE1_DECODED_SCALARS),
        BomPolicy::AllowAndStrip,
    ) {
        Ok(value) => value,
        Err(error) => {
            let kind = if error.kind() == crucible_yaml::DecodeErrorKind::SourceByteLimitExceeded {
                ConfigurationErrorKind::SourceByteLimitExceeded
            } else {
                ConfigurationErrorKind::YamlSyntax
            };
            return Err(ConfigurationError::at(kind, error.byte_offset()));
        },
    };
    let atoms = match atomize_profile1(&decoded, AtomizeLimits::new(MAX_PROFILE1_LEXICAL_ATOMS)) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let layout = match analyze_profile1_layout(&atoms, canonical_structural_layout_limits()) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let structural = match scan_profile1_structural_lexemes(
        &atoms,
        &layout,
        canonical_structural_scan_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let quoted = match scan_profile1_quoted_scalars(
        &atoms,
        &layout,
        &structural,
        canonical_quoted_scalar_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let plain = match scan_profile1_plain_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        canonical_plain_scalar_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let block = match scan_profile1_block_scalars(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        canonical_block_scalar_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let tokens = match scan_profile1_completed_tokens(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        canonical_completed_token_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let cst_limits = CstLimits::new(
        MAX_PROFILE1_CST_DOCUMENTS,
        MAX_PROFILE1_CST_NODES,
        MAX_PROFILE1_CST_SEQUENCE_ENTRIES,
        MAX_PROFILE1_CST_MAPPING_ENTRIES,
        MAX_PROFILE1_CST_DIRECTIVES,
        MAX_PROFILE1_CST_WARNINGS,
        max_depth,
    );
    let cst = match parse_profile1_cst(
        &atoms,
        &layout,
        &structural,
        &quoted,
        &plain,
        &block,
        &tokens,
        cst_limits,
    ) {
        Ok(value) => value,
        Err(error) => {
            let kind = if error.kind() == crucible_yaml::CstErrorKind::DepthLimitExceeded {
                ConfigurationErrorKind::DepthLimitExceeded
            } else {
                ConfigurationErrorKind::YamlSyntax
            };
            return Err(ConfigurationError::at(kind, error.byte_offset()));
        },
    };
    let structural_keys = match compose_profile1_canonical_structural_keys(
        &atoms,
        &quoted,
        &plain,
        &block,
        &tokens,
        &cst,
        canonical_semantic_topology_limits(),
        canonical_semantic_scalar_table_limits(),
        AnchorAliasLimits::new(MAX_PROFILE1_ANCHOR_DECLARATIONS, MAX_PROFILE1_ALIAS_BINDINGS),
        canonical_semantic_node_table_limits(),
        canonical_alias_cycle_limits(),
        canonical_scalar_key_limits(),
        canonical_structural_key_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let duplicate_free = match reject_profile1_duplicate_keys(
        structural_keys,
        canonical_duplicate_key_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    let expanded = match expand_profile1_merge_keys(
        duplicate_free,
        canonical_merge_expansion_limits(),
    ) {
        Ok(value) => value,
        Err(error) => return Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    };
    match lower_profile1_canonical_graph(expanded, canonical_lowering_limits()) {
        Ok(value) => Ok(value),
        Err(error) => Err(
            ConfigurationError::at(ConfigurationErrorKind::YamlSyntax, error.byte_offset()),
        ),
    }
}

} // verus!
