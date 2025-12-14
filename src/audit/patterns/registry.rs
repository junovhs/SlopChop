use super::PatternTemplate;

/// Built-in patterns to detect.
///
/// DESIGN PRINCIPLE: Each pattern must have a CLEAR CONSOLIDATION PATH
/// that results in FEWER TOTAL LINES. We don't flag idioms, we flag
/// opportunities.
///
/// Pattern quality checklist:
/// - [ ] Can I explain the refactor in one sentence?
/// - [ ] Will the refactored code be shorter?
/// - [ ] Is this a real code smell, not just "using the language"?
pub const PATTERNS: &[PatternTemplate] = &[
    // ==========================================================
    // STRUCT PATTERNS - Constructor/initialization duplication
    // ==========================================================
    PatternTemplate {
        name: "struct_literal_many_fields",
        description: "Struct literal with 5+ fields (consider Default + struct update syntax)",
        rust_query: r"
            (struct_expression
                body: (field_initializer_list
                    (field_initializer)
                    (field_initializer)
                    (field_initializer)
                    (field_initializer)
                    (field_initializer)))
        ",
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "option_field_none_init",
        description: "Struct with multiple None field inits (implement Default)",
        rust_query: r#"
            (field_initializer
                value: (identifier) @val
                (#eq? @val "None"))
        "#,
        python_query: None,
        min_occurrences: 10,
    },
    // ==========================================================
    // ERROR HANDLING - Repeated error construction
    // ==========================================================
    PatternTemplate {
        name: "anyhow_bail_format",
        description: "Repeated bail!/anyhow! with format strings (consider error enum)",
        rust_query: r#"
            (macro_invocation
                macro: (identifier) @mac
                (#match? @mac "^(bail|anyhow)$"))
        "#,
        python_query: None,
        min_occurrences: 8,
    },
    PatternTemplate {
        name: "err_return_format",
        description: "Repeated Err(format!(...)) returns (consider typed errors)",
        rust_query: r#"
            (return_expression
                (call_expression
                    function: (identifier) @err
                    (#eq? @err "Err")
                    arguments: (arguments
                        (macro_invocation
                            macro: (identifier) @fmt
                            (#eq? @fmt "format")))))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // MATCH PATTERNS - Duplicated branching logic
    // ==========================================================
    PatternTemplate {
        name: "match_with_many_arms",
        description: "Match with 6+ arms (consider lookup table or trait dispatch)",
        rust_query: r"
            (match_expression
                body: (match_block
                    (match_arm) (match_arm) (match_arm)
                    (match_arm) (match_arm) (match_arm)))
        ",
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "match_arm_same_body",
        description: "Match arms with trivial bodies (consider combining with |)",
        rust_query: r"
            (match_arm
                pattern: (or_pattern))
        ",
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "if_let_some_pattern",
        description: "Repeated if-let Some(x) pattern (consider ? or map)",
        rust_query: r#"
            (if_expression
                condition: (let_condition
                    pattern: (tuple_struct_pattern
                        type: (identifier) @typ
                        (#eq? @typ "Some"))))
        "#,
        python_query: None,
        min_occurrences: 8,
    },
    // ==========================================================
    // LOOP PATTERNS - Manual iteration vs functional
    // ==========================================================
    PatternTemplate {
        name: "for_loop_push",
        description: "For loop with push (likely .map().collect() or .filter().collect())",
        rust_query: r#"
            (for_expression
                body: (block
                    (expression_statement
                        (call_expression
                            function: (field_expression
                                field: (field_identifier) @method
                                (#eq? @method "push"))))))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "for_loop_insert",
        description: "For loop with insert (likely .collect::<HashMap>())",
        rust_query: r#"
            (for_expression
                body: (block
                    (expression_statement
                        (call_expression
                            function: (field_expression
                                field: (field_identifier) @method
                                (#eq? @method "insert"))))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "for_loop_counter",
        description: "For loop with manual index (consider .enumerate())",
        rust_query: r"
            (for_expression
                body: (block
                    (expression_statement
                        (assignment_expression
                            left: (identifier)
                            right: (binary_expression)))))
        ",
        python_query: None,
        min_occurrences: 3,
    },
    // ==========================================================
    // TEST PATTERNS - Test boilerplate
    // ==========================================================
    PatternTemplate {
        name: "test_assert_eq_pattern",
        description: "Repeated assert_eq! in tests (consider parameterized test helper)",
        rust_query: r#"
            (expression_statement
                (macro_invocation
                    macro: (identifier) @mac
                    (#eq? @mac "assert_eq")))
        "#,
        python_query: None,
        min_occurrences: 20,
    },
    PatternTemplate {
        name: "test_setup_let_binding",
        description: "Repeated let bindings at test start (consider test fixture)",
        rust_query: r#"
            (function_item
                (attribute_item
                    (attribute
                        (identifier) @attr
                        (#eq? @attr "test")))
                body: (block
                    (let_declaration)
                    (let_declaration)
                    (let_declaration)))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // STRING PATTERNS - Repeated string operations
    // ==========================================================
    PatternTemplate {
        name: "format_same_pattern",
        description: "format! with path/file patterns (consider path helper)",
        rust_query: r#"
            (macro_invocation
                macro: (identifier) @mac
                (#eq? @mac "format")
                (token_tree
                    (string_literal) @str
                    (#match? @str "\\{\\}/")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "to_string_call",
        description: "Repeated .to_string() calls (consider Into<String> or Cow)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "to_string")))
        "#,
        python_query: None,
        min_occurrences: 15,
    },
    // ==========================================================
    // CLONE PATTERNS - Unnecessary copying
    // ==========================================================
    PatternTemplate {
        name: "clone_in_loop",
        description: "clone() inside loop body (consider borrowing or Rc/Arc)",
        rust_query: r#"
            (for_expression
                body: (block
                    (_
                        (call_expression
                            function: (field_expression
                                field: (field_identifier) @method
                                (#eq? @method "clone"))))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "clone_method_arg",
        description: "clone() as method argument (consider taking reference)",
        rust_query: r#"
            (arguments
                (call_expression
                    function: (field_expression
                        field: (field_identifier) @method
                        (#eq? @method "clone"))))
        "#,
        python_query: None,
        min_occurrences: 10,
    },
    // ==========================================================
    // IMPL PATTERNS - Trait implementation boilerplate
    // ==========================================================
    PatternTemplate {
        name: "impl_from_manual",
        description: "Manual From impl (consider derive_more or thiserror #[from])",
        rust_query: r#"
            (impl_item
                trait: (generic_type
                    type: (type_identifier) @trait
                    (#eq? @trait "From"))
                body: (declaration_list
                    (function_item
                        name: (identifier) @fn
                        (#eq? @fn "from"))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "impl_display_write",
        description: "Display impl with write!/writeln! (consider derive_more Display)",
        rust_query: r#"
            (impl_item
                trait: (type_identifier) @trait
                (#eq? @trait "Display")
                body: (declaration_list
                    (function_item
                        body: (block
                            (expression_statement
                                (macro_invocation
                                    macro: (identifier) @mac
                                    (#match? @mac "^write")))))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    // ==========================================================
    // NESTING PATTERNS - Deep control flow
    // ==========================================================
    PatternTemplate {
        name: "nested_if_else",
        description: "Deeply nested if-else (consider early returns or match)",
        rust_query: r"
            (if_expression
                alternative: (else_clause
                    (if_expression
                        alternative: (else_clause
                            (if_expression)))))
        ",
        python_query: None,
        min_occurrences: 2,
    },
    PatternTemplate {
        name: "triple_nested_block",
        description: "Triple-nested blocks (refactor to extract functions)",
        rust_query: r"
            (block
                (expression_statement
                    (_
                        (block
                            (expression_statement
                                (_
                                    (block)))))))
        ",
        python_query: None,
        min_occurrences: 2,
    },
    // ==========================================================
    // CLOSURE PATTERNS - Anonymous function duplication
    // ==========================================================
    PatternTemplate {
        name: "closure_map_ok",
        description: "Closure in map with Ok wrapping (consider map_ok or ok())",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "map"))
                arguments: (arguments
                    (closure_expression
                        body: (call_expression
                            function: (identifier) @ok
                            (#eq? @ok "Ok")))))
        "#,
        python_query: None,
        min_occurrences: 3,
    },
    PatternTemplate {
        name: "closure_unwrap_or",
        description: "Repeated unwrap_or_else with closure (consider helper or Default)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "unwrap_or_else"))
                arguments: (arguments
                    (closure_expression)))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // METHOD CHAIN PATTERNS - Long chains that need helpers
    // ==========================================================
    PatternTemplate {
        name: "iter_filter_map_collect",
        description: "iter().filter().map().collect() chain (consider dedicated method)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    value: (call_expression
                        function: (field_expression
                            value: (call_expression
                                function: (field_expression
                                    field: (field_identifier) @f
                                    (#eq? @f "filter")))
                            field: (field_identifier) @m
                            (#eq? @m "map")))
                    field: (field_identifier) @c
                    (#eq? @c "collect")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    // ==========================================================
    // FILE I/O PATTERNS - Repeated file operations
    // ==========================================================
    PatternTemplate {
        name: "fs_read_to_string",
        description: "Repeated fs::read_to_string (consider file helper with context)",
        rust_query: r#"
            (call_expression
                function: (scoped_identifier
                    path: (identifier) @mod
                    name: (identifier) @fn
                    (#eq? @mod "fs")
                    (#eq? @fn "read_to_string")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "fs_write",
        description: "Repeated fs::write (consider file helper with context)",
        rust_query: r#"
            (call_expression
                function: (scoped_identifier
                    path: (identifier) @mod
                    name: (identifier) @fn
                    (#eq? @mod "fs")
                    (#eq? @fn "write")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
    PatternTemplate {
        name: "path_join_chain",
        description: "Chained path.join().join() (consider path builder)",
        rust_query: r#"
            (call_expression
                function: (field_expression
                    value: (call_expression
                        function: (field_expression
                            field: (field_identifier) @j1
                            (#eq? @j1 "join")))
                    field: (field_identifier) @j2
                    (#eq? @j2 "join")))
        "#,
        python_query: None,
        min_occurrences: 5,
    },
];