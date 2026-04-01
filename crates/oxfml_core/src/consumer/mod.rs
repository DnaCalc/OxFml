//! Preferred consumer-facing OxFml entry surfaces.
//!
//! Ordinary downstream integrations should start here rather than assembling
//! runtime, editor, or replay behavior directly from historical substrate
//! modules.

use crate::interface::{
    LibraryContextProvider, LibraryContextSnapshotRef, PinnedLibraryContextView,
};
use crate::semantics::LibraryContextSnapshot;

/// Facade-first editor and language-service surface.
pub mod editor;
/// Facade-first replay projection surface.
pub mod replay;
/// Facade-first runtime execution surface.
pub mod runtime;

#[derive(Clone)]
pub(crate) struct ConsumerLibraryContextState<'a> {
    pub provider: Option<&'a dyn LibraryContextProvider>,
    pub snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub snapshot: Option<LibraryContextSnapshot>,
}

impl ConsumerLibraryContextState<'_> {
    pub fn new() -> Self {
        Self {
            provider: None,
            snapshot_ref: None,
            snapshot: None,
        }
    }

    pub fn from_parts<'a>(
        provider: Option<&'a dyn LibraryContextProvider>,
        snapshot_ref: Option<LibraryContextSnapshotRef>,
        snapshot: Option<LibraryContextSnapshot>,
    ) -> ConsumerLibraryContextState<'a> {
        ConsumerLibraryContextState {
            provider,
            snapshot_ref,
            snapshot,
        }
    }

    pub fn pinned_view(&self) -> PinnedLibraryContextView<'_> {
        PinnedLibraryContextView::new(
            self.provider,
            self.snapshot_ref.as_ref(),
            self.snapshot.as_ref(),
        )
    }
}

impl Default for ConsumerLibraryContextState<'_> {
    fn default() -> Self {
        Self::new()
    }
}
