# Targets, Execution, and Isolation

## 10. Target Interface

All targets should normalize behind a common abstraction, but a configured target name must
not stand in for an immutable build identity.

```rust
pub struct TargetBuild {
    pub id: TargetBuildId,
    pub target: TargetId,
    pub source_revision: Option<String>,
    pub dirty_source_digest: Option<String>,
    pub primary_executable: Option<ArtifactRef>,
    pub runtime_artifacts: Vec<ArtifactRef>,
    pub identity_digest: String,
    pub build_manifest: ArtifactRef,
    pub toolchain: ToolchainIdentity,
    pub platform: PlatformIdentity,
    pub source_snapshot: SourceSnapshotId,
    pub build_recipe: BuildRecipeId,
    pub build_execution: BuildExecutionId,
    pub instrumentation: Vec<InstrumentationIdentity>,
    pub symbols: Vec<ArtifactRef>,
    pub proof_artifacts: Vec<ProofArtifactId>,
}
```

The build manifest should record compiler and linker flags, enabled sanitizers, relevant
dependencies, runtime configuration, and other inputs needed to distinguish behaviorally
different builds. `identity_digest` should cover a canonical build manifest and all declared
runtime artifacts. Targets such as configured services may have no embedded executable but
must still provide an immutable version/configuration identity suitable for comparison.

Source, build, deployment, and runtime identity are separate records:

```text
SourceSnapshot
    ↓ built by
BuildRecipe + BuildEnvironment
    ↓ produces
BuildExecution + Deployable Artifact Set + Proof Artifacts
    ↓ installed by
Deployment Recipe
    ↓ creates
Target Deployment in a Runtime Environment
```

The build recipe records commands as typed argv arrays, declared inputs and outputs, dependency
locks, environment, network policy, and expected toolchains. The build execution records all
effective values, logs, exit outcomes, generated artifacts, and undeclared-input detections.
Hermetic and reproducible builds are preferred, but a non-hermetic build is still representable
and must disclose its unresolved inputs rather than claiming reproducibility.

Runtime identity includes the Crucible build, adapter and plugin versions, operating system and
kernel, CPU architecture and relevant feature flags, container or VM image, filesystem and
mount configuration, loaded dynamic libraries, locale, timezone, and other behaviorally
relevant state available to the backend.

Preparation creates an exclusively owned instance. A single prepared mutable target must not
be reset concurrently with another run.

```rust
#[async_trait]
pub trait TargetAdapter: Send + Sync {
    fn id(&self) -> TargetId;

    async fn prepare(
        &self,
        build: &TargetBuild,
        context: &PrepareContext,
    ) -> Result<Box<dyn TargetInstance>, HarnessError>;
}

#[async_trait]
pub trait TargetInstance: Send {
    async fn execute(
        &mut self,
        request: &RunRequest,
        context: &ExecutionContext,
    ) -> Result<RawObservation, HarnessError>;

    async fn reset(
        &mut self,
    ) -> Result<(), HarnessError>;

    async fn cleanup(
        &mut self,
    ) -> Result<(), HarnessError>;
}
```

The coordinator owns the instance lifecycle, guarantees cleanup, and discards an instance
whose state cannot be reset with confidence. Stateless CLI runs may use a fresh instance per
run. Stateful and persistent-mode adapters may pool instances explicitly.

Supported initial target adapters:

### CLI Target

Executes a binary.

Input delivery modes:

- stdin,
- generated relative file,
- argument,
- environment variable.

### Library Target

Calls a library function through:

- Rust API,
- C ABI,
- generated helper executable.

### Local HTTP Target

Tests explicitly configured local or authorized services.

Input dimensions may include:

- request path,
- method,
- headers,
- body,
- action sequence.

### Stateful Target

Represents systems such as:

- databases,
- caches,
- transactional services,
- protocol state machines.

Input is an action sequence.

### Scenario / Service-Topology Target

Coordinates multiple named processes, services, clients, network links, storage devices, and
observers through the scenario graph. It is the foundation for distributed-system and complex
integration testing.

### Virtual-Machine / Kernel Target

Boots a versioned VM image, restores a clean snapshot, injects guest stimuli, captures serial
output and crash dumps, and tears down the complete VM. Kernel targets must not execute with
host-kernel privilege merely because the process sandbox supports privileged operations.

### Emulator / Embedded Target

Runs firmware under an emulator or simulator, or through an explicitly configured
hardware-in-the-loop controller. It supports reset causes, watchdogs, serial/debug transports,
flash and nonvolatile state, virtual peripherals, power interruption, and deterministic clock
or interrupt control where the backend exposes them.

### Compiler / Toolchain Target

Models a compile pipeline followed by optional execution, interpretation, disassembly, IR
inspection, or proof. It preserves source-language validity assumptions and distinguishes
compiler crashes, invalid diagnostics, miscompilations, nondeterministic artifacts, and
disagreements that do not yet establish blame.

---

## 11. Execution Isolation

Buggy software must be treated as untrusted with respect to harness stability.

Every execution backend must publish a capability manifest and should support:

- wall-clock timeout,
- CPU limit,
- memory limit,
- process-count limit,
- file-size limit,
- bounded stdout/stderr capture,
- isolated temporary working directory,
- controlled environment variables,
- controlled writable paths,
- network disabled by default,
- deterministic seed where possible.

Isolation guarantees are platform-specific. The core must not encode one operating system or
CPU architecture as the universal execution model. The initial implementation should provide
platform backends for the environments where Crucible is built and tested, including Linux,
macOS, and Windows where CI capacity is available.

```text
Linux:
  process groups, rlimits, namespaces, cgroups, seccomp, containers where available

macOS:
  process groups, resource limits, application sandbox profiles, VMs where required

Windows:
  job objects, restricted tokens, resource controls, isolated working directories
```

Architecture variants should include x86_64 and AArch64 where toolchains and CI support them.
The same evidence contract applies across platform backends so cross-platform behavior can
participate in differential experiments.

Isolation tiers are explicit:

```text
Tier 0: direct process with resource accounting
Tier 1: operating-system sandbox
Tier 2: container or equivalent isolated userspace
Tier 3: disposable VM or microVM
Tier 4: emulator, simulator, or controlled hardware-in-the-loop lab
```

Projects declare the minimum tier and individual required capabilities. Kernel, privileged
runtime, hostile build, and untrusted candidate-patch workloads default to Tier 3 or stronger.
Sanitizer and fuzzer runtimes are detection tools, not security boundaries.

For every run, record which controls were requested, which were successfully enforced, and
which were unavailable. A run must not claim that networking or filesystem access was
isolated when the host could not enforce that policy. Projects may require particular
capabilities and fail closed when they are absent.

### 11.1 Capability manifest

```rust
pub struct CapabilityManifest {
    pub harness_build: HarnessBuildId,
    pub backend: BackendIdentity,
    pub platform: PlatformIdentity,
    pub capabilities: BTreeMap<CapabilityId, CapabilityStatus>,
    pub evidence: Vec<EvidenceId>,
}

pub enum CapabilityStatus {
    Enforced { mechanism: MechanismIdentity },
    AvailableButDisabled { reason: String },
    Degraded { limitations: Vec<String> },
    Unavailable { reason: String },
    NotApplicable { reason: String },
}
```

Capability detection is executable evidence and should be independently self-tested. Campaign
planning resolves every required capability before expensive preparation. A later runtime
failure to enforce an advertised capability invalidates affected attempts and creates a harness
finding.

The threat model must distinguish trusted project configuration from less-trusted target
binaries, build scripts, candidate patches, and fully untrusted target input and output.
Building or verifying a candidate patch executes project-controlled code and therefore
requires a separately declared build policy.

Secrets needed by an authorized target must be represented by opaque secret references. The
artifact store, logs, reports, agent packets, reproduction bundles, and external integrations
must apply field-aware redaction and must record that redaction occurred. Secret values must
not become corpus mutation material unless the project explicitly declares a synthetic test
secret safe for retention.

Target-controlled data must never be directly interpolated into shell commands.

Prefer direct process APIs over shell invocation.

When stdout or stderr reaches its capture limit, the runner must follow an explicit policy:
continue draining while discarding excess bytes, or terminate the target. Merely stopping the
read can deadlock a child on a full pipe. Record truncation, retained byte count, and discarded
byte count in the raw observation. Timeout and cancellation must act on the configured
process tree, not only the immediate child.

---
