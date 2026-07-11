//! The principal-aware shape used by the chokepoint — a thin projection
//! of `lb_auth::Principal` that the chokepoint's `assert_reach` / list
//! signatures take. Re-exported from `lb_auth::Principal` so call sites
//! don't grow when the underlying principal shape evolves.

pub use lb_auth::Principal as CarePrincipal;
