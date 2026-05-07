# Product Message Ledger

This contract defines how `stim-server` owns product IM message-ledger facts.

The shared vocabulary lives in `stim-proto` as message facts, content
references, and relations. This file defines only the product-server mapping.

## Ownership

`stim-server` is the source of truth for product IM messages.

It owns:

- product-visible message facts
- product participant identity
- product conversation/session membership and permission policy
- current transcript projections
- delivery, read, unread, latest-message, and deletion/redaction projections
- async projections for search, analytics, audit, and columnar scans

It does not own:

- controller operation/debug events
- local sidecar process supervision
- `santi` turns, provider assembly, tool-call execution, compacts, or runtime
  memory

Runtime artifacts from agents may be referenced by product facts when a product
surface intentionally exposes them, but they do not become product-ledger truth
just because they were observed by the client or controller.

## Shared contract mapping

Product ledger append records should map to `stim-proto` `MessageFactEnvelope`.

Required product mapping:

- `ledger_id`: stable product ledger namespace owned by `stim-server`
- `fact_id`: one immutable product-ledger fact id
- `message_id`: durable product message id
- `participant_id`: product participant identity
- `kind`: product-visible message kind
- `occurred_at`: product event time
- `ledger_seq`: product ordering projection when available
- `causation_id` and `correlation_id`: explicit links to prior product facts or
  cross-ledger operations
- `source`: technical source facts, such as an agent or instance marker, without
  replacing `participant_id`

Current-state reads should map to `MessageCurrentProjection`. The current
projection is a read model, not the immutable source of truth.

## Fact categories

Product message facts should stay small and append-oriented:

- create a message
- append streamed content chunks to a live message
- finalize a live message as completed or failed
- revise message content
- change product visibility state
- record a relation such as reply, quote, forward, redaction, or compact cover
- record delivery/read/unread state

Use relation records for semantic links. Do not encode reply, forward, redaction,
compact coverage, or tool-result linkage only inside free-form message content.

## Streaming lifecycle

The first online ledger API uses an explicit live-message lifecycle:

1. create a chat session
2. create a message with `state=pending` or a current terminal state
3. append ordered chunks while the message is live
4. finalize the message as `completed` or `failed`

The current projection returns `text` for fast transcript reads and keeps
`chunks[]` as ordered content facts. Future patch/rewrite support should extend
this lifecycle with revision facts against the same `message_id` and `version`,
not by replacing controller-local strings or by adding a separate renderer-owned
message model.

Agent-era process artifacts such as `thinking`, `tool-call`, and `tool-result`
are message kinds first. Whether they are rendered, read-only, or included in
provider context is a projection/assembly policy and should not be hard-coded
into the ledger fact category.

## Content mapping

Message facts should carry `MessageContentRef` instead of requiring inline
content.

Recommended product content stores:

- text/html records in typed product tables
- file/audio/video/image metadata in typed product tables
- large objects in OSS/blob storage with key, optional bucket, checksum,
  `mime_type`, and `byte_size`
- columnar fact rows as an async projection, not the product write path

Product tables may differ from `santi` runtime tables. Preserve shared ids and
relations; do not force identical storage layout.

## Online truth and projections

The online product path needs strong current-state behavior for small scoped
queries:

- idempotent send/retry
- edit and redaction visibility
- latest-message list
- unread/read state
- participant permissions
- delivery target and delivery state
- conversation continuity

Keep that online truth in an OLTP or wide-row-friendly owner store with
transactional/current projections. A columnar system may mirror
`MessageFactEnvelope` rows for scans, audit, compact input, context analytics,
and search/index feeding, but it is not the only synchronous source of truth
until the online behavior is proven there.

## Cross-ledger links

When product messages involve `santi`, keep the layers explicit:

- `participant_id` names the product chat participant
- `agent_id` and `instance_id` are source/observability markers
- `santi` session/message/turn/tool ids are external references
- controller operation ids are local execution references

Do not rely on identical local `conversation_id` or `message_id` values across
repos as the ownership model.
