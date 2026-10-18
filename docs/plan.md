# Plan: eero management for a software factory

## Goal

Let a fleet of machines, and the coding agents that run on them, read and manage an eero mesh
network the same way they manage the rest of the infrastructure: from scripts, with the credential
in a secrets vault, with a record of what changed, and with the router kept in step with an
inventory instead of edited by hand in the eero app.

## Why a fork

[erikh/eero](https://github.com/erikh/eero) is a good base: a small, readable Rust client, a mock
server, and a CLI that covers most of the API. Upstream has been quiet since March 2026 and has open
bug reports and an unmerged pull request. We need fixes now, and a few things automation needs
that a personal CLI doesn't. So we maintain this fork in the open. We don't send changes upstream;
anyone is welcome to use them.

## How the credential is handled

eero signs in with a one-time code sent by email or SMS, which gives a long-lived session token.
That shapes everything else:

1. **An operator signs in once**, on a trusted machine, into a private temporary directory, and
   moves the token into the vault. It never stays in `~/.config/eero/` on a shared machine.
2. **The vault is the only copy.** Each host gets its own read-only vault key for the eero
   project, so access can be audited and revoked per host.
3. **Hosts fetch the token at run time** into `EERO_SESSION_TOKEN` and run `eeroctl`. The token
   is never passed on a command line or written to disk.
4. **Nobody runs `auth logout`** from automation: every host shares one session, so logging out
   on one signs out all of them.
5. **A daily check** calls `account get`. A `401` means the session has expired; it alerts the
   operator, who signs in again (step 1). The API offers no refresh that we rely on yet.

## Guardrails

- A thin wrapper on each host loads the token, drops debug logging, refuses `auth` commands and
  Wi-Fi password changes, and requires an explicit `--yes` for anything that changes the network.
  Every change is announced in the team's chat with the host and the command.
- `eeroctl` itself asks before reboots and deletes, and refuses them without `--yes` when nothing
  can answer the prompt.
- Nothing critical depends on this. The API is unofficial and a firmware update can break it; the
  eero app stays the fallback.

## Desired state

The fleet already keeps an inventory of hosts with their reserved addresses and port forwards. A
read-only drift check compares it with `eeroctl reservation list` and `eeroctl forward list` and
reports differences. A later `--apply` mode makes the router match, defaulting to a dry run, and
never deletes something the inventory doesn't mention unless asked.

## Work

### Fork fixes (this repository)

| Issue | Work | Status |
|---|---|---|
| [#1](https://github.com/heka-vault/eero/issues/1) | Session token written to debug logs | |
| [#2](https://github.com/heka-vault/eero/issues/2) | `auth login` prints the user token | |
| [#3](https://github.com/heka-vault/eero/issues/3) | Token files created world-readable | |
| [#4](https://github.com/heka-vault/eero/issues/4) | `device set` silently ignored (API 2.2 vs 2.3), from erikh/eero#1 | |
| [#5](https://github.com/heka-vault/eero/issues/5) | `diagnostics run` 400 without a symptom, from erikh/eero#3 | |
| [#6](https://github.com/heka-vault/eero/issues/6) | Data usage API, from erikh/eero#2 | |
| [#7](https://github.com/heka-vault/eero/issues/7) | DHCP reservations | |
| [#8](https://github.com/heka-vault/eero/issues/8) | `--version`; `--yes` for destructive commands | |
| [#9](https://github.com/heka-vault/eero/issues/9) | Release binaries for Linux x86_64 | |

### Fleet integration (outside this repository)

1. A vault project for the eero session, with a read-only key per host.
2. An install script that puts a pinned release of `eeroctl` and the wrapper on every host, checks
   the checksum, and reports the installed `--version`.
3. The daily session and drift check, with alerts to the team's chat.
4. An agent skill that explains the wrapper, what needs `--yes`, and where the inventory lives.
5. `--apply` for reservations and port forwards once the drift check has run cleanly for a while.
