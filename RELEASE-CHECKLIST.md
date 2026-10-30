# LWIM Release Checklist

1. Bump crate versions before tagging:
   - Set the `version` field in the root [`Cargo.toml`](Cargo.toml) (`leafwing-input-manager`).
   - If the `macros` crate changed, set the `version` field in [`macros/Cargo.toml`](macros/Cargo.toml) (`leafwing_input_manager_macros`) and update the `version` on the `leafwing_input_manager_macros` path dependency in the root `Cargo.toml` to match.
2. Ensure no tests (other than ones in the README) are ignored.
3. Manually verify that all examples work.
4. Make sure there's no Git dependencies.
5. Remove "(unreleased)" from the RELEASES.md.
6. Generate a tag and push it to origin.
7. Update the table in README.md.
