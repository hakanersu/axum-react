See AGENTS.md for the full project guide (architecture, conventions, gotchas, step-by-step workflows).

## Claude-Specific Notes

- Always read the relevant source file before editing it.
- When adding a new resource, follow the 5-step workflow in AGENTS.md ("How to Add a New Resource").
- Run `cargo build -p sekizgen-backend` after any Rust change to confirm it compiles before finishing.
- Run `make dev` to verify the full stack works end-to-end when requested.
- Commit messages should be imperative, concise, and include a `Co-Authored-By` trailer.
