# Security policy

## Supported line

Digital Ham Radio Logbook is a pre-1.0 project under active development. Security maintenance is focused on the current `main` line and the current public release candidate.

Historical development checkpoints are retained for engineering context only and are not supported release lines.

## Reporting a vulnerability

Please do not disclose suspected security vulnerabilities in a public issue, pull request, discussion, or social post before the problem has been assessed.

Prefer GitHub's private **Report a vulnerability** / Security Advisory flow for this repository when it is available. If that entry is not available in the GitHub interface, contact the maintainer privately through the repository owner's GitHub profile and include enough information to reproduce and assess the issue safely.

Useful reports include:

- affected version, tag, or commit;
- operating system and architecture;
- concise reproduction steps;
- expected and observed behavior;
- impact assessment;
- relevant logs or proof of concept that do not expose unrelated private data.

Do not include real logbook databases, callsign histories, personal configuration files, credentials, tokens, or other user data unless specifically requested through a private channel and sanitized first.

## Dependency security

The repository runs a scheduled RustSec audit against `Cargo.lock` and also audits dependency-sensitive changes. Confirmed vulnerabilities fail the security gate.

RustSec informational advisories, such as notices that a transitive crate is unmaintained, are reviewed separately from vulnerabilities. They are not silently treated as CVEs, and dependency replacements are evaluated through normal pull requests and regression testing rather than forced into the release line without validation.

## Release integrity

Published artifacts are expected to correspond to an immutable release tag and include SHA-256 verification data. Release candidates remain explicitly marked as pre-releases until the project completes its release gates and manual validation.
