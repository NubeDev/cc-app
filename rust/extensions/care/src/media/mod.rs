//! care.media.* — the daily-feed PHOTO path (milestone 08; daily-feed-scope
//! §Photos, §"Photo consent", §Risks "the photo-URL leak").
//!
//! Verb-per-file (FILE-LAYOUT §one responsibility per file):
//!
//! - [`validate`] — the ONE place the v1 photos-only rule lives
//!   ([`reject_non_photo`]) + the typed [`MediaError`]. lb's core accepts video;
//!   cc-app rejects it at its own boundary.
//! - [`begin`] (`care.media.begin`, staff): the upload boundary — reject video,
//!   then delegate to lb's `media.upload_begin`. The honest reject seam (it is
//!   where the mime is known; `log::add` sees only committed ids).
//! - [`commit`] (`care.media.commit`, staff): finalize — delegate to lb's
//!   `media.upload_commit` (checksum verify + thumb derivation).
//! - [`serve_grant`] — the media-URL-leak defense: mint `store:media/{id}:read`
//!   to the child's feed recipients so ONLY reach-holders fetch the bytes. Wired
//!   into `log::add` after each photo row lands.
//!
//! ## Serve gate (verified from lb source)
//!
//! lb serves media bytes behind `store:media/{id}:read` (`host/src/media/serve.rs`).
//! A guardian holds no such cap by default, so a leaked media URL 403s unless
//! `serve_grant` mints the per-media read to the feed recipients. This is a real,
//! working grant (NOT an lb gap) — see [`serve_grant`]'s module doc for the
//! `grants.assign` shape + the no-widening reasoning.

pub mod begin;
pub mod commit;
pub mod serve_grant;
pub mod validate;

pub use serve_grant::{grant_media_read, revoke_media_read, MEDIA_SERVE_CAP_HELD};
pub use validate::{reject_non_photo, MediaError};
