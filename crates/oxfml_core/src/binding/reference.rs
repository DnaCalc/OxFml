use std::fmt;

use crate::binding::profile::ProfileReferenceRecord;
use crate::syntax::token::TextSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellCoord {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AddressMode {
    pub row_absolute: bool,
    pub col_absolute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRef {
    pub workbook_id: String,
    pub sheet_id: String,
    pub coord: CellCoord,
    pub address_mode: AddressMode,
    pub caller_anchor_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaRef {
    pub workbook_id: String,
    pub sheet_id: String,
    pub top_left: CellCoord,
    pub height: u32,
    pub width: u32,
    pub address_mode: AddressMode,
    pub caller_anchor_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WholeRowRef {
    pub workbook_id: String,
    pub sheet_id: String,
    pub row_start: u32,
    pub row_count: u32,
    pub address_mode: AddressMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WholeColumnRef {
    pub workbook_id: String,
    pub sheet_id: String,
    pub col_start: u32,
    pub col_count: u32,
    pub address_mode: AddressMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameKind {
    ReferenceLike,
    ValueLike,
    MixedOrDeferred,
    HelperLocal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameRef {
    pub name: String,
    pub workbook_id: String,
    pub sheet_id: String,
    pub kind: NameKind,
    pub caller_context_dependent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorRef {
    pub error_class: String,
    pub source_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRef {
    pub external_target_id: String,
    pub sheet_selector_summary: String,
    pub capability_requirement: String,
    pub external_reference_class: String,
    pub target_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredSectionKind {
    All,
    Data,
    Headers,
    Totals,
    ThisRow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredSelectorKind {
    Section,
    Column,
    ThisRowColumn,
    SectionColumn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuredResolvedRef {
    Cell(CellRef),
    Area(AreaRef),
    EmptyArea(StructuredEmptyAreaRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredEmptyAreaRef {
    pub workbook_id: String,
    pub sheet_id: String,
    pub section_kind: StructuredSectionKind,
    pub selected_column_ids: Vec<String>,
    pub column_count: u32,
    pub row_membership_identity: Option<String>,
    pub row_order_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredRef {
    pub table_id: String,
    pub table_name: String,
    pub selector_kind: StructuredSelectorKind,
    pub section_qualifiers: Vec<StructuredSectionKind>,
    pub selected_column_ids: Vec<String>,
    pub caller_row_sensitive: bool,
    pub workbook_scope_ref: String,
    pub sheet_scope_ref: String,
    pub resolved_reference: StructuredResolvedRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredReferenceSelectedRegion {
    pub section_kind: StructuredSectionKind,
    pub region_ref: Option<String>,
    pub column_range_refs: Vec<String>,
    pub is_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredReferenceBindDiagnosticLink {
    pub diagnostic_code: String,
    pub message: String,
    pub source_span_utf8: TextSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredReferenceSourceTokenKind {
    StructuredReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredReferenceBindRecord {
    pub bind_record_handle: String,
    pub source_span_utf8: TextSpan,
    pub source_token_text: String,
    pub source_token_kind: StructuredReferenceSourceTokenKind,
    pub explicit_table_name: Option<String>,
    pub omitted_table_name: bool,
    pub effective_table_id: Option<String>,
    pub effective_table_name: Option<String>,
    pub selected_column_ids: Vec<String>,
    pub selected_sections: Vec<StructuredSectionKind>,
    pub selected_regions: Vec<StructuredReferenceSelectedRegion>,
    pub uses_this_row: bool,
    pub caller_context_dependent: bool,
    pub resolved_reference: Option<StructuredResolvedRef>,
    pub diagnostics: Vec<StructuredReferenceBindDiagnosticLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizedReference {
    Cell(CellRef),
    Area(AreaRef),
    WholeRow(WholeRowRef),
    WholeColumn(WholeColumnRef),
    Name(NameRef),
    External(ExternalRef),
    Structured(StructuredRef),
    ProfileSymbolic(ProfileReferenceRecord),
    Error(ErrorRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceExpr {
    Atom(NormalizedReference),
    Range {
        start: Box<ReferenceExpr>,
        end: Box<ReferenceExpr>,
    },
    Union {
        left: Box<ReferenceExpr>,
        right: Box<ReferenceExpr>,
    },
    Intersection {
        left: Box<ReferenceExpr>,
        right: Box<ReferenceExpr>,
    },
    Spill {
        anchor: Box<ReferenceExpr>,
    },
}

impl fmt::Display for NormalizedReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cell(cell) => {
                write!(
                    f,
                    "{}!R{}C{}",
                    cell.sheet_id, cell.coord.row, cell.coord.col
                )
            }
            Self::Area(area) => write!(
                f,
                "{}!R{}C{}:{}x{}",
                area.sheet_id, area.top_left.row, area.top_left.col, area.height, area.width
            ),
            Self::WholeRow(rows) => write!(
                f,
                "{}!R{}:R{}",
                rows.sheet_id,
                rows.row_start,
                rows.row_start + rows.row_count - 1
            ),
            Self::WholeColumn(columns) => write!(
                f,
                "{}!C{}:C{}",
                columns.sheet_id,
                columns.col_start,
                columns.col_start + columns.col_count - 1
            ),
            Self::Name(name) => write!(f, "name:{}", name.name),
            Self::External(external) => write!(f, "external:{}", external.target_summary),
            Self::Structured(structured) => write!(
                f,
                "structured:{}:{}:{}",
                structured.table_id,
                match structured.selector_kind {
                    StructuredSelectorKind::Section => "Section",
                    StructuredSelectorKind::Column => "Column",
                    StructuredSelectorKind::ThisRowColumn => "ThisRowColumn",
                    StructuredSelectorKind::SectionColumn => "SectionColumn",
                },
                structured.selected_column_ids.join("|")
            ),
            Self::ProfileSymbolic(record) => write!(
                f,
                "profile-symbolic:{}:{}",
                record.profile_id, record.normal_form_key.0
            ),
            Self::Error(error) => write!(f, "error:{}", error.error_class),
        }
    }
}
