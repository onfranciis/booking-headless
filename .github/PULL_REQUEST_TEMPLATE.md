## Description

<!-- What does this PR do, and why? -->

## Related Issue

<!-- Closes #... -->

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo test` passes
- [ ] If a SQL query or migration changed, ran `cargo sqlx prepare -- --lib` and committed the updated offline metadata
- [ ] Added/updated tests in `src/tests/unit_test.rs` for any new logic
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
