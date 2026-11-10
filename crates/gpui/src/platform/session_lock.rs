//! Locking the session with the `ext-session-lock-v1` Wayland protocol.
//!
//! A screen locker asks the compositor to lock the session with [`App::lock_session`], then
//! opens one [`WindowKind::SessionLock`] window per display (see [`App::displays`]), passing
//! the display as [`WindowOptions::display_id`]. The compositor sizes each lock surface to
//! cover its output, ignoring the requested bounds. While the session is locked only lock
//! surfaces are shown and receive input.
//!
//! The compositor answers the request with [`SessionLockEvent::Locked`] or
//! [`SessionLockEvent::Finished`]. Once authenticated, call [`App::unlock_session`], then
//! close the lock windows. If the locker exits or crashes without unlocking, the session
//! stays locked.
//!
//! [`App::lock_session`]: crate::App::lock_session
//! [`App::unlock_session`]: crate::App::unlock_session
//! [`App::displays`]: crate::App::displays
//! [`WindowKind::SessionLock`]: crate::WindowKind::SessionLock
//! [`WindowOptions::display_id`]: crate::WindowOptions::display_id

use thiserror::Error;

/// An event the compositor sends about a session lock requested with
/// [`App::lock_session`](crate::App::lock_session).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SessionLockEvent {
    /// The session is locked: every output shows a lock surface or is blanked, and no unlocked
    /// content is visible. Sent at most once per lock.
    Locked,
    /// The compositor has ended the lock. Either it refused to lock (for example because
    /// another client holds a lock), or it unlocked the session by other means. Lock windows
    /// are no longer shown and should be closed. Always the last event of a lock.
    Finished,
}

/// The compositor doesn't support `ext_session_lock_manager_v1`, or the platform has no
/// session lock.
#[derive(Debug, Error)]
#[error("Compositor doesn't support ext_session_lock_v1")]
pub struct SessionLockNotSupportedError;
