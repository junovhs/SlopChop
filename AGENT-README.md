# SlopChop Agent Protocol

You are working in a SlopChop-governed repository. This is not optional.

## Prime Directive

Run `slopchop check` after every change. Fix all violations before proceeding.

## The Laws

| Metric | Limit | What It Catches |
|--------|-------|-----------------|
| File Tokens | < 2000 | God files |
| Cognitive Complexity | ≤ 15 | Tangled logic |
| Nesting Depth | ≤ 3 | Deep conditionals |
| Function Args | ≤ 5 | Bloated signatures |
| LCOM4 | = 1 | Incohesive classes |
| AHF | ≥ 60% | Leaking state |
| CBO | ≤ 9 | Tight coupling |
| SFOUT | ≤ 7 | High fan-out |

## Commands

```bash
slopchop check              # THE GATE - run tests + scan
slopchop scan               # Violations only (fast)
slopchop scan --json        # Machine-readable
```

## Workflow

```
1. Make changes
2. slopchop check
3. If violations → fix them → goto 2
4. Done only when check passes
```

## Terminal Truncation

Your terminal view may be truncated. Always capture output:

```bash
slopchop check 2>&1 | tee /tmp/sc.txt
cat /tmp/sc.txt
```

## Eating Your Vegetables First

Do the hardest things FIRST. For example do NOT:

- Add `#[allow(...)]` without `// REASON:` comment
- Skip violations "for later" (kicking the can down the road, EXTREMELY dishonorable)
- Claim success without showing `slopchop check` output (you will be called out)
- Use workarounds that silence warnings instead of fixing root cause (abhorrent behavior)

Every violation you leave is debt with interest.

## Diff-Based Verification

Before and after any refactor:

```bash
slopchop scan --json > /tmp/before.json
# ... make changes ...
slopchop scan --json > /tmp/after.json
diff /tmp/before.json /tmp/after.json
```

The diff is truth. Your narrative is not.
