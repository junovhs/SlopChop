use std::path::Path;
use warden_core::skeleton;

#[test]
fn test_clean_rust_basic() {
    let code = r"
fn main() {
    println!();
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("fn main() { ... }"));
}

#[test]
fn test_clean_rust_nested() {
    let code = r"
fn outer() {
    fn inner() {
        // inner
    }
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("fn outer() { ... }"));
    assert!(!result.contains("fn inner"));
}

#[test]
fn test_clean_rust_impl() {
    let code = r"
impl MyStruct {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    fn helper(&self) {
        // help
    }
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("pub fn new() -> Self { ... }"));
    assert!(result.contains("fn helper(&self) { ... }"));
}

#[test]
fn test_clean_python() {
    let code = r"
def calculate(x, y):
    result = x + y
    for i in range(10):
        result += i
    return result

class MyClass:
    def method(self):
        return self.value
";
    let result = skeleton::clean(Path::new("test.py"), code);
    assert!(result.contains("def calculate(x, y): ..."));
    assert!(result.contains("def method(self): ..."));
}

#[test]
fn test_clean_typescript() {
    let code = r"
function processData(data: string): number {
    const parsed = JSON.parse(data);
    return parsed.value * 2;
}

class Processor {
    process() {
        // logic
    }
}
";
    let result = skeleton::clean(Path::new("test.ts"), code);
    assert!(result.contains("function processData(data: string): number { ... }"));
    assert!(result.contains("process() { ... }"));
}

#[test]
fn test_clean_arrow_functions() {
    let code = r"
const add = (a: number, b: number): number => {
    const sum = a + b;
    console.log(sum);
    return sum;
};

const multiply = (a: number, b: number) => a * b;
";
    let result = skeleton::clean(Path::new("test.ts"), code);
    assert!(result.contains("const add = (a: number, b: number): number => { ... }"));
}

#[test]
fn test_clean_unsupported_extension() {
    let code = r"
Some random content
That is not code
";
    let result = skeleton::clean(Path::new("test.xyz"), code);
    assert_eq!(result, code, "Should pass through unsupported files");
}

#[test]
fn test_structs_preserved() {
    let code = r"
pub struct Config {
    pub max_tokens: usize,
    pub max_depth: usize,
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("pub struct Config"));
    assert!(result.contains("max_tokens: usize"));
}

#[test]
fn test_enums_preserved() {
    let code = r"
pub enum Status {
    Active,
    Inactive,
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("pub enum Status"));
    assert!(result.contains("Active"));
}

#[test]
fn test_traits_preserved() {
    let code = r"
pub trait Processor {
    fn process(&self) -> Result<(), Error>;
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("pub trait Processor"));
    assert!(result.contains("fn process"));
}

#[test]
fn test_imports_preserved() {
    let code = r"
use std::io;
use std::collections::HashMap;

fn main() {
    // code goes here
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("use std::io;"));
    assert!(result.contains("fn main() { ... }"));
}

#[test]
fn test_js_imports_preserved() {
    let code = r"
import { useState } from 'react';
import axios from 'axios';

function Component() {
    // render
}
";
    let result = skeleton::clean(Path::new("test.js"), code);
    assert!(result.contains("import { useState }"));
    assert!(result.contains("function Component() { ... }"));
}

#[test]
fn test_doc_comments_preserved() {
    let code = r"
/// This is a doc comment explaining the function
/// It has multiple lines
pub fn documented_function() {
    // inner logic
}
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("/// This is a doc comment"));
    assert!(result.contains("pub fn documented_function() { ... }"));
}

#[test]
fn test_type_aliases_preserved() {
    let code = r"
type Result<T> = std::result::Result<T, Error>;
type Handler = fn(&Request) -> Response;
";
    let result = skeleton::clean(Path::new("test.rs"), code);
    assert!(result.contains("type Result<T>"));
    assert!(result.contains("type Handler"));
}