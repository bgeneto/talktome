PROJECT CONTEXT
Repo: local Tauri 2.0 + Rust project (frontend: Svelte; backend: Rust Tauri).
Purpose: small desktop app described in @README.md 
Primary target and dev platform: Windows/Powershell (verify compatibility), then Linux/macOS.
ENVIRONMENT & RUNNING
Rust toolchain: stable (use rustup default stable). If project requires a specific toolchain, use file rust-toolchain.
Node: >=16 (or project's package.json engine). Use npm/yarn as in repo.
Commands:
Install deps: npm install (or yarn)
Build frontend: npm run build (@README.md if exists)
Run app dev: npm run tauri dev or cargo tauri dev
Build release: cargo tauri build
CI checks to run locally: cargo fmt --all -- --check, cargo clippy --all -- -D warnings, frontend linter/tests as repo defines.
BUGS / FEATURES TO FIX (priority order)
Persistence of settings (highest)
Symptom: changing settings (API key, model selection) appears applied in UI but on restart settings revert to defaults.
Goal: reliably persist and restore user settings across restarts on Windows (then cross-platform).
Acceptance criteria:
After changing settings and closing/reopening app, the new settings persist.
Unit/integration tests for the persistence layer (where possible).
Provide a short manual verification: steps to reproduce, and a small test script if feasible.
Hotkey (start/stop recording) instability
Symptom: pressing the configured combo once triggers many start/stop events; same combo is used to toggle start/stop.
Goal: deterministic start/stop toggle with no phantom triggers. Ensure same combo toggles reliably.
Acceptance criteria:
Press combo once → single toggle. Rapid repeated presses behave predictably (debounced or stateful).
Add automated tests where possible (integration tests or simulated events) and document manual test steps.
Ensure Windows global shortcut implementation is robust.
Code cleanup and refactor
Identify and remove unused/non-required code and nonsense patterns.
Replace anti-patterns with SOTA Rust/Tauri patterns (see code-style section).
Acceptance criteria:
Provide a list of removed files/sections and rationale.
Each removal has tests or a risk note; no regressions introduced.
PR includes cargo fmt and cargo clippy clean results.
Propose improvements and document them
Update README or add a REFactorING.md describing major changes and migration notes.
SCOPE & CONSTRAINTS
Do not change high-level app behaviour described in @/README.md unless explicitly annotated.
Avoid breaking public APIs used by external consumers (if any).
Target Windows compatibility first; include platform-specific fixes and tests.
Keep changes modular and incremental — create a working commit for each subtask.
DETAILED IMPLEMENTATION GUIDANCE (recommended patterns)
Persistence:
Use a single consistent storage approach. Prefer Tauri-recommended persistence (e.g., tauri-plugin-store or Tauri store APIs) , remember to configure The Tauri store plugin required permissions in default.json like "store:allow-load", etc...
Ensure atomic writes and a clear migration path for previous formats. Use serde for (de)serialization and validate data on load; fall back to defaults with a migration log on failure.
Sync UI ↔ backend: changes from browser should call a dedicated API that both updates runtime state and persists to disk. Do not rely only on localStorage in browser for cross-platform reliability.
Hotkey:
Check for conflicting source-of-truth for hotkey state (there is a button in the main app page that should allow start and stop the recordin and must by in sync with hotkey state. 
Implement a simple deterministic finite state machine (states: Idle, Recording) and explicit transitions on key events.
Add debouncing or rate-limiting (e.g., ignore triggers within 200ms) only after ensuring the root cause is not duplicate event registration.
Ensure registration/unregistration lifecycle is correct (register once, unregister on app exit or user change).
Rust code style:
Use Result and ?; avoid unwrap()/expect() except in tests or startup where panic is acceptable.
Prefer small functions, single responsibility, modularization by feature.
Run and fix cargo clippy warnings; run cargo fmt.
Add unit tests for new logic, integration tests for critical paths (persistence, hotkey FSM).
DELIVERABLES (exact)
A feature branch named fix-refactor-storage-hotkey (or similar).
One PR (or a few small PRs) containing:
Clear commit history: each commit fixes one thing and has an imperative message: e.g. fix(settings): persist model selection using tauri store plugin
Diffs/patch files if PR not possible.
Tests added/updated and instructions to run them.
CHANGES.md short summary of changes and manual verification steps.
For anything left incomplete: a short TODO.md with exact next steps.
REPORTING & DEBUG ARTIFACTS
For each bug fixed, provide:
Reproduction steps (before and after).
Logs collected and root-cause analysis (short).
Files changed and justification.
Test commands and expected output.
If a change risks regressions, add a fallback (feature flag) or a migration README and a rollback plan.
ACCEPTANCE TESTS (manual + automated)
Manual: explicit sequence (press hotkey or button on main page once; observe recording toggled once; change model and restart app; model remains selected).
Automated: unit tests that validate persistence serialize/deserialize; FSM tests for hotkey state transitions.
CI: cargo fmt --all -- --check, cargo clippy --all -- -D warnings, run unit tests.
DEBUGGING PERMISSIONS & NOTES
You may re-organize modules and delete unused code if covered by tests or low risk.
If uncertain about intended behaviour, consult @/README.md, then implement the most conservative fix and document assumptions.
OUTPUT FORMAT
Start with a short plan (ordered tasks).
Then produce incremental commits implementing each task.
Include test output and verification steps in the PR description.
If any issue cannot be fixed fully, produce a reproducible failing test and a clear TODO with required info.
EXTRA (optional) improvements to propose
Think a smart way to properly implement the chunk recording mechanism to avoid submitting a large data request by always using the single recording mode. 
END
