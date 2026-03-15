use std::fmt;

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
pub enum NormalizedReference {
    Cell(CellRef),
    Area(AreaRef),
    Name(NameRef),
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
            Self::Name(name) => write!(f, "name:{}", name.name),
            Self::Error(error) => write!(f, "error:{}", error.error_class),
        }
    }
}
