# Releasing

Releases are built by [dist](https://github.com/axodotdev/cargo-dist) on GitHub Actions. Pushing a version tag builds keyquest for macOS (Apple Silicon and Intel), Linux (x86_64 and ARM) and Windows, then publishes a GitHub Release with the binaries and the install scripts from the README.

## Cut a release

1. Set the new `version` in `Cargo.toml`, for example `0.2.0`, then run `cargo build` so `Cargo.lock` picks it up
2. Check it: `cargo test && cargo clippy --all-targets`
3. Commit: `git commit -am "chore: release v0.2.0"`
4. Tag and push: `git push && git tag v0.2.0 && git push origin v0.2.0`
5. Watch the Release workflow in the Actions tab. After about 10 minutes the release shows up on the releases page

The tag must be the `Cargo.toml` version with a `v` in front. A version with a suffix, such as `v0.2.0-beta.1`, is published as a prerelease, and the install scripts keep pointing at the latest full release.

## Check before tagging

You need dist locally for this: `brew install cargo-dist`. CI installs its own copy.

- `dist plan` lists what a release will build
- `dist build` builds the artifacts for this computer into `target/distrib/`

## Change the release setup

Targets, installers and the install path live in `dist-workspace.toml`. After editing it, run `dist generate` and commit both files. Don't edit `.github/workflows/release.yml` by hand; `dist generate` overwrites it.

To move to a newer dist, run `brew upgrade cargo-dist`, then `dist init`. It updates `cargo-dist-version` and regenerates the workflow.

## If a release fails

Fix the problem, then move the tag to the new commit and push it again:

```sh
git tag -d v0.2.0 && git push origin :refs/tags/v0.2.0
git tag v0.2.0 && git push origin v0.2.0
```

If the failed run already created a GitHub Release, delete it on the releases page first.
