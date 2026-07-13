import { useEffect, useMemo, useState } from "react";
import { Megaphone, Send } from "lucide-react";
import { useCareApi } from "../../api/care";
import { useChannelsApi, type ChannelMessage } from "../../api/channels";
import { LargeTitle } from "../../components/LargeTitle";
import { Textarea } from "../../components/ui/textarea";
import { Button } from "../../components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { useT } from "../../hooks/useT";
import type { CenterRow } from "../admin/CentersRoomsPage";

// Admin announcements: compose to `care-center-<centerId>` + see recent history.
// Multi-center orgs get one announcements channel PER center (messaging-scope
// open question resolved: per-center), so the admin picks the center to address.
// Posting is gated by lb (`bus:chan/{cid}:pub`) — an admin/staff holds it,
// guardians hold `sub` only and read these in their Messages tab.
export function AnnouncementsCompose() {
  const t = useT();
  const care = useCareApi();
  const channels = useChannelsApi();

  const [centers, setCenters] = useState<CenterRow[] | null>(null);
  const [centerId, setCenterId] = useState<string>("");
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [history, setHistory] = useState<ChannelMessage[]>([]);

  const cid = useMemo(() => (centerId ? "care-center-" + centerId : ""), [centerId]);

  useEffect(() => {
    care
      .list<CenterRow>("center")
      .then((c) => {
        const active = c.filter((x) => !x.archived);
        setCenters(active);
        if (active[0]) setCenterId(active[0].id);
      })
      .catch(() => setCenters([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!cid) return;
    setHistory([]);
    channels
      .history(cid)
      .then(setHistory)
      .catch(() => setHistory([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cid]);

  function flash(msg: string) {
    setToast(msg);
    setTimeout(() => setToast((cur) => (cur === msg ? null : cur)), 2500);
  }

  async function post() {
    const body = draft.trim();
    if (!body || !cid || sending) return;
    setSending(true);
    setError(null);
    try {
      await channels.post(cid, crypto.randomUUID(), body);
      setDraft("");
      flash(t("messaging.announcement_sent"));
      const msgs = await channels.history(cid).catch(() => history);
      setHistory(msgs);
    } catch {
      setError(t("messaging.send_failed"));
    } finally {
      setSending(false);
    }
  }

  return (
    <main className="pb-24">
      <LargeTitle>{t("messaging.announcements.title")}</LargeTitle>

      <div className="space-y-3 px-4">
        {centers === null ? (
          <div className="h-11 animate-pulse rounded-xl bg-muted" />
        ) : !centers.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">
            {t("center.empty")}
          </p>
        ) : (
          <>
            {centers.length > 1 && (
              <Select value={centerId} onValueChange={setCenterId}>
                <SelectTrigger className="h-12 text-base">
                  <SelectValue placeholder={t("messaging.pick_center")} />
                </SelectTrigger>
                <SelectContent>
                  {centers.map((c) => (
                    <SelectItem key={c.id} value={c.id}>
                      {c.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}

            <div className="rounded-2xl border border-border bg-card p-4">
              <div className="mb-2 flex items-center gap-2 text-[13px] font-medium text-muted-foreground">
                <Megaphone className="size-4" aria-hidden />
                {t("messaging.announcements.compose_hint")}
              </div>
              <Textarea
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                placeholder={t("messaging.announcements.placeholder")}
                aria-label={t("messaging.announcements.placeholder")}
                rows={4}
              />
              <div className="mt-3 flex justify-end">
                <Button
                  type="button"
                  onClick={() => void post()}
                  disabled={!draft.trim() || !cid || sending}
                >
                  <Send className="size-4" aria-hidden />
                  {t("messaging.announcements.send")}
                </Button>
              </div>
              {error && <p className="pt-1 text-[12px] text-destructive">{error}</p>}
            </div>

            <section>
              <h2 className="px-1 pb-2 pt-2 text-[13px] font-semibold text-muted-foreground">
                {t("messaging.announcements.recent")}
              </h2>
              {!history.length ? (
                <p className="py-8 text-center text-[15px] text-muted-foreground">
                  {t("messaging.thread_empty")}
                </p>
              ) : (
                <ul className="space-y-2">
                  {history
                    .slice()
                    .reverse()
                    .map((m) => (
                      <li
                        key={m.id}
                        className="rounded-2xl border border-border bg-card px-3.5 py-2.5 text-[15px] leading-snug text-foreground"
                      >
                        <span className="whitespace-pre-wrap break-words">{m.body}</span>
                      </li>
                    ))}
                </ul>
              )}
            </section>
          </>
        )}
      </div>

      {toast && (
        <div className="pointer-events-none fixed inset-x-0 bottom-24 z-40 flex justify-center px-4">
          <div className="rounded-full bg-success px-5 py-2.5 text-sm font-semibold text-success-foreground shadow-lg">
            {toast}
          </div>
        </div>
      )}
    </main>
  );
}
