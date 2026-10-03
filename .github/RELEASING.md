# Releasing the workspace

The crates `wc3-derive`, `wc3`, `bevy-mpq`, and `bevy-wc3` are released together.
Their package versions inherit `[workspace.package].version` from the root
`Cargo.toml`; internal dependency requirements live in
`[workspace.dependencies]` in the same file. The release script synchronizes
these values and creates a matching Git tag. Pushing that tag starts the
publishing workflow.

## Development versions

Between releases, the shared package version and internal dependency
requirements can use a prerelease such as `0.3.0-dev.0`. Individual development
builds are identified by their Git commit; they do not need separate version
numbers or tags. When preparing a release, supply the final version to the
script, such as `0.3.0` or `0.3.0-alpha.1` for a published preview.

Cargo requires explicit version requirements for published dependencies, so
internal dependencies cannot inherit `workspace.package.version` directly.
Keep those requirements synchronized when changing development versions, and
refresh `Cargo.lock` with `cargo check --workspace`.

## Preparing a release

The script requires Python 3.9 or newer, Git, and Cargo. Use a Cargo toolchain
that supports workspace publishing; the publishing workflow uses Rust 1.99.0.
Run the following command from the repository root to preview a release:

```sh
python3 scripts/release.py 0.3.0
```

The preview shows the proposed manifest changes, expected workspace version
changes in `Cargo.lock`, and the commands execution would run. It also reports
uncommitted files, an existing local tag, or a detached HEAD that would block
execution. It does not build or validate the packages; Cargo resolves the
actual lockfile when executing the plan.

After reviewing the preview, commit or stash existing changes and run:

```sh
python3 scripts/release.py 0.3.0 --execute
```

Execution updates the root manifest, runs `cargo check --workspace` to refresh
the lockfile, and validates packaging with
`cargo publish --workspace --dry-run --locked`. It then commits the manifest
and lockfile changes and creates the annotated local tag `v0.3.0`. If those
files already contain the requested versions and Cargo leaves them unchanged,
the script skips the commit and tags the current commit. When installed, the
local Codex commit timestamp helper is used for the release commit.

If a command fails, execution stops and leaves any file changes or completed
commit available for inspection. Check `git status` and the command's error
before retrying; execution requires a clean working tree. The script does not
push refs or publish crates.

## Publishing

For automated release, configure a GitHub trusted publisher for
each of the four crates in its crates.io settings. Use the following values
for every crate:

| Field | Value |
| --- | --- |
| Repository owner | `jonathanvdc` |
| Repository name | `wc3` |
| Workflow filename | `publish.yml` |
| Environment | Leave unset |

The environment is unset because the publishing job does not declare one.
The workflow obtains a short-lived crates.io token through OIDC, so it does
not require a stored registry token in GitHub secrets.

Push the release commit through the repository's normal review process and
wait for CI to pass before pushing its tag. For a prepared `0.3.0` release:

```sh
git push origin v0.3.0
```

Pull requests and pushes to `main` run the workspace package dry run. Pushing
a `v*` tag starts the publishing job in
[the publishing workflow](workflows/publish.yml), which checks that every
crate version matches the tag, runs the workspace tests, and publishes the
crates in dependency order. Check the GitHub Actions result to confirm that
publishing completed.
