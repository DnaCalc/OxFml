use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormulaStableId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormulaTextVersion(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormulaToken(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructureContextVersion(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaSourceRecord {
    pub formula_stable_id: FormulaStableId,
    pub formula_text_version: FormulaTextVersion,
    pub entered_formula_text: String,
    pub stored_formula_text: Option<String>,
}

impl FormulaSourceRecord {
    pub fn new(
        formula_stable_id: impl Into<String>,
        formula_text_version: u64,
        entered_formula_text: impl Into<String>,
    ) -> Self {
        Self {
            formula_stable_id: FormulaStableId(formula_stable_id.into()),
            formula_text_version: FormulaTextVersion(formula_text_version),
            entered_formula_text: entered_formula_text.into(),
            stored_formula_text: None,
        }
    }

    pub fn formula_token(&self) -> FormulaToken {
        let mut hasher = DefaultHasher::new();
        self.formula_stable_id.hash(&mut hasher);
        self.formula_text_version.hash(&mut hasher);
        self.entered_formula_text.hash(&mut hasher);
        self.stored_formula_text.hash(&mut hasher);
        FormulaToken(format!("{:016x}", hasher.finish()))
    }
}
