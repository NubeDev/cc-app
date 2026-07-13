//! The LIVE-NODE wire-in proof — the honest end-to-end test that the care
//! sidecar is reachable on a REAL node and that the node's durable store is the
//! single source of truth (Part A + Part B, no mocks — CLAUDE.md rule 4).
//!
//! Every earlier test proved a HALF: `live_wire.rs` drives `Care::boot`
//! in-process; `matrix_era2*.rs` drives the chokepoint's callbacks against a
//! booted gateway with an IN-PROCESS `SidecarClient`. NONE of them stood the
//! sidecar up as a real OS child the host spawned. This one does:
//!
//!   1. Boot a real `Node` over a real **on-disk** store + a real
//!      `lb-role-gateway` on a real TCP port.
//!   2. `install_native(care)` — the SAME path cc-node's boot uses
//!      (`care_mount::mount`) — which SPAWNS the real `care` binary as an OS
//!      child and registers it in the MCP routing registry.
//!   3. Drive the roster over real HTTP `POST /mcp/call` with an admin token —
//!      the exact surface the browser shell + `scripts/seed.sh` ride.
//!   4. Assert an admin READ sees the seeded data — proving the sidecar's writes
//!      landed in the NODE's durable store over the host callback (Part B), NOT a
//!      private sidecar store.
//!   5. **Restart:** drop the node + sidecar, re-open the SAME on-disk store in a
//!      fresh node, re-install care, and assert the roster is STILL readable —
//!      the durability proof (the node store, not the child, owns the records).
//!   6. **Rule 7 (sacred) — ENFORCED in-sidecar.** A LINKED guardian (Ana)
//!      reaches her child; a STRANGER guardian (Mallory) is DENIED — 403 on
//!      `child.get`, EMPTY on `child.list`. Green as of `sdk-v0.4.0` /
//!      `node-v0.4.0` (native-caller-identity scope): the host stamps the caller
//!      into the native call frame, `Care::call_with_caller` projects it, and the
//!      chokepoint asks reach ABOUT the caller (`subject = caller.sub`) behind the
//!      extension's `mcp:authz.delegate_reach:call` install grant. See
//!      `docs/debugging/authz/native-sidecar-not-spawned-and-caller-identity-not-propagated.md`.
//!
//! `#[ignore]` by default (spawns an OS child): needs `cargo build -p care`
//! first. Run: `cargo test -p care --test live_node -- --ignored`.

#[path = "live_node_support.rs"]
mod support;

use lb_store::Store;
use serde_json::json;
use support::{admin_token, boot_and_install, guardian_token, mcp, seed_roster};

/// THE full end-to-end proof. `#[ignore]` — spawns the real `care` OS child;
/// run with `cargo test -p care --test live_node -- --ignored`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "spawns the real care OS child; run with --ignored after `cargo build -p care`"]
async fn care_roster_lands_in_the_node_store_and_survives_restart() {
    // A durable on-disk store so the restart half re-opens the SAME data.
    let dir = std::env::temp_dir().join(format!("cc-live-node-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store_url = format!("file://{}", dir.join("store").display());

    let http = reqwest::Client::new();

    // ── Phase 1: fresh node → install care → seed the roster over HTTP ──
    {
        let store = Store::open(&store_url).await.expect("open durable store");
        let (_node, key, base) = boot_and_install(store).await;

        let admin = admin_token(
            &key,
            &[
                "mcp:care.center.create:call",
                "mcp:care.center.list:call",
                "mcp:care.room.create:call",
                "mcp:care.guardian.create:call",
                "mcp:care.child.create:call",
                "mcp:care.child.get:call",
                "mcp:care.guardianship.link:call",
            ],
        );
        seed_roster(&http, &base, &admin).await;

        // Admin READ sees the data → the sidecar's writes landed in the NODE
        // store over the callback (Part B), visible to a DIFFERENT reader.
        let (c, b) = mcp(&http, &base, &admin, "care.center.list", json!({})).await;
        assert_eq!(c, 200, "center.list failed: {b}");
        assert!(
            b.as_array()
                .map(|a| a.iter().any(|x| x["name"] == "Sunshine Childcare"))
                .unwrap_or(false),
            "seeded center not visible to an admin read: {b}"
        );

        let (c, b) = mcp(&http, &base, &admin, "care.child.get", json!({"id":"leo"})).await;
        assert_eq!(c, 200, "child.get leo failed: {b}");
        assert_eq!(b["name"], "Leo García", "seeded child not readable: {b}");

        // ── Rule 7 (sacred): guardian isolation — ENFORCED in-sidecar ──
        //
        // These assertions PROVE guardian isolation over the live native sidecar,
        // end to end on a real spawned OS child. They are GREEN as of the
        // `sdk-v0.4.0` / `node-v0.4.0` pin (native-caller-identity scope): the host
        // stamps the authorized caller into the native call frame
        // (`CallParams.caller`), `Care::call_with_caller` projects it per dispatch,
        // and the chokepoint asks `authz.check_scoped`/`scope_filter` ABOUT the
        // caller (`subject = caller.sub`) behind the extension's
        // `mcp:authz.delegate_reach:call` install grant. So a LINKED guardian
        // reaches her child, a STRANGER guardian is denied (403 on get, EMPTY on
        // list), and admins reach everything — the row-level second gate is real.
        let ana = guardian_token(&key, "ana");
        let (ana_c, _b) = mcp(&http, &base, &ana, "care.child.get", json!({"id":"leo"})).await;

        let stranger = guardian_token(&key, "mallory");
        let (str_c, str_b) = mcp(
            &http,
            &base,
            &stranger,
            "care.child.get",
            json!({"id":"leo"}),
        )
        .await;
        let (list_c, list_b) = mcp(&http, &base, &stranger, "care.child.list", json!({})).await;
        let stranger_kids = list_b.as_array().map(|a| a.len()).unwrap_or(0);
        eprintln!(
            "RULE-7 (enforced in-sidecar, native-caller-identity): \
             Ana→leo={ana_c} (want 200), stranger→leo={str_c} (want !=200), \
             stranger child.list={list_c} with {stranger_kids} kids (want 0)"
        );

        // Ana (LINKED to Leo) reaches her own child.
        assert_eq!(ana_c, 200, "guardian Ana must reach her linked child Leo");
        // A STRANGER guardian (no edge to Leo) is DENIED on get — the sacred
        // cross-family deny (403, not 200). A leak here is the existential bug.
        assert_ne!(
            str_c, 200,
            "a stranger guardian must NOT reach Leo (rule 7): {str_b}"
        );
        // …and their child.list is EMPTY (list denies by returning zero rows,
        // never an error — the chokepoint's list contract).
        assert!(
            stranger_kids == 0,
            "stranger guardian's child.list must be EMPTY (rule 7), got {stranger_kids}: {list_b}"
        );

        // node + sidecar drop here (end of scope) → the "restart".
    }

    // A beat for the dropped sidecar's OS child to exit + release the store.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // ── Phase 2: RESTART — re-open the SAME store, re-install care, re-read ──
    {
        let store = Store::open(&store_url)
            .await
            .expect("re-open durable store");
        let (_node, key, base) = boot_and_install(store).await;

        let admin = admin_token(
            &key,
            &["mcp:care.center.list:call", "mcp:care.child.get:call"],
        );

        // The roster is STILL there — proving the node's durable store owned the
        // records, not the ephemeral first-boot sidecar.
        let (c, b) = mcp(&http, &base, &admin, "care.center.list", json!({})).await;
        assert_eq!(c, 200, "post-restart center.list failed: {b}");
        assert!(
            b.as_array()
                .map(|a| a.iter().any(|x| x["name"] == "Sunshine Childcare"))
                .unwrap_or(false),
            "seeded center did NOT survive the restart: {b}"
        );

        let (c, b) = mcp(&http, &base, &admin, "care.child.get", json!({"id":"leo"})).await;
        assert_eq!(c, 200, "post-restart child.get leo failed: {b}");
        assert_eq!(
            b["name"], "Leo García",
            "seeded child did NOT survive the restart: {b}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}
