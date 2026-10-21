# eero: agent operating manual

`eeroctl` is a command-line tool for eero mesh routers, and `eero-api` is the async Rust client it
is built on. This repository is heka's maintained fork of [erikh/eero](https://github.com/erikh/eero).
We use it to let the heka software factory (people and coding agents) read and manage a home
network's eero from scripts. Fixes land here; we don't send them upstream.

This repository is **public**. Everything in it, including issues, PRs, commit messages and test
fixtures, is read by strangers.

Read [docs/plan.md](docs/plan.md) before non-trivial work. It says what the fork is for and what is
still to do.

## Layout

| Path | What |
|---|---|
| `eero-api/src/client.rs` | `EeroClient`, `ClientBuilder`, the HTTP verbs, the session cookie |
| `eero-api/src/api/` | One file per API area (auth, account, networks, eeros, devices, profiles, forwards, reservations, activity, diagnostics, data usage); each adds methods to `EeroClient` |
| `eero-api/src/types/` | Request and response types. Responses keep unknown fields in `extra` |
| `eero-api/src/credential/` | `CredentialStore` and its backends (file, memory, keyring, 1Password, DPAPI) |
| `eero-api/tests/` | `api_tests.rs` against the mock server in `mock_server/`, serialization and credential tests, opt-in `live_tests.rs` |
| `eeroctl/src/main.rs` | The whole CLI: clap commands, name and `id:` resolution, YAML/JSON/jq output |

## Commands

```bash
make test                                        # clippy, check, offline tests: the PR gate
cargo fmt --all                                  # CI checks formatting
cargo build --release -p eeroctl                 # target/release/eeroctl
make test-live EERO_IP=<gateway>                 # read-only tests against a real network
```

## Rules

- **You have no eero session, and you don't need one.** Everything here is built and tested
  against the mock server. Only the operator can reach a real network, so anything that needs one
  (live tests, checking a new endpoint's shape) is a request to the operator: say exactly which
  read-only commands to run. Never ask for the token.

- **Never log, print or commit a token.** The session cookie is a live credential for someone's
  home network. Header values that carry it are marked sensitive; keep it that way, and keep raw
  auth request and response bodies out of logs. Tests assert this.
- **No real network data anywhere in the repo.** Fixtures, docs, issues and PRs use the made-up
  values already in `tests/mock_server/fixtures.rs` (network `12345`, `11:22:33:44:55:66`,
  `test@example.com`). Redact network IDs, MAC addresses, IPs, SSIDs, emails and phone numbers
  from anything copied out of a live run.
- **eero's API is undocumented.** Reads live under `/2.2/`. Some writes only take effect under
  `/2.3/` and are silently ignored on `2.2` (device updates). When a write "succeeds" but nothing
  changes, check the version first. Note the source when you rely on a shape that other clients
  reverse-engineered.
- **Live tests stay read-only.** Nothing in `live_tests.rs` may change a network.
- **Destructive commands need `--yes`** when there's no terminal to confirm on (reboots and
  deletes). Don't add one without the guard.
- **Every change comes with a mock-server test.** New endpoints get a route in
  `tests/mock_server/routes.rs`, a fixture, and a test in `api_tests.rs`.
- **Licenses:** `eeroctl` is AGPL-3.0-or-later, `eero-api` is MIT. Keep each crate's license.

## Workflow

Work happens in GitHub issues and pull requests on `detroitpro/eero`. One issue per bug or
feature; the PR closes it. CI (`.github/workflows/ci.yml`) must be green before merge. Releases are
tags `vX.Y.Z`; the release workflow attaches Linux binaries with checksums.
