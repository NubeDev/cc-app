//! `care.ping` — the loop-proof verb. A trivial tool the host can call to
//! prove the care sidecar is wired end to end (publish → spawn → init →
//! call → reply → unmarshal). Stateless — every reply is a pure function
//! of `(ws, echo)`. The matrix harness in `tests/matrix.rs` exercises this
//! verb with the seeded fixture to prove the loop is alive.

use crate::call::{PingInput, PingReply};

/// Run `care.ping`. Returns opaque-JSON for the wire.
pub fn run(ws: &str, input: &str) -> Result<String, String> {
    // `input` may be empty (an empty body is a valid ping). Default it to
    // a no-op input so a bare `call(ping, "")` still works.
    let parsed: PingInput = if input.trim().is_empty() {
        PingInput { echo: None }
    } else {
        serde_json::from_str(input).map_err(|e| format!("invalid ping input: {e}"))?
    };

    let reply = PingReply {
        ws: ws.to_string(),
        tier: "native",
        ok: true,
        echoed: parsed.echo,
    };
    serde_json::to_string(&reply).map_err(|e| format!("serialize ping reply: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_round_trips_an_echo() {
        let out = run("acme", r#"{"echo":"hello"}"#).expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        assert_eq!(v["ws"], "acme");
        assert_eq!(v["tier"], "native");
        assert_eq!(v["ok"], true);
        assert_eq!(v["echoed"], "hello");
    }

    #[test]
    fn ping_with_empty_input_returns_a_no_echo_reply() {
        let out = run("acme", "").expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        assert_eq!(v["ws"], "acme");
        assert!(v.get("echoed").is_none(), "no echo ⇒ echoed omitted");
    }

    #[test]
    fn ping_with_invalid_json_is_an_error_not_a_panic() {
        assert!(run("acme", "not json").is_err());
    }
}
