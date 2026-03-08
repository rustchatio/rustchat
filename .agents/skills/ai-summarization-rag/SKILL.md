---
name: ai-summarization-rag
description: Workflow for retrieval-augmented thread summarization using vector search, prompt templates, and deterministic structured outputs.
license: MIT
---

# AI Summarization RAG

Use this skill when implementing AI summarization features that rely on retrieval-augmented generation.

## Workflow

1. Fetch thread context:
   - Retrieve relevant messages and metadata from the source of truth.
   - Use vector retrieval for semantic recall and include deterministic filters (workspace/team/channel/thread/time scope).
2. Apply prompt template:
   - Build a fixed prompt structure with task instructions, context, and output schema.
   - Include explicit "do not invent facts" constraints.
3. Return structured summary:
   - Emit a deterministic schema (stable field names/order).
   - Attach provenance/citations back to retrieved context entries.

## Output Contract

Use a stable summary structure with explicit keys (example):
- `summary`
- `key_points`
- `open_questions`
- `action_items`
- `citations`

If retrieval confidence is low or evidence is insufficient, return a constrained fallback stating insufficiency instead of generating speculative content.

## Implementation Constraints

- Keep retrieval and generation steps separable for debugging and auditability.
- Ensure citation entries are traceable to concrete source chunks/records.
- Avoid non-deterministic output shape changes without explicit versioning.
- For compatibility-sensitive surfaces, preserve existing API response signatures.

## Verification Expectations

- Unit/integration coverage for prompt shaping and response schema validation.
- Regression checks for citation linkage and deterministic field presence.
- Manual spot-check with representative long threads and sparse-context threads.
