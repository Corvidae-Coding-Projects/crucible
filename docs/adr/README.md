# Architecture Decision Records

Architecture decision records preserve the context, alternatives, consequences, and approval of
material project decisions.

Use an ADR when a change affects public evidence meaning, durable schemas, trusted-boundary
policy, isolation, target or engine interfaces, compatibility, security posture, or committed
scope.

## Process

1. Copy `0000-template.md` to the next four-digit sequence number and a short descriptive name.
2. Open or link the required design-proposal issue.
3. Fill in evidence, alternatives, consequences, verification, migration, and scope impact.
4. Submit the ADR and the corresponding specification change in one pull request when possible.
5. Record the final status and decision date after maintainer approval.

Accepted ADRs are immutable historical records. A later decision supersedes an earlier ADR with
a new record and cross-links both documents; it does not rewrite the original rationale.

## Status values

- Proposed
- Accepted
- Rejected
- Deprecated
- Superseded by ADR-NNNN
