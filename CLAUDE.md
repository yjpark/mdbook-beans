# mdbook-beans

mdBook preprocessor that injects beans task data as book chapters.

## Rust Workflow

bacon is running in the background and continuously writes compiler
diagnostics to `.bacon-claude-diagnostics` in the project root.

Before attempting to fix compiler errors, read `.bacon-claude-diagnostics` to see
current errors and warnings with their exact file/line/column locations.
Prefer reading this file over running `cargo check` yourself — it's
already up to date and costs no compile time.

Each line in `.bacon-claude-diagnostics` uses a pipe-delimited format:

```
level|:|file|:|line_start|:|line_end|:|message|:|rendered
```

- `level` — severity: `error`, `warning`, `note`, `help`
- `file` — relative path to the source file
- `line_start` / `line_end` — affected line range
- `message` — short diagnostic message
- `rendered` — full cargo-rendered output including code context and suggestions

After making changes, wait a moment for bacon to recompile, then re-read
`.bacon-claude-diagnostics` to verify the fix.

**All compiler warnings must be fixed before committing.** Zero warnings is the
standard. Check `.bacon-claude-diagnostics` for warnings (not just errors) and
resolve them as part of every change.

If `.bacon-claude-diagnostics` is absent or clearly stale, warn the user that
bacon does not appear to be running and ask them to start it.

## Planning

Do NOT write design docs or plans to local files. All planning and design
work should be captured in GitHub issues on edger-dev/mdbook-beans.

## Development Workflow

### Test-Driven Development

Write tests **before** implementation. The sequence:

1. Write tests that capture the expected behavior from the spec
2. Run `cargo test` — confirm tests fail for the right reasons
3. Implement the minimum code to make tests pass
4. Verify all tests pass

### Commit Granularity

Each task should produce 2–3 focused commits:

1. **Tests commit** — the failing tests that define the expected behavior
2. **Implementation commit** — the code that makes them pass, plus any warning fixes
3. **Review fixes commit** (if needed) — issues caught during code review

### Acceptance Criteria

Every task must pass before being marked complete:

- All `cargo test` tests pass
- Zero compiler warnings in `.bacon-claude-diagnostics`
- Changes committed with descriptive messages
