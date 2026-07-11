//! Orchestrator-owned authz cap + table constants (the era-2 reach grant).
//!
//! These name the ONE scoped-grant surface the guardian-reach chokepoint
//! resolves through in era 2. The chokepoint derives a per-child scoped grant
//! under [`REACH_CAP`] on `guardianship.link` and reads it back via
//! `authz.check_scoped` / `authz.scope_filter` keyed on `(REACH_CAP,
//! REACH_TABLE)`. Keeping these in one named file (not scattered string
//! literals) means the derive path and the read path can never drift — a
//! mismatch would be a lockout (grant under cap X, read under cap Y) or a
//! leak. Orchestrator owns these; no verb file re-decides them.

/// The capability the guardian-reach scoped grants are keyed on. A guardian
/// holds this cap scoped (via `Scope::Ids`) to exactly the children they hold
/// a live `guardianship` edge to. It is a **reach** cap — distinct from the
/// per-verb `mcp:care.<verb>:call` caps the host wall checks first — so the
/// row-level reach union is one `scope_filter` read regardless of which read
/// verb is asking. Never granted `All` to a guardian (that would be a leak);
/// admins reach everything via the chokepoint's audited role check, not a
/// grant.
pub const REACH_CAP: &str = "mcp:care.reach.child:call";

/// The lb store table the reach selector narrows over. Opaque to the core
/// (rule 10) — the care extension owns that "child" means a child record.
pub const REACH_TABLE: &str = "child";
