# Hash Flip-Flopping Investigation

**Status:** Open  
**Discovered:** 2026-01-02  
**Severity:** Critical - Blocks patch workflow  
**Affected:** Patch application on Windows with CRLF files

---

## The Problem

While attempting to apply patches to `src/apply/validator.rs`, the BASE_SHA256 hash verification failed repeatedly - but not consistently. The hash flip-flopped between two distinct values across consecutive commands:

```
Hash A: 4bf584d72cdc44c1c0cd43acc41923ed6659503e1e18f548038ba95fe8a28e26
Hash B: 1d95de406269561a8176c551104b53c439bf936ded53c0a4e7c417f4fdef061a
```

The file was NOT being modified between attempts:
- `git status` showed clean working tree
- `git diff` showed no changes
- File content was stable (CRLF line endings confirmed)

---

## Observed Behavior

1. Run `slopchop apply` with Hash A → fails, reports actual hash is B
2. Update patch to use Hash B → fails, reports actual hash is A
3. Repeat indefinitely - hash ping-pongs between two values

---

## Environment

- **OS:** Windows
- **Line endings:** CRLF (confirmed via `file` and `cat -A`)
- **Git autocrlf:** Unknown (needs check)
- **Concurrent sessions:** Another AI session was open (may have been factor)

---

## Hypotheses

### H1: Line Ending Normalization Race

Hash computation might normalize CRLF→LF inconsistently:
- `pack` computes hash one way
- `apply` computes hash another way
- Or: same code path, but caching/timing affects result

**Investigate:**
- `src/apply/patch/common.rs` - `compute_sha256()` function
- Does it read raw bytes or normalize first?
- Is there any caching layer?

### H2: File Read Timing / Caching

Windows file system or Rust's `fs::read_to_string` may behave inconsistently:
- File handle caching
- Antivirus scanning mid-read
- Editor holding file handle

**Investigate:**
- Add debug logging to hash computation
- Print raw byte length before hashing
- Check if byte count is stable

### H3: Two Code Paths Computing Hash Differently

Possible that `pack` and `apply` use different hash functions:
- `pack` uses one implementation
- `apply` verification uses another
- They handle line endings differently

**Investigate:**
- Grep for all `sha256` / `compute_sha256` calls
- Ensure single source of truth

### H4: Stage vs Workspace Confusion

Hash might sometimes be computed from:
- The workspace file
- The staged copy
- A cached/stale version

**Investigate:**
- Trace exactly which file path is being hashed
- Add path to error message for debugging

---

## Reproduction Steps

```bash
# 1. Confirm file is stable
git status src/apply/validator.rs
cat -A src/apply/validator.rs | head -3  # confirm CRLF

# 2. Get hash via pack
slopchop pack --focus src/apply/validator.rs --noprompt
grep SHA256 context.txt

# 3. Immediately get hash again
slopchop pack --focus src/apply/validator.rs --noprompt
grep SHA256 context.txt

# 4. Compare - are they the same?
```

---

## Suggested Fix Approach

1. **Add deterministic normalization:** Always normalize to LF before hashing, everywhere
2. **Single hash function:** Ensure `pack` and `apply` use identical code path
3. **Debug logging:** Temporarily log byte count and first/last bytes before hash
4. **Test:** Create unit test that hashes same CRLF file 100 times, assert all match

---

## Workaround (Current)

When hash ping-pongs, bypass slopchop and edit file manually:
```bash
# Just edit the damn file
code src/apply/validator.rs
cargo clippy
cargo install --path . --force
```

---

## Notes

- This may be related to the CRLF/trailing newline fix from v1.3.0
- That fix added `try_match_trimmed()` fallback for patch MATCHING
- But hash COMPUTATION may have a parallel issue
- The two hashes differ by exactly the delta you'd expect from CRLF vs LF
