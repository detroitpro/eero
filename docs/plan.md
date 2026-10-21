# Plan: eero management for a software factory

## Goal

Manage an eero mesh network the way the rest of the infrastructure is managed: from scripts, with
the credential in a secrets vault, with a record of what changed, and with the router kept in step
with an inventory instead of edited by hand in the eero app.

**Who can change the network:** only the operator, from one trusted machine. Coding agents that
work on other projects never hold the eero session. When a project needs a network change (a
fixed address, a port forward, a reboot), it asks the operator, who makes it. A wrapper with
guards is not enough on its own: anyone holding the session token can do anything the eero app
can, so the token stays off the machines those agents run on.

## Why a fork

[erikh/eero](https://github.com/erikh/eero) is a good base: a small, readable Rust client, a mock
server, and a CLI that covers most of the API. Upstream has been quiet since March 2026 and has open
bug reports and an unmerged pull request. We need fixes now, and a few things automation needs
that a personal CLI doesn't. So we maintain this fork in the open. We don't send changes upstream;
anyone is welcome to use them.

## How the credential is handled

eero signs in with a one-time code sent by email or SMS, which gives a long-lived session token.
That shapes everything else:

1. **The operator signs in once**, on the trusted machine, into a private temporary directory, and
   moves the token into the vault. It never stays in `~/.config/eero/`.
2. **The vault is the only copy.** Read keys for the eero project exist only for the trusted
   machine and, later, a monitoring host that runs read-only checks. Agent hosts get none.
3. **The wrapper fetches the token at run time** into `EERO_SESSION_TOKEN` and runs `eeroctl`. The
   token is never passed on a command line or written to disk.
4. **Nobody runs `auth logout`** against the stored session: everything that uses it shares one
   session, so logging out signs all of it out.
5. **A daily check** calls `account get`. A `401` means the session has expired; it alerts the
   operator, who signs in again (step 1). The API offers no refresh that we rely on yet.

## Guardrails

- A thin wrapper on the trusted machine loads the token, drops debug logging, refuses `auth`
  commands and Wi-Fi password changes, and requires an explicit `--yes` for anything that changes
  the network. Every change is announced in the team's chat with the command words, never
  addresses or names.
- Changes are made only when the operator asks for them, including requests passed on from a
  project. The inventory is updated in the same change as the router.
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
| [#1](https://github.com/detroitpro/eero/issues/1) | Session token written to debug logs | Done in v0.2.0 (#12, #14) |
| [#2](https://github.com/detroitpro/eero/issues/2) | `auth login` prints the user token | Done in v0.2.0 (#12) |
| [#3](https://github.com/detroitpro/eero/issues/3) | Token files created world-readable | Done in v0.2.0 (#12) |
| [#4](https://github.com/detroitpro/eero/issues/4) | `device set` silently ignored (API 2.2 vs 2.3), from erikh/eero#1 | Done in v0.2.0 (#13); needs a live check |
| [#5](https://github.com/detroitpro/eero/issues/5) | `diagnostics run` 400 without a symptom, from erikh/eero#3 | Done in v0.2.0 (#14); accepted symptom values unknown |
| [#6](https://github.com/detroitpro/eero/issues/6) | Data usage API, from erikh/eero#2 | Done in v0.2.0 (#10) |
| [#7](https://github.com/detroitpro/eero/issues/7) | DHCP reservations | Done in v0.2.0 (#15); writes need a live check |
| [#8](https://github.com/detroitpro/eero/issues/8) | `--version`; `--yes` for destructive commands | Done in v0.2.0 (#16) |
| [#9](https://github.com/detroitpro/eero/issues/9) | Release binaries for Linux x86_64 | Done in v0.2.0 |
| [#19](https://github.com/detroitpro/eero/issues/19) | Port forwards didn't match the live API | Done in v0.2.1 (#20); list and delete verified live |
| [#23](https://github.com/detroitpro/eero/issues/23) | Deletes reported an error when they succeeded | Done in v0.2.2 |

Live-verified writes so far: `reservation create`, `forward delete` (it worked, but exited 1 before #23).

### Operator integration (outside this repository)

1. A vault project for the eero session, readable only by the trusted machine. Done.
2. An install script for a pinned release of `eeroctl` and the wrapper on that machine, which also
   checks that agent hosts have no eero key, wrapper or binary. Done.
3. A skill for the operator's own agent: the commands, what needs `--yes`, how to handle a
   request from a project, and keeping the inventory in step. Done.
4. The daily session and drift check, with alerts to the team's chat.
5. `--apply` for reservations and port forwards once the drift check has run cleanly for a while.
