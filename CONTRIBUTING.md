# Contributing

## Structure

Cargo workspace with one crate, `crates/surveyor`:

- `src/app/` - CLI bootstrap, subcommand dispatch, DI registration
- `src/commands/` - `version` and `notes` handlers
- `src/schema/` - types shared across domains, such as `Commit`, `ChannelName`, and `RepoName`
- `src/extensions/` - extension traits on existing types, such as `Version` and `Command`
- `src/services/` - adapters for `git`, `cargo`, `gh`, and GitHub Actions files, and the services that use them
- `tests/` - integration tests against real `git` repos and Cargo workspaces, using `Type::fixture()` fixtures from the `testing` feature

Built with `studiole-di`, `studiole-report`, `studiole-logging`, and `usage-rs`.

## Test

Run every test:

```shell
cargo test --all-targets --all-features
```

Integration tests need `git` on `PATH`. Unit tests replace adapters with hand-written `<Trait>Stub` and `<Trait>Spy` doubles and use `insta` snapshots.
