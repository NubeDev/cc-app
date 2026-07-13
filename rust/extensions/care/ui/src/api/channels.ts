import { useMcpClient } from "@nube/ext-ui-sdk/runtime";

// The lb channel MCP surface — the ONLY reach an ext page has to channels.
//
// TRANSPORT NOTE (documented decision, mirrors FeedPage's SSE note):
// lb ships channels behind two surfaces — an HTTP/SSE gateway
// (`GET /channels`, `GET /channels/{cid}/stream?token=`, …) AND a generic MCP
// contract (`channel.list` / `channel.history` / `channel.post`, see lb
// `channels-scope.md` §"Host surface" + `channel/tool.rs`). The ext-ui-sdk
// runtime exposes ONLY `useMcpClient()` + `useSession()` — it surfaces neither
// the gateway origin nor the raw session token, so the UI cannot open the SSE
// `EventSource(stream + "&token=…")` today. We therefore reach channels through
// the MCP contract (which the care install's `bus:chan/care-**:{pub,sub}`
// grants authorize) and POLL `channel.history` for liveness, exactly as the
// m08 feed polls `log.list`. TODO(sse): once the runtime exposes the gateway
// origin + session token, swap the poll for an EventSource on the stream route.
//
// Every id/read/post is gated PER CHANNEL by lb (`bus:chan/{cid}:sub` to read,
// `:pub` to post). `channel.list` returns only channels the caller can `sub`,
// so a guardian never even learns a non-member channel exists (the messaging
// matrix invariant). `channel.post` FORCES the author to the caller's `sub`
// server-side — the UI never supplies it.

/** A registered channel row (lb `channel_registry::ChannelRecord`). */
export interface ChannelRecord {
  id: string;
  created_by?: string;
  kind?: string;
  ts?: number;
}

/** A durable channel message (lb `lb_inbox::Item`). `body` is user-authored
 *  content — NEVER translated (chrome only, per rule 8). */
export interface ChannelMessage {
  id: string;
  channel: string;
  author: string;
  body: string;
  ts: number;
}

export function useChannelsApi() {
  const call = useMcpClient();
  return {
    /** Channels the caller is a member of (lb gates by `bus:chan/*:sub`). */
    list: () =>
      call<{ channels: ChannelRecord[] }>("channel.list", {}).then((r) => r.channels ?? []),

    /** Durable history oldest-first for one channel (`bus:chan/{cid}:sub`). */
    history: (cid: string) =>
      call<{ messages: ChannelMessage[] }>("channel.history", { cid }).then(
        (r) => r.messages ?? [],
      ),

    /** Post a message (`bus:chan/{cid}:pub`). A read-only member (guardian on an
     *  announcements channel) 403s here at lb's gate — no care-side check. */
    post: (cid: string, id: string, body: string) =>
      call<ChannelMessage>("channel.post", { cid, id, body, ts: Date.now() }),
  };
}
