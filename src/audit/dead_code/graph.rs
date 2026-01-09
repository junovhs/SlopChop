use super::Symbol;
use std::collections::{HashMap, HashSet};

/// Call graph for reachability analysis.
pub struct CallGraph {
    symbols: HashSet<Symbol>,
    calls: HashMap<Symbol, HashSet<Symbol>>,
    called_by: HashMap<Symbol, HashSet<Symbol>>,
    entry_points: HashSet<Symbol>,
    public_symbols: HashSet<Symbol>,
}

impl CallGraph {
    /// Creates a new empty call graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            symbols: HashSet::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            entry_points: HashSet::new(),
            public_symbols: HashSet::new(),
        }
    }

    /// Adds a symbol definition.
    pub fn add_symbol(&mut self, symbol: Symbol, is_public: bool, is_entry: bool) {
        self.symbols.insert(symbol.clone());

        if is_public {
            self.public_symbols.insert(symbol.clone());
        }

        if is_entry {
            self.entry_points.insert(symbol);
        }
    }

    /// Adds an edge: `from` calls/references `to`.
    pub fn add_edge(&mut self, from: Symbol, to: Symbol) {
        self.calls
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.called_by.entry(to).or_default().insert(from);
    }

    /// Computes reachable symbols from entry points.
    #[must_use]
    pub fn compute_reachable(&self) -> HashSet<Symbol> {
        let mut reachable = HashSet::new();
        let mut worklist: Vec<Symbol> = self.entry_points.iter().cloned().collect();

        worklist.extend(self.public_symbols.iter().cloned());

        while let Some(sym) = worklist.pop() {
            if reachable.contains(&sym) {
                continue;
            }

            reachable.insert(sym.clone());
            self.process_callees(&sym, &reachable, &mut worklist);
        }

        reachable
    }

    fn process_callees(
        &self,
        sym: &Symbol,
        reachable: &HashSet<Symbol>,
        worklist: &mut Vec<Symbol>,
    ) {
        if let Some(callees) = self.calls.get(sym) {
            for callee in callees {
                if !reachable.contains(callee) {
                    worklist.push(callee.clone());
                }
            }
        }
    }

    /// Returns the number of symbols in the graph.
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Returns the number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.calls.values().map(HashSet::len).sum()
    }
    
    /// Accessor for symbols (for analysis).
    #[must_use]
    pub fn symbols(&self) -> &HashSet<Symbol> {
        &self.symbols
    }

    /// Accessor for called_by (for analysis).
    #[must_use]
    pub fn called_by(&self) -> &HashMap<Symbol, HashSet<Symbol>> {
        &self.called_by
    }

    /// Accessor for calls (for analysis).
    #[must_use]
    pub fn calls(&self) -> &HashMap<Symbol, HashSet<Symbol>> {
        &self.calls
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}
