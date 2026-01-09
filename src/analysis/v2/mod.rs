// src/analysis/v2/mod.rs
pub mod cognitive;
pub mod scope;
pub mod visitor;

use crate::config::Config;
use crate::lang::Lang;
use crate::types::{Violation, ViolationDetails};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::Parser;

pub struct ScanEngineV2 {
    #[allow(dead_code)]
    config: Config,
}

impl ScanEngineV2 {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the Scan v2 engine and returns violations mapped by file path.
    #[must_use]
    pub fn run(&self, files: &[PathBuf]) -> HashMap<PathBuf, Vec<Violation>> {
        let mut global_scopes = HashMap::new();
        let mut path_map = HashMap::new();

        for path in files {
            if let Some((scopes, p_str)) = Self::process_file(path) {
                for (name, scope) in scopes {
                    let key = format!("{p_str}::{name}");
                    global_scopes.insert(key, scope);
                    path_map.insert(p_str.clone(), path.clone());
                }
            }
        }

        Self::analyze_all_scopes(&global_scopes, &path_map)
    }

    fn process_file(path: &Path) -> Option<(HashMap<String, scope::Scope>, String)> {
        let source = std::fs::read_to_string(path).ok()?;
        let ext = path.extension()?.to_str()?;
        let lang = Lang::from_ext(ext)?;

        let mut parser = Parser::new();
        parser.set_language(lang.grammar()).ok()?;
        
        let tree = parser.parse(&source, None)?;
        let visitor = visitor::AstVisitor::new(&source, lang);
        let file_scopes = visitor.extract_scopes(tree.root_node());
        
        Some((file_scopes, path.to_string_lossy().to_string()))
    }

    fn analyze_all_scopes(
        scopes: &HashMap<String, scope::Scope>,
        path_map: &HashMap<String, PathBuf>
    ) -> HashMap<PathBuf, Vec<Violation>> {
        let mut results: HashMap<PathBuf, Vec<Violation>> = HashMap::new();

        for (full_name, scope) in scopes {
            let path_str = full_name.split("::").next().unwrap_or("");
            let Some(path) = path_map.get(path_str) else { continue; };
            
            let mut violations = Vec::new();
            Self::check_scope_cohesion(scope, &mut violations);
            Self::check_scope_coupling(scope, &mut violations);

            if !violations.is_empty() {
                results.entry(path.clone()).or_default().extend(violations);
            }
        }

        results
    }

    fn check_scope_cohesion(scope: &scope::Scope, out: &mut Vec<Violation>) {
        let lcom4 = scope.calculate_lcom4();
        if lcom4 > 1 {
            out.push(Violation::with_details(
                scope.row,
                format!("Class '{}' has low cohesion (LCOM4: {})", scope.name, lcom4),
                "DEEP ANALYSIS",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![format!("Connected components: {lcom4}")],
                    suggestion: Some("Consider splitting this class/struct into smaller units.".into()),
                }
            ));
        }
    }

    fn check_scope_coupling(scope: &scope::Scope, out: &mut Vec<Violation>) {
        let cbo = scope.calculate_cbo();
        if cbo > 9 {
            out.push(Violation::with_details(
                scope.row,
                format!("Class '{}' is tightly coupled (CBO: {})", scope.name, cbo),
                "DEEP ANALYSIS",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![format!("External dependencies: {cbo}")],
                    suggestion: Some("Reduce dependencies on external modules.".into()),
                }
            ));
        }

        let sfout = scope.calculate_max_sfout();
        if sfout > 7 {
            out.push(Violation::with_details(
                scope.row,
                format!("Class '{}' has high fan-out (Max SFOUT: {})", scope.name, sfout),
                "DEEP ANALYSIS",
                ViolationDetails {
                    function_name: Some(scope.name.clone()),
                    analysis: vec![format!("Max outgoing calls in one method: {sfout}")],
                    suggestion: Some("Delegate responsibilities to helper classes.".into()),
                }
            ));
        }
    }
}