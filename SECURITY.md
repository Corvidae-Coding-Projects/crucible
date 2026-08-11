# Security Policy

## Project status and supported versions

Crucible is currently a design specification and pre-release project. There is no supported
production executable yet.

| Version | Supported |
| --- | --- |
| `main` | Yes, for repository and specification security issues |
| Tagged pre-releases | Only when the release notes explicitly say so |
| Unreleased forks or downstream modifications | No |

This table will be replaced with a versioned support window before the first production release.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability, leaked secret, unsafe default, sandbox
escape, proof unsoundness, or other security-sensitive matter.

Use GitHub's
[private vulnerability reporting](https://github.com/Corvidae-Coding-Projects/crucible/security/advisories/new)
to contact the maintainers confidentially. If GitHub prevents access to that form, contact an
organization owner through the organization's established private channel and reference this
policy without including sensitive details publicly.

Include as much of the following as is safe:

- affected revision, release, component, and platform;
- impact and violated security or correctness property;
- minimal reproduction steps or evidence bundle;
- relevant configuration, build, proof, and trusted-boundary identities;
- whether the issue is already public or known to another party;
- suggested mitigations and disclosure constraints;
- a secure way to follow up.

Do not submit real credentials, private target data, weaponized payloads, or exploit chains. Use
synthetic evidence and the smallest non-destructive demonstration that establishes the defect.

## Response process

Maintainers aim to:

1. acknowledge a report within three business days;
2. provide an initial triage assessment within seven business days;
3. agree on disclosure timing and interim mitigations with the reporter;
4. preserve evidence while limiting unnecessary access;
5. prepare a fix, regression artifact, advisory, and release when applicable;
6. credit the reporter if requested and legally permissible.

These are response targets rather than warranties. Complex proof, platform, or coordinated
disclosure work may require more time, in which case maintainers will provide status updates.

## Scope

Relevant reports include vulnerabilities in:

- Crucible's executor, isolation, cleanup, and resource controls;
- configuration, artifact, evidence, plugin, agent, report, or bundle parsing;
- path handling, shell boundaries, secret redaction, and report rendering;
- remote worker, build, proof, and content-addressed storage protocols;
- Verus specifications, assumptions, proof-to-binary linkage, and trusted-boundary accounting;
- CI/CD, release, dependency, and repository configuration;
- defaults that could cause unauthorized or unexpectedly remote testing.

Security defects in unrelated third-party targets should be reported to their owners, not to
Crucible's public issue tracker.

## Research expectations

Only test systems you own or are explicitly authorized to test. Avoid privacy violations,
service disruption, data destruction, persistence, credential access, or lateral movement.
Give maintainers a reasonable opportunity to investigate before disclosure.

Crucible's non-goals prohibit exploit generation and offensive automation. A security-relevant
correctness finding remains valuable, but this repository will focus on evidence, reproduction,
invariant restoration, and verification rather than weaponization.
