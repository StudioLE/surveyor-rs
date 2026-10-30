# surveyor-rs

Version a Cargo repo from git tags and conventional commits, write the versions into `Cargo.toml` and `Cargo.lock`, and render release notes.

- `surveyor version` sets the version of every crate from your commits
- `surveyor notes` prints release notes for the same commits

## Install

Install the latest version with Homebrew:

```shell
brew install studiole/tap/surveyor
```

Or download the binary from [GitHub Releases](https://github.com/StudioLE/surveyor-rs/releases).

## Version

Determine the version and write it into every `Cargo.toml` and `Cargo.lock`:

```shell
surveyor version
```

> [!TIP]
> Preview the changes without writing anything with `--dry-run`, and see why each version was chosen with `surveyor --log-level debug version`.

> [!NOTE]
> Surveyor stops if there are uncommitted changes. Add `--allow-dirty` to carry on anyway, and the version ends `+dirty`, or `+abcdef1-dirty` for development builds.

### How the version is chosen

Surveyor finds the previous release tag, reads the commits since, and increments the version:

| Commits since `v1.2.3` | On `release` | On `main`       | On other branches   |
|------------------------|--------------|-----------------|---------------------|
| `fix: …`               | `1.2.4`      | `1.2.4-alpha.1` | `1.2.4-DEV+abcdef1` |
| `feat: …`              | `1.3.0`      | `1.3.0-alpha.1` | `1.3.0-DEV+abcdef1` |
| `feat!: …`             | `2.0.0`      | `2.0.0-alpha.1` | `2.0.0-DEV+abcdef1` |

The previous release is the newest `v` tag reachable from `HEAD` with no pre-release suffix, so `v1.2.0` counts and `v1.2.0-alpha.1` doesn't. With no release tag, Surveyor starts from `0.0.0`.

The biggest change across the commits wins, ignoring merge commits:

- A breaking change, marked by `!` or a `BREAKING CHANGE` footer, bumps `1.2.3` to `2.0.0`, or `0.2.3` to `0.3.0` before `1.0.0`
- `feat` bumps `1.2.3` to `1.3.0`
- Any other commit bumps `1.2.3` to `1.2.4`, including `chore`, so every build gets a new version

The branch decides the format:

- `release` gives a release, e.g. `1.2.4`
- `main` gives a pre-release, e.g. `1.2.4-alpha.1`, `1.2.4-alpha.2`, …
- Any other branch gives a development build, e.g. `1.2.4-DEV+abcdef1`

> [!TIP]
> Set the version directly with a `Release-As: 2.0.0` footer in a conventional commit.
>
> It must be a plain `major.minor.patch` above the previous release and not already tagged. The newest one wins.

> [!NOTE]
> If `v1.2.4` was already released from another branch, a `fix` on `1.2.3` gives `1.2.5` instead.

> [!NOTE]
> If `HEAD` already has the tag, Surveyor reuses it, sets `tagged=true`, and logs a warning.

Change which commits bump which number, and which branches give which format, in [Configuration](#configuration).

### Crate versions

Each crate in the workspace gets the new version only if it changed.

Say `v1.2.0` was released, then a `fix` to `core` gives `1.2.1`:

| Crate    | Version | Why                      |
|----------|---------|--------------------------|
| `core`   | `1.2.1` | Changed since `v1.2.0`   |
| `app`    | `1.2.1` | Depends on `core`        |
| `macros` | `1.1.0` | Unchanged since `v1.1.0` |

- A crate with `version.workspace = true` always gets the new version, written to `[workspace.package]`
- A crate has changed if a commit touched its directory, or the directory of any workspace crate it depends on, directly or through another crate. Dev dependencies don't count.
- An unchanged crate keeps the version of the first release that included its last change
- On `main`, `alpha` pre-releases count as releases too, so an unchanged crate can keep a version like `1.3.0-alpha.2`. A full release still wins over a pre-release.

When a crate depends on another workspace crate with both `path` and `version`, Surveyor sets `version` to that crate's new version, without the `+…` suffix. It leaves `path`-only dependencies alone, but warns if the crate can be published, since it can't be published without a `version`. Each manifest keeps its formatting and comments.

If `Cargo.lock` exists, Surveyor updates each workspace crate's version in it too, including the `+…` suffix as Cargo does, so `--locked` builds still pass.

### Outputs

If `GITHUB_OUTPUT` is set, `surveyor version` appends these:

| Output               | Value                                                                    |
|----------------------|--------------------------------------------------------------------------|
| `version`            | Version written to the manifests                                         |
| `stage`              | `release`, `prerelease`, or `development`                                |
| `channel`            | Channel name, omitted for development builds                             |
| `change`             | `major`, `minor`, `patch`, or `none`                                     |
| `tagged`             | `true` if the version is already a tag on `HEAD`, else `false`           |
| `cargo_publish_args` | `-p` flag for each bumped crate that can be published, empty if none     |

If `GITHUB_STEP_SUMMARY` is set, it appends a table of these outputs, then a table of every crate's version. Every output except `cargo_publish_args`, and the first table, are written before the Cargo files are edited, so a failed edit still shows the version. `cargo_publish_args` and the crate table are written after, so a failed edit publishes nothing.

> [!TIP]
> A run with only `chore` commits still bumps the patch, but reports `change=none`. Skip releasing those with:
>
> ```yaml
> if: steps.version.outputs.change != 'none'
> ```

> [!TIP]
> A rerun on an already tagged commit reuses the tag. Skip publishing it twice with:
>
> ```yaml
> if: steps.version.outputs.tagged != 'true'
> ```

> [!TIP]
> Publish only the crates that changed, in dependency order, with:
>
> ```yaml
> - name: Publish
>   if: steps.version.outputs.cargo_publish_args != '' && steps.version.outputs.tagged != 'true'
>   run: cargo publish --locked ${{ steps.version.outputs.cargo_publish_args }}
> ```
>
> Publishing several crates at once needs Cargo 1.90 or later.

## Notes

Render release notes for the commits since the previous release:

```shell
surveyor notes > release-notes.md
```

It uses the same commits as `surveyor version`, and:

- Works with uncommitted changes
- Prints nothing if there are no commits
- Appends the notes to `GITHUB_STEP_SUMMARY` too, if set

The built-in template groups commits into these sections, skipping empty ones:

| Section               | Commits                                               |
|-----------------------|-------------------------------------------------------|
| `⚠️ Breaking changes` | Breaking changes, with body and footers shown in full |
| `Features`            | `feat`                                                |
| `Fixes`               | `fix`                                                 |
| `Performance`         | `perf`                                                |
| `Maintenance`         | Everything else, collapsed                            |

In each section:

- Commits are listed oldest first
- Commits with a body or footers expand from a `<details>` summary
- Reverted commits are struck through, unless the revert is itself reverted

Authors show as `@login` when `gh` can match the commit to a GitHub user, else as the git author name. Surveyor reads the repo from `GITHUB_REPOSITORY`, else from `gh repo view`.

## Configuration

Add an optional `surveyor.toml` in the directory Surveyor runs from.

### Version

Each key shown is the default:

```toml
[[channels]]
name = "alpha"
branches = ["main"]
stage = "prerelease"

[[channels]]
name = "release"
branches = ["release"]
stage = "release"

[types]
minor = ["feat"]
patch = ["fix", "perf", "deps", "build", "docs", "revert"]
none = ["ci", "chore", "test", "style", "refactor"]
```

- `[[channels]]` and `[types]` each replace their whole default, so list every entry you want
- The first channel whose `branches` match the current branch wins. A branch with no match gives a development build.
- `branches` are globs where `*` does not cross `/`, so `release/*` matches `release/1.x` only
- Channel names must hold only `0-9`, `A-Z`, `a-z`, and `-`, and not only digits
- List pre-release channels in ASCII order, e.g. `alpha`, `beta`, `rc`, since versions sort by channel name. Surveyor warns otherwise.
- Types match case-insensitively with one trailing `s` ignored, so `tests` matches `test`
- Surveyor fails on unknown keys, invalid globs, or two types that match the same name, e.g. `test` and `tests`

### Notes

Use your own [minijinja](https://github.com/mitsuhiko/minijinja) template for release notes with `template = "release-notes.md.jinja"`, a path relative to the directory Surveyor runs from.

A custom template receives:

- `commits`, oldest first
- `sections`, with `breaking`, `features`, `fixes`, `performance`, and `maintenance`, each oldest first

Each commit has:

- `hash`, `subject`, `body`, and `footers`, each footer with `key` and `value`
- `author`, with `name`, `email`, and `github`, the GitHub login if `gh` matched the author, else empty
- `conventional`, `kind`, `scope`, `breaking`, `breaking_description`, `change`, and `reverted_by`

## CI

Run Surveyor after a full checkout, then build with the written versions:

```yaml
steps:
- uses: actions/checkout@v6
  with:
    fetch-depth: 0
    fetch-tags: true

- name: Install Surveyor
  run: brew install studiole/tap/surveyor

- name: Version
  id: version
  run: surveyor version

- name: Release notes
  env:
    GH_TOKEN: ${{ github.token }}
  run: surveyor notes > "$RUNNER_TEMP/release-notes.md"

- name: Build
  run: cargo build --release
```

`fetch-depth: 0` gives Surveyor every tag and commit. `GH_TOKEN` lets `gh` match authors to GitHub users.

## Contributing

→ [Contributing](CONTRIBUTING.md)

## Background

This project is a Rust rewrite of [surveyor-dotnet](https://github.com/StudioLE/surveyor-dotnet).

## License

Surveyor is provided open source with the [AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html) license.

→ [GNU Affero General Public License](LICENSE.md)
