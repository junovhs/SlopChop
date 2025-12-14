// tests/unit_graph.rs
//! Tests for import extraction across languages.

use slopchop_core::graph::imports;
use std::path::Path;

fn check_imports(filename: &str, content: &str, expected: &str) {
    let imports = imports::extract(Path::new(filename), content);
    assert!(
        !imports.is_empty(),
        "Should extract imports from {filename}"
    );
    assert!(
        imports.iter().any(|i| i.contains(expected)),
        "Should find '{expected}' import in {filename}"
    );
}

#[test]
fn test_rust_use_extraction() {
    let content = r"
use std::collections::HashMap;
use crate::config::Config;
use super::types::Violation;
";
    check_imports("src/lib.rs", content, "HashMap");
}

#[test]
fn test_rust_mod_extraction() {
    let content = r"
mod config;
mod analysis;
pub mod types;
";
    check_imports("src/lib.rs", content, "config");
}

#[test]
fn test_python_import() {
    let content = r"
import os
import sys
import json
";
    check_imports("main.py", content, "os");
}

#[test]
fn test_python_from_import() {
    let content = r"
from pathlib import Path
from typing import Optional, List
from .utils import helper
";
    check_imports("main.py", content, "pathlib");
}

#[test]
fn test_ts_import() {
    let content = r"
import { useState } from 'react';
import axios from 'axios';
import * as utils from './utils';
";
    check_imports("app.ts", content, "react");
}