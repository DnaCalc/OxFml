use std::collections::BTreeMap;

use oxfml_core::binding::{BindContext, BindRequest, NameKind, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::LibraryContextSnapshot;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{CompileSemanticPlanRequest, SemanticPlan, compile_semantic_plan};

#[allow(dead_code)]
pub struct CompiledFormulaArtifacts {
    pub bound_formula: oxfml_core::BoundFormula,
    pub semantic_plan: SemanticPlan,
}

pub fn compile_formula(
    formula_stable_id: &str,
    formula: &str,
    names: BTreeMap<String, NameKind>,
    structure_context_version: &str,
    oxfunc_catalog_identity: &str,
) -> CompiledFormulaArtifacts {
    compile_formula_with_library_context(
        formula_stable_id,
        formula,
        names,
        structure_context_version,
        oxfunc_catalog_identity,
        None,
    )
}

#[allow(dead_code)]
pub fn compile_formula_with_library_context(
    formula_stable_id: &str,
    formula: &str,
    names: BTreeMap<String, NameKind>,
    structure_context_version: &str,
    oxfunc_catalog_identity: &str,
    library_context_snapshot: Option<LibraryContextSnapshot>,
) -> CompiledFormulaArtifacts {
    let source = FormulaSourceRecord::new(formula_stable_id, 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion(
                structure_context_version.to_string(),
            ),
            names,
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });

    let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: oxfunc_catalog_identity.to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot,
    })
    .semantic_plan;

    CompiledFormulaArtifacts {
        bound_formula: bind.bound_formula,
        semantic_plan,
    }
}
