import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ChevronLeft, ChevronRight, Megaphone, MessagesSquare, Send, Users } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useCareApi } from "../../api/care";
import { useChannelsApi, type ChannelMessage } from "../../api/channels";
import { LargeTitle } from "../../components/LargeTitle";
import { Textarea } from "../../components/ui/textarea";
import { Button } from "../../components/ui/button";
import { useT } from "../../hooks/useT";
import { useCareSession } from "../../hooks/useCareSession";
import type { ChildRow, RoomRow } from "../child/ChildrenListPage";
import type { CenterRow } from "../admin/CentersRoomsPage";
import {
  channelTitle,
  isAnnouncement,
  sortChannels,
  type ChannelKind,
  type ChannelMeta,
} from "./channel-meta";

// How often the open thread re-polls `channel.history` for new messages. lb's
// SSE stream is unreachable from ext UI today (no gateway origin / token in the
// runtime — see api/channels.ts), so we poll, mirroring the m08 feed's idiom.
const POLL_MS = 15000;

const KIND_ICON: Record<ChannelKind, LucideIcon> = {
  child: MessagesSquare,
  room: Users,
  center: Megaphone,
  other: MessagesSquare,
};

// Guardian + staff Messages: a list of the channels the caller is a member of
// (lb `channel.list` returns only reachable channels — a guardian never learns
// a non-member channel exists), and a thread view per channel. Announcements
// (`care-center-*`) are read-only for guardians (no composer); child/room
// channels take a composer for every member.
export function MessagesPage() {
  const t = useT();
  const session = useCareSession();
  const channels = useChannelsApi();
  const care = useCareApi();
  const role = session?.role ?? "guardian";
  const isGuardian = role === "guardian";

  const [metas, setMetas] = useState<ChannelMeta[] | null>(null);
  const [names, setNames] = useState<Record<string, string>>({});
  const [openId, setOpenId] = useState<string | null>(null);

  useEffect(() => {
    channels
      .list()
      .then((rows) => setMetas(sortChannels(rows)))
      .catch(() => setMetas([]));
    // Resolve channel titles from the domain records the caller can already see
    // (each list is chokepoint-scoped server-side). Best-effort — a missing map
    // just falls back to the translated kind label.
    Promise.allSettled([
      care.list<ChildRow>("child"),
      care.list<RoomRow>("room"),
      care.list<CenterRow>("center"),
    ]).then((res) => {
      const map: Record<string, string> = {};
      for (const r of res) {
        if (r.status === "fulfilled") {
          for (const row of r.value as Array<{ id: string; name: string }>) map[row.id] = row.name;
        }
      }
      setNames(map);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const open = useMemo(
    () => metas?.find((m) => m.id === openId) ?? null,
    [metas, openId],
  );

  if (open) {
    return (
      <Thread
        meta={open}
        title={channelTitle(open, names, t)}
        readOnly={isGuardian && isAnnouncement(open.id)}
        onBack={() => setOpenId(null)}
      />
    );
  }

  return (
    <main className="pb-24">
      <LargeTitle>{t("nav.messages")}</LargeTitle>
      {metas === null ? (
        <ListSkeleton />
      ) : !metas.length ? (
        <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">
          {t("messaging.no_channels")}
        </p>
      ) : (
        <ul className="space-y-2 px-4">
          {metas.map((m) => {
            const Icon = KIND_ICON[m.kind];
            return (
              <li key={m.id}>
                <button
                  type="button"
                  onClick={() => setOpenId(m.id)}
                  className="flex w-full items-center gap-3 rounded-2xl border border-border bg-card p-3.5 text-left transition-colors hover:bg-accent active:scale-[0.99] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-accent text-primary">
                    <Icon className="size-5" aria-hidden />
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-[15px] font-semibold text-foreground">
                      {channelTitle(m, names, t)}
                    </span>
                    <span className="block truncate text-[13px] text-muted-foreground">
                      {t(`messaging.kind.${m.kind}`)}
                    </span>
                  </span>
                  <ChevronRight className="size-5 shrink-0 text-muted-foreground" aria-hidden />
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </main>
  );
}

// One channel's thread: history (oldest-first) + poll for liveness + composer
// (unless read-only). Keyed remount per channel keeps the poll clean.
function Thread({
  meta,
  title,
  readOnly,
  onBack,
}: {
  meta: ChannelMeta;
  title: string;
  readOnly: boolean;
  onBack: () => void;
}) {
  const t = useT();
  const session = useCareSession();
  const channels = useChannelsApi();
  const [messages, setMessages] = useState<ChannelMessage[] | null>(null);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);
  const bottom = useRef<HTMLDivElement>(null);

  const poll = useCallback(async () => {
    try {
      const msgs = await channels.history(meta.id);
      if (mounted.current) setMessages(msgs);
    } catch {
      if (mounted.current) setMessages((prev) => prev ?? []);
    }
  }, [channels, meta.id]);

  useEffect(() => {
    mounted.current = true;
    poll();
    const timer = setInterval(poll, POLL_MS);
    return () => {
      mounted.current = false;
      clearInterval(timer);
    };
  }, [poll]);

  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  async function send() {
    const body = draft.trim();
    if (!body || sending) return;
    setSending(true);
    setError(null);
    try {
      await channels.post(meta.id, crypto.randomUUID(), body);
      setDraft("");
      await poll();
    } catch {
      setError(t("messaging.send_failed"));
    } finally {
      if (mounted.current) setSending(false);
    }
  }

  const mySub = session?.sub;

  return (
    <main className="flex min-h-[100dvh] flex-col pb-24">
      <header className="sticky top-0 z-10 flex items-center gap-1 border-b border-border/70 bg-background/80 px-2 py-2 backdrop-blur-xl">
        <button
          type="button"
          onClick={onBack}
          className="flex items-center gap-0.5 rounded-lg px-1.5 py-1 text-[15px] font-medium text-primary transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <ChevronLeft className="size-5" aria-hidden />
          {t("common.back")}
        </button>
        <h1 className="min-w-0 flex-1 truncate text-center text-[17px] font-semibold text-foreground">
          {title}
        </h1>
        <span className="w-16 shrink-0" aria-hidden />
      </header>

      <div className="flex-1 space-y-3 px-4 py-4">
        {messages === null ? (
          <ThreadSkeleton />
        ) : !messages.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">
            {t("messaging.thread_empty")}
          </p>
        ) : (
          messages.map((m) => <Bubble key={m.id} message={m} mine={m.author === mySub} />)
        )}
        <div ref={bottom} />
      </div>

      {readOnly ? (
        <p className="fixed inset-x-0 bottom-[calc(52px+env(safe-area-inset-bottom))] z-10 border-t border-border/70 bg-background/80 px-4 py-3 text-center text-[13px] text-muted-foreground backdrop-blur-xl">
          {t("messaging.read_only")}
        </p>
      ) : (
        <div className="fixed inset-x-0 bottom-[calc(52px+env(safe-area-inset-bottom))] z-10 border-t border-border/70 bg-background/80 px-3 py-2 backdrop-blur-xl">
          <div className="mx-auto flex max-w-2xl items-end gap-2">
            <Textarea
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder={t("messaging.composer_placeholder")}
              aria-label={t("messaging.composer_placeholder")}
              rows={1}
              className="max-h-32 min-h-[44px] flex-1 resize-none py-2.5"
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  void send();
                }
              }}
            />
            <Button
              type="button"
              size="icon"
              onClick={() => void send()}
              disabled={!draft.trim() || sending}
              aria-label={t("messaging.send")}
              className="size-11 shrink-0 rounded-full"
            >
              <Send className="size-5" aria-hidden />
            </Button>
          </div>
          {error && (
            <p className="mx-auto max-w-2xl px-1 pt-1 text-[12px] text-destructive">{error}</p>
          )}
        </div>
      )}
    </main>
  );
}

// A single message bubble. Content (`body`) is user-authored — never translated.
function Bubble({ message, mine }: { message: ChannelMessage; mine: boolean }) {
  return (
    <div className={mine ? "flex justify-end" : "flex justify-start"}>
      <div
        className={
          "max-w-[80%] rounded-2xl px-3.5 py-2 text-[15px] leading-snug " +
          (mine
            ? "bg-primary text-primary-foreground"
            : "border border-border bg-card text-foreground")
        }
      >
        {!mine && (
          <span className="mb-0.5 block truncate text-[12px] font-semibold text-muted-foreground">
            {message.author}
          </span>
        )}
        <span className="whitespace-pre-wrap break-words">{message.body}</span>
      </div>
    </div>
  );
}

function ListSkeleton() {
  return (
    <div className="space-y-2 px-4" aria-busy="true">
      {[0, 1, 2].map((i) => (
        <div key={i} className="h-16 animate-pulse rounded-2xl bg-muted" />
      ))}
    </div>
  );
}

function ThreadSkeleton() {
  return (
    <div className="space-y-3" aria-busy="true">
      {[0, 1, 2].map((i) => (
        <div
          key={i}
          className={
            "h-12 w-2/3 animate-pulse rounded-2xl bg-muted " + (i % 2 ? "ml-auto" : "")
          }
        />
      ))}
    </div>
  );
}
