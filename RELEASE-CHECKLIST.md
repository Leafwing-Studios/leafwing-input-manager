# LWIM Release Checklist

1. Bump the crate version.
2. Ensure no tests (other than ones in the README) are ignored.
3. Manually verify that all examples work.
4. Make sure there's no Git dependencies.
5. Remove "(unreleased)" from the RELEASES.md.
6. Generate a tag and push it to origin.
7. Update the table in README.md.
