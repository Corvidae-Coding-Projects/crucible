# Contributing to Crucible

Thank you for helping build Crucible. Contributions may include specification corrections,
architecture proposals, Verus specifications and proofs, implementation, tests, known-defect
fixtures, adapters, documentation, and adversarial review.

## Before contributing

1. Read the [README](README.md), [design specification index](crucible.md),
   [roadmap](ROADMAP.md), and [governance model](GOVERNANCE.md).
2. Search existing issues and discussions before opening a new thread.
3. Use the appropriate issue form for a reproducible defect, capability proposal, or design
   change.
4. Report security-sensitive matters privately as described in [SECURITY.md](SECURITY.md).

The repository is in active Phase 0 implementation. Do not present an unimplemented capability
as available, and do not reduce the declared end-state scope merely to simplify an early phase.

## Contribution principles

Contributions must preserve these constraints:

- Begin eligible executable work in Verus Rust, not unrestricted Rust with verification deferred.
- Specify and test every unavoidable external or unverified boundary.
- Register every assumption, axiom, external body, foreign call, and trusted specification.
- Preserve original evidence when producing normalized, deduplicated, minimized, or repaired
  derivatives.
- Distinguish target defects, disagreements, test gaps, proof failures, harness failures, and
  infrastructure failures.
- Never add exploit generation, intrusion, persistence, weaponization, or other explicit
  non-goals from the specification.
- Add tests and known-defect fixtures for every behavior a subsystem claims to support.
- Record platform, capability, determinism, and isolation limitations rather than silently
  degrading them.

## AI-assisted contributions

AI assistance is allowed and expected. It does not transfer responsibility away from the human
contributor.

A pull request must disclose material AI use and identify:

- the tools or models used, when known;
- which parts were generated or substantially transformed;
- how claims, APIs, dependencies, and security-sensitive behavior were independently checked;
- which tests, proofs, or adversarial reviews were added because of that use;
- any generated assumptions or specifications requiring special scrutiny.

Agents may not approve their own trusted-boundary exceptions. Generated proofs are reviewed for
the actual specification and assumptions proved, not merely for a green solver result.

## Proposing architectural changes

Open a design proposal before implementing a change that affects public evidence schemas,
trusted-boundary policy, target or engine interfaces, storage meaning, security posture, or
committed scope. Significant accepted decisions receive an architecture decision record under
`docs/adr/`.

A proposal should include:

- the problem and motivating evidence;
- affected capabilities and invariants;
- alternatives considered;
- compatibility and migration consequences;
- verification and testing strategy;
- security, privacy, performance, and trusted-computing-base impact;
- explicit confirmation that the change does not silently remove a committed capability.

## Development workflow

1. Fork the repository or create a branch named for the change.
2. Keep commits focused and use clear imperative commit messages.
3. Sign commits with a GitHub-verifiable GPG, SSH, or S/MIME signature.
4. Update documentation, schemas, fixtures, and provenance contracts with code changes.
5. Run the checks applicable to the current repository phase.
6. Open a pull request using the repository template.
7. Respond to review and keep the branch current without rewriting other contributors' work.

For documentation changes, run:

```bash
bash scripts/check-docs.sh
npx --yes markdownlint-cli2@0.23.2 '**/*.md' '#node_modules'
```

For every code change, run the mandatory project interfaces:

```bash
cargo xtask format --check
cargo xtask verify --all
cargo xtask tcb-audit --deny-unregistered --deny-unapproved-growth
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked -- --test-threads=1
actionlint .github/workflows/*.yml
```

The xtask commands consume `tools/verus-toolchain.lock`, resolve absolute tools, validate exact
versions and binary digests, use `--locked` proof builds, and invalidate their own stale reports
before publishing new machine-readable evidence.

## Pull-request expectations

Every pull request should explain:

- what changed and why;
- user, contributor, evidence-schema, and compatibility impact;
- verification, tests, and fixtures run;
- new dependencies and why they are necessary;
- trusted-boundary additions or reductions;
- material AI assistance;
- remaining limitations or follow-up work.

Reviewers may ask for a smaller commit sequence, but they should not demand a capability or
verification downgrade as a substitute for sound implementation.

## Licensing

By submitting a contribution, you agree that it may be distributed under the repository's
[MIT License](LICENSE). You represent that you have the right to submit the contribution and
that it does not knowingly include incompatible material.
