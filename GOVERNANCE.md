# Governance

## Project stewardship

Crucible is a Corvidae Coding Projects project. The organization owns the repository and appoints
maintainers responsible for technical direction, releases, security response, moderation, and
repository administration.

The project uses maintainer-led, evidence-based governance. Community input is invited and
substantive technical disagreement is recorded, but consensus is not required when a timely,
accountable decision is necessary.

## Roles

### Contributors

Contributors submit issues, discussions, reviews, documentation, specifications, proofs, code,
tests, fixtures, or other project work. A merged contribution does not automatically grant a
maintainer role.

### Reviewers

Reviewers have demonstrated sustained knowledge in one or more project areas. They may provide
formal review and triage but do not merge changes unless they also have maintainer permissions.

### Maintainers

Maintainers may triage, review, merge, release, moderate, and make scoped project decisions.
Maintainers are expected to disclose conflicts, preserve evidence, apply the security policy,
and avoid approving their own trusted-boundary exceptions without independent review.

### Organization owners

Organization owners appoint and remove maintainers, resolve governance deadlocks, control
credentials and legal matters, and act as the final escalation point for security and conduct
matters.

## Decision process

Routine fixes and documentation changes are decided through pull-request review. Material
architecture, evidence-schema, compatibility, security, trusted-boundary, or scope decisions use
this process:

1. open a design proposal describing evidence, alternatives, risks, and verification impact;
2. allow a reasonable review period proportionate to the decision's consequences;
3. obtain approval from at least one maintainer who did not author the proposal;
4. record significant accepted decisions as an architecture decision record;
5. update the specification, roadmap, tests, and migration policy in the same change or linked
   work.

Maintainers may merge urgent security or repository-integrity changes before the ordinary review
period. They must document the decision once disclosure constraints permit.

## Scope and verification integrity

Release staging may reorder work but must not silently redefine the committed product as the
MVP. A proposal that removes a declared capability, weakens an oracle, expands authorization,
reduces isolation, disables evidence retention, or broadens the trusted computing base must say
so explicitly and receive organization-owner approval.

Every Verus assumption, external body, foreign boundary, and proof exception is reviewable
project state. Coding agents cannot approve trusted-boundary changes. A human maintainer remains
accountable even when an agent generated the implementation, proof, review, or recommendation.

## Merging and releases

- Changes to `main` normally arrive through reviewed pull requests.
- Required checks and review conversations must pass or be explicitly resolved.
- Force pushes and branch deletion are prohibited on protected branches.
- Releases are created from identified commits with release notes, artifact and toolchain
  provenance, known limitations, and applicable security information.
- Maintainers may delay a release when evidence, proof reproduction, or supported-platform
  results are incomplete.

## Conflicts of interest

Anyone reviewing a contribution should disclose financial, employment, personal, or authorship
interests that could reasonably affect impartiality. Authors should not be the sole approvers of
their own material changes. Security and conduct reports should be handled by an uninvolved
maintainer whenever possible.

## Becoming or ceasing to be a maintainer

Organization owners may appoint maintainers based on sustained constructive participation,
technical judgment, reliability, security awareness, and alignment with project norms.

Maintainers may step down at any time. Extended inactivity may lead to emeritus status and
removal of elevated access after an attempt to make contact. Access is removed promptly when
continued access would create a security, legal, or community risk.

## Amendments

Governance amendments use the material decision process above. The pull request must explain why
the change improves accountability, participation, security, or project continuity.
