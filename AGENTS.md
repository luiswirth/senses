---
description: "Senses: small readings that deliver what an agent perceives into its inbox, through MindIO, part of Ag² (`~/dev/agentics`). Use when working on a sense or adding one."
---

# senses

Each is its own small piece, general, so anyone can use it by default, never tied to Luis's setup.
The domain senses.sh is Luis's, unused for now.

- A sense is a reading of MindIO (`~/dev/agentics/mindio`): a command its take runs at each step, listed in `mindio/readings` of the user's config directory, which does nothing without `MINDIO`; no harness knows a sense.
  It delivers through the crate `mindio`, without its command line, as `kind=reading`, which wakes no agent unless the agent's waking rules say so.
- A sense follows MindIO's rule for readings: it reports a change, not a clock, delivers only as a reading crosses a mark, and keeps its state, such as the mark told last, in a file of the agent's IO named after it.
- Each sense is a crate of the workspace, its command named after what it senses.

## Senses

- `clock`: the date and time, once 15 minutes have passed since it last told them, as MindIO's `AGENTS.md` says.
- `claude-usage`: the usage limits of the user's Claude subscription, told by the tenth as each crosses one, either way.
  - It reads the login of the user's Claude Code where Claude Code keeps it, the macOS keychain or `.credentials.json` of its config directory, only reads it, and never stores or logs it; Claude Code renews it.
  - It asks the endpoint Claude Code shows `/usage` from, `api.anthropic.com/api/oauth/usage`, undocumented, at most every five minutes, and reads its list `limits`, classified by `kind`.
  - Claude Code tells the windows to its status line but not to hooks, and pi tells them to nothing, so the endpoint is the one source every harness shares.
- `others`: the statuses of the other agents of the machine and of its peers' agents, asked at most every five minutes, told as they change; its first look tells them all, and how to set one's own (MindIO's "Status").
