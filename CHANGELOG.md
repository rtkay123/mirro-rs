## [v0.2.2] - 2023-11-19
### New Features
- [`37245a4`](https://github.com/rtkay123/mirro-rs/commit/37245a4139436c694967ca9aa3a1941f025958af) - get mirrors with client
- [`25250b0`](https://github.com/rtkay123/mirro-rs/commit/25250b0bac3746e984cea3c1047e6a24e5c84506) - replace logger with tracing
- [`6506dd1`](https://github.com/rtkay123/mirro-rs/commit/6506dd1ea2f45c66d0029e000d619ea72a522468) - handle cases where journald is absent

### Bug Fixes
- [`d829437`](https://github.com/rtkay123/mirro-rs/commit/d829437e966990a10dd86869fd1753f63f232b61) - remove flaky parsing and get lastsync timestamp directly from url *(commit by [@phanen](https://github.com/phanen))*
- [`6a4c8ac`](https://github.com/rtkay123/mirro-rs/commit/6a4c8acd1e2f9a559a28c0325beb66747975cfc8) - skip journal errors outside of unix

### Refactors
- [`983bc38`](https://github.com/rtkay123/mirro-rs/commit/983bc38208406eeaf672f1887d1f55143ac684b2) - use non blocking fs module

### Chores
- [`8f610b2`](https://github.com/rtkay123/mirro-rs/commit/8f610b2247bbe76191c8199071767de6e2fdb357) - bump lib ver
- [`0306ff3`](https://github.com/rtkay123/mirro-rs/commit/0306ff300b4564d0626103a8aa3454da31021d98) - update changelog
- [`b31531c`](https://github.com/rtkay123/mirro-rs/commit/b31531c2510146ba2f7a92f09ac4edf3c40813e6) - bump pkg ver
- [`9b3b56a`](https://github.com/rtkay123/mirro-rs/commit/9b3b56a9d1ff83da121bec55d4930fcf95a22c15) - clippy fix
- [`bfd2b08`](https://github.com/rtkay123/mirro-rs/commit/bfd2b0825cc0500ccdd67c01bbbb1f5cb629be35) - remove log comments


## [v0.2.1] - 2023-11-14

### What's Changed
* tests: add more tests to test suite by @rtkay123 in https://github.com/rtkay123/mirro-rs/pull/22
* Implement From to convert args to config struct by @rtkay123 in https://github.com/rtkay123/mirro-rs/pull/23
* docs: update README.md about official Arch Linux package by @orhun in https://github.com/rtkay123/mirro-rs/pull/24

### New Contributors
* @dependabot made their first contribution in https://github.com/rtkay123/mirro-rs/pull/15
* @orhun made their first contribution in https://github.com/rtkay123/mirro-rs/pull/24

**Full Changelog**: https://github.com/rtkay123/mirro-rs/compare/v0.2.0...v0.2.1

## [v0.2.0] - 2023-11-11

### What's Changed
* fix: make `ftp` known as protocol type by @rtkay123 in https://github.com/rtkay123/mirro-rs/pull/8
* chore: replace tui with ratatui by @rtkay123 in https://github.com/rtkay123/mirro-rs/pull/9
* refactor: replace hyper with reqwest by @rtkay123 in https://github.com/rtkay123/mirro-rs/pull/10

### New Contributors
* @rtkay123 made their first contribution in https://github.com/rtkay123/mirro-rs/pull/8

[v0.2.2]: https://github.com/rtkay123/mirro-rs/compare/v0.2.1...v0.2.2