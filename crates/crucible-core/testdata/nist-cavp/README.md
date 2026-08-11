# NIST CAVP SHA-256 byte vectors

`SHA256ShortMsg.rsp` is the complete SHA-256 short-message member from NIST's byte-oriented
SHA test-vector archive. It is checked in with line endings normalized from CRLF to LF and its
final blank line removed so it can be reviewed and exercised on every platform.

- Source: <https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabytetestvectors.zip>
- Retrieved: 2026-08-11
- Source archive SHA-256: `929ef80b7b3418aca026643f6f248815913b60e01741a44bba9e118067f4c9b8`
- Archive member: `shabytetestvectors/SHA256ShortMsg.rsp`
- Original member SHA-256: `75e1cb83994638481808e225b9eb0c1ebd0c232d952ac42b61abce6363be283c`
- Checked-in file SHA-256: `294ecec26959357405a621121bbfb01db4d45b9e834624b2d71aedd94ffde019`

The accompanying 65-byte `0x00..0x40` vector fills the first post-block boundary not present in
NIST's short-message member. Its expected digest was independently cross-checked with Python's
OpenSSL-backed `hashlib` and the OpenSSL CLI on 2026-08-11.
