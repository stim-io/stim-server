# stim-server

Server-side product implementation boundary for `stim`.

## Current baseline

This repo is intentionally still minimal.

At this stage it exists to keep the server-side product boundary explicit inside the workspace without faking a mature server architecture before the first real implementation slice exists.

It now also carries the smallest executable dependency proof that `stim-server` can consume the canonical shared contracts from `stim-proto` without redefining them locally.

The next executable baseline now starts in the same narrow posture:

- one in-memory registry adapter
- one register/update API surface
- one discovery API surface
- Rust-side OpenAPI generation from the real HTTP boundary

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
- avoid adding CI/release workflow machinery before the server has a real executable verification or release path worth automating

## Current automation stance

This repo does not currently carry CI or release workflow machinery.

Add automation only when there is a real server verification or release path worth enforcing, rather than preserving placeholder workflow coverage.
