import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Segmented } from "../../components/ui/segmented";
import { useT } from "../../hooks/useT";
import type { ChildRow } from "../child/ChildrenListPage";
import { DaySummary } from "./DaySummary";
import { EntryRow } from "./EntryRow";
import {
  entryKey,
  isoDay,
  mergeEntries,
  todayStartIso,
  type DailyLog,
  type LogDayReply,
  type LogListReply,
} from "./shared";

// How often the guardian feed re-polls `log.list` for new entries.
//
// LIVE-FEED NOTE (documented decision): `care.feed.watch` authorizes a per-child
// SSE subscription and returns a gateway `stream_path` to open with the session
// token. But the ext-ui-sdk runtime exposes ONLY `useMcpClient()` + `useSession()`
// — it surfaces neither the gateway base URL nor the raw session token, so the UI
// cannot correctly construct an `EventSource(stream_path + "&token=…")` today.
// Rather than guess the origin/token, v1 authorizes via `feed.watch` (so the
// reach gate + descriptor are exercised) and then POLLS `log.list` on this
// interval for liveness. TODO(sse): once the runtime exposes the gateway origin +
// session token, swap this poll for an EventSource on `watch.stream_path`.
const POLL_MS = 20000;

export function FeedPage() {
  const t = useT();
  const api = useCareApi();
  const since = useMemo(() => todayStartIso(), []);
  const today = useMemo(() => isoDay(), []);

  const [children, setChildren] = useState<ChildRow[] | null>(null);
  const [childId, setChildId] = useState<string | null>(null);

  useEffect(() => {
    api
      .list<ChildRow>("child")
      .then((c) => {
        const active = c.filter((x) => !x.archived);
        setChildren(active);
        setChildId(active[0]?.id ?? null);
      })
      .catch(() => setChildren([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <main className="pb-24">
      <LargeTitle>{t("nav.feed")}</LargeTitle>

      {children && children.length > 1 && childId && (
        <div className="px-4 pb-3">
          <Segmented<string>
            value={childId}
            onChange={setChildId}
            columns={Math.min(children.length, 3)}
            segments={children.map((c) => ({ value: c.id, label: c.name }))}
          />
        </div>
      )}

      {children === null ? (
        <FeedSkeleton />
      ) : !childId ? (
        <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">
          {t("feed.no_children")}
        </p>
      ) : (
        <ChildFeed key={childId} childId={childId} since={since} date={today} />
      )}
    </main>
  );
}

// The live-appending timeline for ONE child. Keyed by childId so switching child
// remounts a clean stream. Authorizes via feed.watch, loads the day rollup +
// the first list page, then polls for new entries (see POLL_MS note).
function ChildFeed({ childId, since, date }: { childId: string; since: string; date: string }) {
  const t = useT();
  const api = useCareApi();
  const [entries, setEntries] = useState<DailyLog[] | null>(null);
  const [summary, setSummary] = useState<Record<string, number>>({});
  // Local-only ack state (see acknowledge() below for why v1 is optimistic).
  const [acked, setAcked] = useState<Set<string>>(new Set());
  const mounted = useRef(true);

  const poll = useCallback(async () => {
    try {
      const reply = await api.run<LogListReply>("log.list", { child_id: childId, since });
      if (mounted.current) setEntries((prev) => mergeEntries(prev ?? [], reply.entries ?? []));
    } catch {
      if (mounted.current) setEntries((prev) => prev ?? []);
    }
  }, [api, childId, since]);

  useEffect(() => {
    mounted.current = true;
    // Authorize the live subscription (exercises the reach gate + returns the
    // stream descriptor). We don't open the SSE yet — see POLL_MS note.
    api.run("feed.watch", { child_id: childId }).catch(() => {});
    api
      .run<LogDayReply>("log.day", { child_id: childId, date })
      .then((d) => mounted.current && setSummary(d.summary ?? {}))
      .catch(() => {});
    poll();
    const timer = setInterval(poll, POLL_MS);
    return () => {
      mounted.current = false;
      clearInterval(timer);
    };
  }, [api, childId, date, poll]);

  // ACKNOWLEDGE (v1, optimistic-local). The `acknowledged` flag lives on the
  // incident record, but `log.list`/`log.day` expose no row id, and guardians
  // hold no write cap (`log.correct` is staff-only), so the UI cannot flip the
  // server flag today. We record the ack locally so the parent gets immediate
  // confirmation. TODO(ack): add a guardian-scoped `care.log.acknowledge` verb
  // (keyed by a row id the list reply would need to expose) and call it here.
  function acknowledge(e: DailyLog) {
    setAcked((s) => new Set(s).add(entryKey(e)));
  }

  if (entries === null) return <FeedSkeleton />;

  return (
    <>
      <DaySummary summary={summary} />
      {!entries.length ? (
        <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">{t("feed.empty")}</p>
      ) : (
        <ul className="space-y-3 px-4">
          {entries.map((e) => {
            const k = entryKey(e);
            return (
              <li key={k}>
                <EntryRow
                  entry={e}
                  acknowledged={acked.has(k) || (e.incident?.acknowledged ?? false)}
                  onAcknowledge={e.kind === "incident" ? () => acknowledge(e) : undefined}
                />
              </li>
            );
          })}
        </ul>
      )}
    </>
  );
}

function FeedSkeleton() {
  return (
    <div className="space-y-3 px-4" aria-busy="true">
      {[0, 1, 2, 3].map((i) => (
        <div key={i} className="h-24 animate-pulse rounded-2xl bg-muted" />
      ))}
    </div>
  );
}
