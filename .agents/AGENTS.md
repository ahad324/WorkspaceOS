# WorkspaceOS Custom Workspace Rules

Mandatory behavior guidelines for AI assistants working on WorkspaceOS.

## Development & Release Integrity

Every agent working on any phase must ensure complete development integrity before ending their turn or declaring a phase complete:

1. **Zero Lint Warnings**: All lint scripts (`pnpm lint`, `pnpm lint:web`, `pnpm lint:rust`) must run and return 0 warnings and 0 errors.
2. **Deterministic Formatting**: All files must be formatted (`pnpm format`, `pnpm format:web`, `pnpm format:rust`) before committing changes.
3. **Tests Validation**: All unit and integration tests (`cargo test --workspace`) must pass cleanly.
4. **Build Verification**: Local development builds (`pnpm dev`, `pnpm build`, etc.) must compile without errors.
5. **CI Status**: Ensure that the GitHub Actions CI pipeline passes successfully.
