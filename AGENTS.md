# AGENTS

## Purpose

This file manages two things only:

- the stable role of `stim-server/` as the server-side implementation boundary for the `stim` product surface
- the small set of durable repo-baseline rules that should exist before the repo grows real server architecture

Detailed server design should be added later when the first real implementation slice exists.

## Core Constraints

- `stim-server/` owns server-side product communication and coordination for `stim`; it should not absorb client-side UI composition or paired-agent runtime semantics.
- Keep this repo intentionally minimal until a real server implementation exists.
- Do not fake maturity with large placeholder docs or workflows that are not backed by an executable surface.
- Even while minimal, keep the repo boundary explicit so it does not drift into blank-history ambiguity.

## Git Baseline

- `main` should advance through PRs rather than direct pushes.
- Keep force-push protection and branch-deletion protection enabled for `main`.
- Keep squash merge as the default history strategy.
- Keep required green checks in front of merge; the current CI baseline is intentionally minimal until the first executable server surface exists.

## Key File Index

- `AGENTS.md`: stable repo boundary and baseline rules
- `README.md`: minimal repo purpose and baseline status
- `.github/workflows/ci.yml`: minimal cold-start CI baseline for repo hygiene
- `../../AGENTS.md`: repo-root workspace boundary across all attached repos

## Update Rules

- Keep this file short and durable.
- Add a fuller docs tree only when the repo has enough real architecture to justify canonical docs.
- When the first executable server slice appears, evolve the minimal CI baseline into a real executable verification path as part of that slice rather than as an afterthought.
