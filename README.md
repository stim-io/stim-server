# stim-server

Server-side product implementation boundary for `stim`.

## Current baseline

This repo is intentionally still minimal.

At this stage it exists to keep the server-side product boundary explicit inside the workspace without faking a mature server architecture before the first real implementation slice exists.

## What this repo owns

- server-side product communication and coordination for `stim`
- server-side implementation work that should not live in the client repo

## What this repo does not own

- client/application composition that belongs in `modules/stim/`
- shared package-boundary component and theme work that belongs in `modules/stim-packages/`
- paired runtime and gateway semantics that belong in `modules/santi/` and `modules/santi-link/`

## Hygiene rule

Before broader convenience work grows around this repo, keep the baseline clean:

- `main` should be PR-only with squash-first history
- avoid accidental artifact commits through a minimal `.gitignore`
- keep a minimal CI baseline in place now, then replace it with real executable verification when the first server slice lands

## Current CI stance

The current CI baseline is intentionally minimal.

Its job is only to keep the repo from remaining governance-empty during cold start. It is not a substitute for real server verification.
