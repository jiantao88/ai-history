# Release Checklist

Use this checklist before each public release. The goal is to make installation feel trustworthy for developers who discover the project from GitHub, Reddit, X, or Hacker News.

## Release readiness

- [ ] `cargo test` passes locally.
- [ ] `cargo build --release` passes locally.
- [ ] `README.md` and `README_CN.md` document the current behavior.
- [ ] `setup` installs the latest release artifact on macOS ARM64, macOS Intel, and Linux x86_64.
- [ ] The release notes explain the user-facing change, not just commit names.
- [ ] The privacy model is still accurate: local-first, no telemetry, optional LLM calls only when explicitly enabled.

## GitHub release

The repository already has `.github/workflows/release.yml`. A version tag matching `vX.Y.Z` triggers release builds for:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`

Before tagging:

```bash
git status --short
cargo test
cargo build --release
```

Tag and push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

After the workflow finishes:

- [ ] Release contains all three `.tar.gz` archives.
- [ ] Release contains matching `.sha256` files.
- [ ] Download links work from an incognito browser.
- [ ] `setup` can detect the latest release.
- [ ] Install command works on a clean machine or temporary test user.

## Trust improvements

These are the next distribution upgrades to prioritize:

1. **Homebrew tap**: gives macOS developers a familiar install path.
2. **Cargo package**: enables `cargo install ai-history` for Rust users.
3. **Manual install docs**: document downloading a specific archive and verifying the SHA-256 checksum.
4. **Signed releases**: add provenance or signing once the release process stabilizes.

## Launch checklist

For each release worth announcing:

- [ ] Update README hero if the core value proposition changed.
- [ ] Add or refresh `assets/ai-history-demo.gif`.
- [ ] Post a short technical note in GitHub Discussions.
- [ ] Share one X post with the demo.
- [ ] Share one Reddit post only if the change matches that community.
- [ ] Avoid posting the same copy to multiple communities in one hour.

## Release note template

````markdown
## What changed

- Summarize the user-facing changes.

## Why it matters

- Explain why developers should upgrade or try the release.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/jiantao88/ai-history/master/setup | bash
```

## Verify

```bash
ai-history --version
ai-history list
```
````
