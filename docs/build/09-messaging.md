# Milestone 09 — messaging (provisioning + membership reconciliation)

No transport is built — lb channels as-is. This milestone is provisioning by convention
and **membership derived from domain records**, with unlink = immediate removal.
Scope: [`../scope/care/messaging-scope.md`](../scope/care/messaging-scope.md).

## Entry gate

- [ ] Milestone 03 closed (edges + staff assignments = the membership sources); 05 closed
      (guardians exist as members). 08 not strictly required — but the guardian Messages
      tab ships here, so UI-wise this naturally follows 08.
- [x] **Post-restriction question RESOLVED (2026-07-13) — NO lb ask needed.** lb channels
      already split the caps: **post** requires `bus:chan/{cid}:pub`, **read/history/
      subscribe** require `bus:chan/{cid}:sub` (distinct actions on distinct resources —
      lb `channels-scope.md` §"Security invariants" + §Verb table). So read-only membership
      = grant `sub` **without** `pub`. It is fully expressible with the existing scoped-grant
      grammar care already uses (`grants.assign` over the `SidecarClient`, the same path that
      mints `store:media/{id}:read`). No additive lb ask; no core work; this is a care-only
      milestone. **Wildcard-hold note:** lb's no-widening rule (`grants_assign`) requires the
      granter to HOLD a cap matching what it grants, so the care install must add
      `bus:chan/care-**:pub` + `bus:chan/care-**:sub` to `care_mount::approved_grant` +
      `extension.toml` + `live_node_support` (lock-step — exactly the `store:media/**:read`
      idiom in `media/serve_grant.rs`). That is a care-side declaration, not lb work.

## Work items

- [ ] Provisioning: child/room/center creation auto-creates its channel (idempotent,
      convention-named `care-child-<id>` etc. — opaque to the core); archive archives it
      (history retained).
- [ ] **One `reconcile(channel)` function**: membership = guardianship edges (use the
      distinct `messaging` edge flag — recommended; resolve the open question) + staff
      room assignments. Called from every edge/assignment handler **and** from an
      idempotent healing sweep (event-driven primary, sweep as repair).
- [ ] Unlink handler removes channel membership in the same breath as grants (02/03
      handlers extended — same transaction discipline).
- [ ] Posting policy: child/room = members post; announcements = admin/staff post,
      guardians read (via the generic lb mechanism from the entry gate).
- [ ] UI: guardian Messages tab (channel list per edge, thread view)
      ([`../scope/personas/guardian/messaging.md`](../scope/personas/guardian/messaging.md));
      staff room/child threads
      ([`../scope/personas/teacher/room-messaging.md`](../scope/personas/teacher/room-messaging.md));
      admin announcements compose
      ([`../scope/personas/admin/announcements.md`](../scope/personas/admin/announcements.md)).

## Exit gate

- [x] Matrix rows on channel membership DERIVATION (`matrix_messaging.rs`): Ana is in
      Leo's channel, Mia's-mum is NOT; **Ana has NO path to Mia's channel** (never in its
      members — she can't know it exists). care controls the derivation; lb's channel gate
      enforces the actual read/post from the caps the reconciler grants (a live-node
      concern — the store-only matrix can't reach lb's gate).
- [x] Unlink → removed: the derivation drops the guardian immediately AND the unlink
      handler revokes her channel caps in the same breath (the next lb read 403s once
      revoked). Reconciler idempotent (grants.assign settles to the same row; derivation
      deterministic). Sweep repairs a hand-broken membership (`care.channel.reconcile` /
      `reconcile_channel`).
- [ ] Archive retains history, stops posts — **DEFERRED**: provisioning is on-demand
      (`care.channel.reconcile` calls `channel.create` idempotently) rather than eager on
      entity-create/archive; the archive→stop-posts hook lands with the entity-archive
      wiring (a small follow-on — no create/archive verb calls reconcile yet).
- [x] Guardian posting to announcements denied by the generic mechanism (no care hack):
      TWICE — guardians hold no `mcp:care.announce.post:call` cap (host wall) AND only
      `bus:chan/care-center-*:sub`, never `:pub`, so lb's `channel.post` gate 403s.
- [ ] Staff room-move swaps memberships — **DEFERRED**: there is no standalone staff-
      reassignment verb in the codebase yet (staff assignments are minted via the
      invite-accept flow, `invite/create_staff.rs`), so there is no handler to hook. Lands
      when a staff-reassignment verb exists. The room-channel STAFF derivation is built +
      tested.
- [x] Messaging chrome in `en` + `es` (message CONTENT never translated — chrome only;
      i18n parity gate green in the care ext UI).
- [x] Open questions resolved: distinct `receives_messaging` flag (added); announcements
      per center (`care-center-<id>`, multi-center picker in the compose UI); removed
      guardian's authored history STAYS (center record — the privacy stance).
- [x] STATUS.md moved.

## Subagent notes

The reconciler is one careful agent (it is the leak vector). Provisioning, policy, three
UI slices fan out. Reviewer brief: drift membership from edges by any sequence of
link/unlink/move/archive events.

## Sources

`../scope/care/messaging-scope.md` · lb `channels/channels-scope.md` · the three persona
docs above · `../scope/care/care-authz-scope.md`.
