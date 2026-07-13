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

- [ ] Matrix rows on channel read/post: post as Ana in Leo's channel → Mia's-mum's reader
      never sees it; **Ana has no path to Mia's channel — including knowing it exists**.
- [ ] Unlink → removed → next read 403 (asserted); reconciler idempotent under double
      events; sweep repairs a hand-broken membership; archive retains history, stops posts.
- [ ] Guardian posting to announcements denied by the generic mechanism (no care hack).
- [ ] Staff room-move swaps memberships.
- [ ] Messaging chrome in `en` + `es` (user-authored message *content* is never
      translated — chrome only; catalog CI gate green).
- [ ] Open questions resolved: distinct `messaging` flag; announcements per center for
      multi-center orgs; removed guardian's authored history stays (record the privacy
      stance).
- [ ] STATUS.md moved.

## Subagent notes

The reconciler is one careful agent (it is the leak vector). Provisioning, policy, three
UI slices fan out. Reviewer brief: drift membership from edges by any sequence of
link/unlink/move/archive events.

## Sources

`../scope/care/messaging-scope.md` · lb `channels/channels-scope.md` · the three persona
docs above · `../scope/care/care-authz-scope.md`.
