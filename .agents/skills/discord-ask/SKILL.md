---
name: discord-ask
description: >-
  Ask a human for something you can't do yourself on eero: a decision, an
  approval, access or a credential the vault doesn't have, a manual step (deploy,
  store review, DNS, billing), or an answer from Eric. Posts an
  @mention to Discord #eero so it reaches a phone. Use when blocked on a
  person, when a PR needs a human, or to leave an FYI. Not for client communication.
---

# Ask a human (eero)

`ask.sh` in this folder posts to Discord **#eero** and pings the person you name.
The webhook and the list of people you may ping come from Heka Vault at run time.

## When to use

| Situation | Level |
|-----------|-------|
| You need a decision, approval, or answer and can keep working on something else | `action` |
| You can't make progress until a person acts | `blocked`, then stop and end your turn |
| Something a person should know but doesn't need to act on (shipped, merged, deferred) | `fyi` (no ping) |

Don't ask for what you can find yourself: the repo, its docs, secrets already in Heka Vault,
issue history. Don't ask for routine PR review: open the PR and say so in your reply.

## Who you can ping

| Name (`--to`) | Ask them about |
|---------------|----------------|
| `eric` | Anything: decisions, approvals, access and credentials, deploys, money |

`ask.sh --who` prints the live list. Default `--to` is `eric`.

## Command (repo root)

```bash
.agents/skills/discord-ask/ask.sh --level action --to eric \
  --title "Approve prod migration 0042" \
  --body "Adds a NOT NULL column to orders; backfill is in the PR. Waiting on a go/no-go before deploy. Meanwhile I'm on #123." \
  --link "https://…/pull/456"
```

Check the message first with `--dry-run`. Long bodies: `--body -` reads stdin.

## Rules

1. **One message per blocker.** The script skips a repeat of the same title within 24h. Use
   `--force` only when something changed, and say what.
2. **Never** put secrets, tokens, `.env` values, or customer data in the message.
3. Make it answerable from a phone: title = what you need, body = why you're blocked, what you
   tried, the options, and what you'll do meanwhile. Always add `--link` when there is one.
4. After `blocked`, stop work on that task and say in your reply that you asked and who.
5. Exit 3 means this host has no Heka Vault key for the project: don't retry, tell the
   operator in your reply instead.
