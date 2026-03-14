---
name: release
description: "Bump version, update CHANGELOG.md, commit, and push to trigger a release. Pass the version bump type: major, minor, or patch."
---

# Release

Bump the version, update the changelog, commit, and push to trigger the GitHub Actions release workflow.

## Steps

1. Read the current version from `Cargo.toml`.
2. Determine the new version based on the user's argument:
   - `patch` (default if no argument): `0.1.0` → `0.1.1`
   - `minor`: `0.1.0` → `0.2.0`
   - `major`: `0.1.0` → `1.0.0`
   - If the argument looks like an explicit version (e.g. `1.2.3`), use it directly.
3. Update the `version` field in `Cargo.toml`.
4. Update `CHANGELOG.md`:
   - Rename `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD` (using today's date).
   - Add a new empty `## [Unreleased]` section above it.
5. Run `cargo build` to verify compilation.
6. Commit both files with message: `Release vX.Y.Z`
7. Push to both `origin` and `upstream`.
8. Show the user the new version and remind them that the GitHub Actions workflow will create the tag and release automatically.
