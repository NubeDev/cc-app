import type { ChannelRecord } from "../../api/channels";

// Channel-id conventions (mirror of care's `messaging/channel_id.rs`):
//   care-child-<childId>   — a child's channel   (guardians + room staff, post)
//   care-room-<roomId>     — a room broadcast     (staff + room guardians, post)
//   care-center-<centerId> — center announcements (admin/staff post; guardians read)
// The prefixes are wire conventions, not user-facing prose.
const CHILD = "care-child-";
const ROOM = "care-room-";
const CENTER = "care-center-";

export type ChannelKind = "child" | "room" | "center" | "other";

export interface ChannelMeta {
  id: string;
  kind: ChannelKind;
  /** The domain id the channel maps to (child/room/center), or the raw id. */
  domainId: string;
}

/** Classify a channel by its conventional id + extract the domain id. */
export function channelMeta(id: string): ChannelMeta {
  if (id.startsWith(CHILD)) return { id, kind: "child", domainId: id.slice(CHILD.length) };
  if (id.startsWith(ROOM)) return { id, kind: "room", domainId: id.slice(ROOM.length) };
  if (id.startsWith(CENTER)) return { id, kind: "center", domainId: id.slice(CENTER.length) };
  return { id, kind: "other", domainId: id };
}

/** True for the read-only-for-guardians announcements channel. */
export function isAnnouncement(id: string): boolean {
  return id.startsWith(CENTER);
}

/** Resolve a channel's human title from domain-name maps, falling back to a
 *  translated kind label. `names` maps a domain id → its display name. */
export function channelTitle(
  meta: ChannelMeta,
  names: Record<string, string>,
  t: (k: string, vars?: Record<string, unknown>) => string,
): string {
  const name = names[meta.domainId];
  if (name) return name;
  return t(`messaging.kind.${meta.kind}`);
}

/** Stable display order: child channels, then rooms, then announcements. */
const ORDER: Record<ChannelKind, number> = { child: 0, room: 1, center: 2, other: 3 };

export function sortChannels(records: ChannelRecord[]): ChannelMeta[] {
  return records
    .map((r) => channelMeta(r.id))
    .sort((a, b) => ORDER[a.kind] - ORDER[b.kind] || a.id.localeCompare(b.id));
}
