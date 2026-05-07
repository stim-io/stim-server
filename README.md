# stim-server

Server-side product implementation boundary and durable product IM message-ledger owner for `stim`.

## Current baseline

This repo is intentionally still minimal.

At this stage it exists to keep the server-side product boundary explicit inside the workspace without faking a mature server architecture before the first real implementation slice exists.

It now also carries the smallest executable dependency proof that `stim-server` can consume the canonical shared contracts from `stim-proto` without redefining them locally.

The next executable baseline now starts in the same narrow posture:

- one in-memory registry adapter
- one register/update API surface
- one discovery API surface
- one agent instance registration/heartbeat projection API
- one participant projection, delivery-target, and chat participant selection API
- Rust-side OpenAPI generation from the real HTTP boundary

## What this repo owns

- server-side product communication and coordination for `stim`
- durable product IM message-ledger facts and product-ledger events for the whole IM system
- product-visible registered agent instance projections maintained by `stim-agents` registration and heartbeat events
- product-visible participant projections, participant delivery targets, and chat participant selection events keyed by `participant_id`
- server-side implementation work that should not live in the client repo

## What this repo does not own

- client/application composition that belongs in `modules/stim/`
- controller operation events used by `modules/stim/` for local app-loop coverage, debugging, and acceptance
- shared package-boundary component and theme work that belongs in `modules/stim-packages/`
- paired runtime, IM-facing agent ledger views, LLM/runtime ledger views, and gateway semantics that belong in `modules/santi/` and `modules/santi-link/`
- local agent process lifecycle management, probing policy, and provider/runtime atomic semantics owned by `modules/stim/` `apps/agents/` and `modules/santi/`

## Ledger boundary rule

`stim-server` product-ledger events and `stim` controller operation events are different layers.

- product-ledger events record durable IM facts
- controller operation events record local execution, projection, debug, and acceptance observations
- `santi` runtime events and provider assembly facts remain runtime-owned

Cross-layer links should use explicit references, correlation ids, and causation ids. Do not rely on another repo's local `conversation_id` or `message_id` as the durable product ownership model.

Participant identity is product-level. Agent and instance ids are technical source markers that can explain where a participant projection came from, but chat surfaces should select `participant_id`. Delivery code may ask `stim-server` to resolve that participant into the current protocol endpoint id; it should not read local agents active-selection state as routing truth.

`docs/contracts/product-message-ledger.md` is the canonical server-side mapping from the shared `stim-proto` message fact/content/relation contract to product IM ledger ownership.

## Hygiene rule

Before broader convenience work grows around this repo, keep the baseline clean:

- `main` should be PR-only with squash-first history
- avoid accidental artifact commits through a minimal `.gitignore`
- keep CI limited to the minimal guard workflow until the server has a real release path worth automating

## Current automation stance

This repo currently carries only the minimal guard workflow required by the workspace governance model.

Add broader automation only when there is a real server release path worth enforcing, rather than preserving placeholder workflow coverage.
