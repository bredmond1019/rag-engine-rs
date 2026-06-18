# Vendored fork of ollama-rs

**Upstream:** https://github.com/pepperoni21/ollama-rs  
**Upstream version at fork time:** 0.2.0  
**License:** MIT (see LICENSE.md)

## Why we vendor instead of using upstream

The upstream crate is at 0.3.5. The API for chat-with-history changed between
0.2.0 and 0.3.5 in a way that is incompatible with this project's per-session
actor model:

- **Upstream 0.3.5** — caller passes an `Arc<Mutex<C: ChatHistory>>` to
  `send_chat_messages_with_history_stream`; `new_default_with_history` was removed.
- **This fork (0.2.0)** — history is keyed by a string ID inside `Ollama`,
  and `new_default_with_history(max_messages)` creates an instance with
  built-in bounded history. This maps naturally to the Actix actor model
  where each `ChatSession` holds an `AIService` with its own history.

Switching to upstream would require refactoring the chat layer to manage
`Arc<Mutex<MessagesHistory>>` externally per session — out of scope for a
portfolio-packaging revamp.

## What was changed from upstream 0.2.0

One commit (`40ba68f`) on top of the upstream 0.2.0 tag:

- **`src/generation/chat/mod.rs`** — fixed a borrow-checker issue in
  `get_chat_messages_by_id`: removed a redundant `binding` variable that
  was clobbering the real history arc before it could be borrowed. The fix
  avoids allocating a fresh empty history when one already exists.
