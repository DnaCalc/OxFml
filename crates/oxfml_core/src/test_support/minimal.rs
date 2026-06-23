//! Minimal, deliberately non-grid reference profile + provider for OxFml's tests.
//!
//! OxFml core is reference-meaning agnostic: it owns the binding lifecycle and
//! the profile seam, but it does not know what a reference means. Real grid
//! meaning lives downstream in OxCalc's `strict-excel-grid` profile (see
//! `CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md` §8). OxFml's own test suite,
//! however, needs *some* reference language to exercise the abstract plumbing.
//!
//! This module is that minimal language — an ordinary downstream profile so the
//! core crate carries zero built-in grammar in production builds. It is compiled
//! only under `cfg(test)` or the `test-support` feature. It is NOT a grid
//! provider: A1-style tokens are used only as a convenient opaque key, with no
//! R1C1, sheet-qualified, whole-axis, `$`-anchor, or spill semantics. Anything
//! grid-shaped belongs in OxCalc, tested against the real provider.
//!
//! - [`MinimalReferenceProfile`] parses bare same-sheet A1 cell/range tokens at
//!   bind time and emits opaque profile-symbolic records keyed by canonical text.
//! - [`MinimalReferenceSystemProvider`] resolves those records at eval time
//!   against an in-memory key -> `CalcValue` store.

use std::collections::BTreeMap;

use crate::binding::{
    ProfilePayload, ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceDependencyEnvelope,
    ReferenceNormalFormKey, ReferenceProfileFingerprintContext,
    ReferenceRangeBindRequest, ReferenceRangeBindResult, ReferenceSourceInfo, ReferenceValidity,
};
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemProvider,
};
use oxfunc_core::value::{
    ArrayShape, CalcArray, CalcValue, ReferenceIdentity, ReferenceLike,
};

/// Profile id for the minimal test-only reference language.
pub const MINIMAL_REFERENCE_PROFILE_ID: &str = "oxfml.test.minimal.v1";

/// Minimal A1 reference profile used by OxFml's own test suite.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalReferenceProfile;

/// Singleton instance, convenient for `with_reference_bind_profile(&MINIMAL_REFERENCE_PROFILE)`.
pub static MINIMAL_REFERENCE_PROFILE: MinimalReferenceProfile = MinimalReferenceProfile;

impl MinimalReferenceProfile {
    fn record(
        &self,
        key: String,
        request_source_text: String,
        source_info: ReferenceSourceInfo,
    ) -> ProfileReferenceRecord {
        ProfileReferenceRecord {
            profile_id: MINIMAL_REFERENCE_PROFILE_ID.to_string(),
            profile_version: ProfileVersion::v1(),
            source_info,
            profile_payload: ProfilePayload::textual("oxfml-test-minimal-reference", key.clone()),
            normal_form_key: ReferenceNormalFormKey(key),
            render_hint: Some(request_source_text),
            validity: ReferenceValidity::ValidNow,
        }
    }
}

impl ReferenceBindProfile for MinimalReferenceProfile {
    fn profile_id(&self) -> &str {
        MINIMAL_REFERENCE_PROFILE_ID
    }

    // Deliberately keeps the default `LegacyCompatibility` policy rather than
    // `ProfileSymbolic`: this profile binds bare A1 tokens via `bind_atom` but
    // lets unbound identifiers (e.g. defined names in `BindContext::names`) fall
    // through to the binder's legacy name path, which the OxFml test suite relies
    // on. Real grid profiles (OxCalc) use `ProfileSymbolic` so they fully own
    // reference + name binding.

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        let Some(cell) = parse_cell(&request.source_text) else {
            // Not an A1 cell: let the binder try the next binder (names, etc.).
            return ReferenceAtomBindResult::LegacyCompatibility;
        };
        let source_info = ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
            address_fidelity: None,
        };
        ReferenceAtomBindResult::Bound(self.record(
            cell.key(),
            request.source_text.clone(),
            source_info,
        ))
    }

    fn bind_range(&self, request: &ReferenceRangeBindRequest) -> ReferenceRangeBindResult {
        let (Some(start), Some(end)) = (
            parse_cell(&request.left.target_text),
            parse_cell(&request.right.target_text),
        ) else {
            return ReferenceRangeBindResult::LegacyCompatibility;
        };
        let key = range_key(start, end);
        let source_info = ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: None,
            address_fidelity: None,
        };
        ReferenceRangeBindResult::Bound(self.record(
            key,
            request.source_text.clone(),
            source_info,
        ))
    }

    fn dependency_hints(
        &self,
        reference: &ProfileReferenceRecord,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceDependencyEnvelope {
        ReferenceDependencyEnvelope::Static {
            profile_id: MINIMAL_REFERENCE_PROFILE_ID.to_string(),
            dependency_key: reference.normal_form_key.0.clone(),
        }
    }
}

/// Test-only provider that resolves [`MinimalReferenceProfile`] references
/// against an in-memory `A1 -> CalcValue` store.
#[derive(Debug, Clone, Default)]
pub struct MinimalReferenceSystemProvider {
    cells: BTreeMap<String, CalcValue>,
}

impl MinimalReferenceSystemProvider {
    /// Build a provider from canonical-A1-keyed cell values (e.g. `"A1" -> 5`).
    #[must_use]
    pub fn new(cells: BTreeMap<String, CalcValue>) -> Self {
        Self { cells }
    }

    fn cell_value(&self, key: &str) -> CalcValue {
        self.cells.get(key).cloned().unwrap_or_else(CalcValue::empty)
    }

    fn area_value(
        &self,
        start: GridCell,
        end: GridCell,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let rows = (end.row - start.row + 1) as usize;
        let cols = (end.col - start.col + 1) as usize;
        let mut cells = Vec::with_capacity(rows * cols);
        for row in start.row..=end.row {
            for col in start.col..=end.col {
                cells.push(self.cell_value(&GridCell { row, col }.key()));
            }
        }
        let shape = ArrayShape { rows, cols };
        CalcArray::new(shape, cells).map(CalcValue::array).ok_or(
            ReferenceResolutionError::ProviderFailure {
                detail: "oxfml_test_minimal_area_shape_invalid".to_string(),
            },
        )
    }
}

impl ReferenceSystemProvider for MinimalReferenceSystemProvider {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let key = reference_key(&request.reference).ok_or_else(|| {
            ReferenceResolutionError::UnresolvedReference {
                target: request.reference.target().to_string(),
            }
        })?;
        if let Some((start, end)) = parse_range_key(&key) {
            return self.area_value(start, end);
        }
        if parse_cell(&key).is_some() {
            return Ok(self.cell_value(&key));
        }
        Err(ReferenceResolutionError::UnresolvedReference { target: key })
    }
}

/// A 1-based grid cell coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridCell {
    row: u32,
    col: u32,
}

impl GridCell {
    /// Canonical A1 text, e.g. `(1, 1) -> "A1"`.
    fn key(self) -> String {
        format!("{}{}", column_letters(self.col), self.row)
    }
}

/// Recover the profile key from a reference: opaque handle bytes (the normal
/// form key) or the textual target.
fn reference_key(reference: &ReferenceLike) -> Option<String> {
    match &reference.identity {
        ReferenceIdentity::Opaque(handle) => String::from_utf8(handle.id.bytes.clone()).ok(),
        ReferenceIdentity::Textual(_) => Some(reference.target().to_string()),
        ReferenceIdentity::Composite(_) => None,
    }
}

fn parse_range_key(key: &str) -> Option<(GridCell, GridCell)> {
    let (left, right) = key.split_once(':')?;
    Some((parse_cell(left)?, parse_cell(right)?))
}

/// Parse a single A1 cell token (`A1`, `$A$1`, `Sheet!B2`) into a coordinate.
fn parse_cell(token: &str) -> Option<GridCell> {
    let token = token.rsplit('!').next().unwrap_or(token);
    let token: String = token.chars().filter(|ch| *ch != '$').collect();
    let bytes = token.as_bytes();
    let mut split = 0;
    while split < bytes.len() && bytes[split].is_ascii_alphabetic() {
        split += 1;
    }
    if split == 0 || split == bytes.len() {
        return None;
    }
    let col = column_from_letters(&token[..split])?;
    let row: u32 = token[split..].parse().ok()?;
    if row == 0 {
        return None;
    }
    Some(GridCell { row, col })
}

fn range_key(a: GridCell, b: GridCell) -> String {
    let start = GridCell {
        row: a.row.min(b.row),
        col: a.col.min(b.col),
    };
    let end = GridCell {
        row: a.row.max(b.row),
        col: a.col.max(b.col),
    };
    format!("{}:{}", start.key(), end.key())
}

fn column_from_letters(letters: &str) -> Option<u32> {
    let mut col: u32 = 0;
    for ch in letters.chars() {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            return None;
        }
        col = col.checked_mul(26)?.checked_add(u32::from(upper as u8 - b'A') + 1)?;
    }
    (col > 0).then_some(col)
}

fn column_letters(mut col: u32) -> String {
    let mut letters = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        letters.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    letters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FormulaChannelKind;
    use crate::syntax::token::TextSpan;
    use oxfunc_core::value::{ReferenceHandle, ReferenceHandleId, ReferenceSystemId};

    fn atom_request(text: &str) -> ReferenceAtomBindRequest {
        ReferenceAtomBindRequest {
            source_channel: FormulaChannelKind::WorksheetA1,
            source_span: TextSpan::new(0, text.len()),
            source_text: text.to_string(),
            parsed_qualifier: None,
            workbook_id: "book".to_string(),
            sheet_id: "sheet".to_string(),
            caller_row: 1,
            caller_col: 1,
        }
    }

    fn opaque_ref(key: &str) -> ReferenceLike {
        ReferenceLike::opaque(
            ReferenceSystemId(MINIMAL_REFERENCE_PROFILE_ID.to_string()),
            ReferenceHandle {
                id: ReferenceHandleId::from_bytes(key.as_bytes().to_vec()),
            },
            None,
        )
    }

    #[test]
    fn column_round_trips() {
        for col in [1u32, 26, 27, 52, 16384] {
            assert_eq!(column_from_letters(&column_letters(col)), Some(col));
        }
    }

    #[test]
    fn parses_cells_with_anchors_and_qualifiers() {
        assert_eq!(parse_cell("A1"), Some(GridCell { row: 1, col: 1 }));
        assert_eq!(parse_cell("$B$2"), Some(GridCell { row: 2, col: 2 }));
        assert_eq!(parse_cell("Sheet1!AA10"), Some(GridCell { row: 10, col: 27 }));
        assert_eq!(parse_cell("SUM"), None);
        assert_eq!(parse_cell("A0"), None);
    }

    #[test]
    fn bind_atom_emits_canonical_symbolic_record() {
        let profile = MinimalReferenceProfile;
        let result = profile.bind_atom(&atom_request("$a$1"));
        match result {
            ReferenceAtomBindResult::Bound(record) => {
                assert_eq!(record.profile_id, MINIMAL_REFERENCE_PROFILE_ID);
                assert_eq!(record.normal_form_key.0, "A1");
            }
            other => panic!("expected Bound, got {other:?}"),
        }
        assert!(matches!(
            profile.bind_atom(&atom_request("NotACell")),
            ReferenceAtomBindResult::LegacyCompatibility
        ));
    }

    #[test]
    fn provider_resolves_single_cell_and_area() {
        let mut cells = BTreeMap::new();
        cells.insert("A1".to_string(), CalcValue::number(2.0));
        cells.insert("B1".to_string(), CalcValue::number(3.0));
        let provider = MinimalReferenceSystemProvider::new(cells);

        let cell = provider
            .dereference(&ReferenceDereferenceRequest {
                reference: opaque_ref("A1"),
            })
            .expect("cell resolves");
        assert_eq!(cell, CalcValue::number(2.0));

        let area = provider
            .dereference(&ReferenceDereferenceRequest {
                reference: opaque_ref("A1:B1"),
            })
            .expect("area resolves");
        assert!(matches!(area.core(), oxfunc_core::value::CoreValue::Array(_)));

        let empty = provider
            .dereference(&ReferenceDereferenceRequest {
                reference: opaque_ref("Z9"),
            })
            .expect("missing cell resolves to empty");
        assert_eq!(empty, CalcValue::empty());
    }
}
