# Digital Ham Radio Logbook v0.11.0-RC2

## Status

`v0.11.0-RC2` is the second public release candidate for the v0.11 line. It promotes the post-RC1 fixes and repository hardening already reviewed on `main`; it does not introduce new product features.

Baseline before release preparation: `c3ffd3dd49d2dc18ef3b7cf227e77217b47cc7c4`.

## Changes since RC1

- fixed clipping and compression in the Logbook workspace at the documented `1050×680` reference size;
- corrected the advanced-filter layout so widget metrics no longer collide with the table workspace;
- consolidated repository governance around `main` as the single permanent branch, with short-lived pull-request branches;
- hardened release/security documentation and CI governance without changing application behavior.

## Manual acceptance

The maintainer approved the real local application after the post-RC1 `1050×680` layout correction. This acceptance is the human visual gate for promoting the corrected state to RC2; packaged RC2 artifacts must still be generated from the exact approved commit and validated before publication.

## Compatibility

RC2 deliberately contains no new feature, SQLite migration, schema change, ADIF contract change, runtime dependency change, index, cache, thread or asynchronous architecture change.

- SQLite schema remains version 7;
- existing configuration compatibility remains unchanged;
- existing backup/recovery behavior remains unchanged;
- ADIF import/export contracts remain unchanged;
- RC1 remains an immutable historical prerelease and its tag must not be moved.

## Security note

The current RustSec review reports no known vulnerability blocking this candidate. Repository issue #10 tracks informational `unmaintained` advisories in transitive dependencies separately; those advisories are not being hidden or force-fixed as part of this release promotion.

## Publication gate

Before `v0.11.0-RC2` is published:

1. synchronized root and fuzz lockfiles must pass the locked CI suite;
2. Quality, Tests and build, Linux packaging smoke, Historical schemas 0-7 and Documentation integrity must pass on the release PR and resulting `main` commit;
3. the exact release-candidate workflow artifact must be generated once from the approved commit;
4. artifact checksums, packaged binary equality and package behavior must be validated;
5. only then may the annotated `v0.11.0-RC2` tag and GitHub prerelease be created.

RC2 is an evaluation candidate, not a declaration that v0.11.0 stable or 1.0.0 is ready.
