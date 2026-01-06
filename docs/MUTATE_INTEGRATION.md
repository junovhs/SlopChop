# Integration Instructions for src/mutate/

## 1. Copy the mutate/ folder

Copy the entire `mutate/` folder to `src/mutate/` [THIS IS DONE, THE REST IS NOT]

## 2. Add to src/lib.rs

Add this line with the other module declarations:

```rust
pub mod mutate;
```

## 3. Add CLI command to src/cli/args.rs

Add this variant to the `Commands` enum (after `Stage`):

```rust
    /// Run mutation testing to find test gaps [EXPERIMENTAL]
    Mutate {
        /// Number of parallel workers (reserved for future use)
        #[arg(long, short)]
        workers: Option<usize>,
        /// Test timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Filter files by path pattern
        #[arg(long, short)]
        filter: Option<String>,
    },
```

## 4. Add import to src/cli/handlers.rs

Add this to the use statements at the top:

```rust
use crate::mutate::{self, MutateOptions};
```

## 5. Add handler function to src/cli/handlers.rs

Add this function (near the other handlers):

```rust
/// Handles the mutate command.
///
/// # Errors
/// Returns error if mutation testing fails.
pub fn handle_mutate(
    workers: Option<usize>,
    timeout: u64,
    json: bool,
    filter: Option<String>,
) -> Result<SlopChopExit> {
    let opts = MutateOptions {
        workers,
        timeout_secs: timeout,
        json,
        filter,
    };
    
    let repo_root = get_repo_root();
    let report = mutate::run(&repo_root, &opts)?;
    
    if report.summary.survived > 0 {
        Ok(SlopChopExit::CheckFailed)
    } else {
        Ok(SlopChopExit::Success)
    }
}
```

## 6. Add command dispatch in src/bin/slopchop.rs

Find where other commands are matched and add:

```rust
Some(Commands::Mutate { workers, timeout, json, filter }) => {
    handlers::handle_mutate(workers, timeout, json, filter)
}
```

## Notes

- **v1 Limitation**: Mutations run serially because parallel execution 
  requires separate workspace copies. The `--workers` flag is reserved
  for future parallel implementation.
  
- **Recommended usage**: Use `--filter` to target specific files for faster runs:
  ```bash
  slopchop mutate --filter src/tokens.rs
  ```
