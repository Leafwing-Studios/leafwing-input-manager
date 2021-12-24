# About

This is the Leafwing Studios' template repo, providing a quick, opinionated base for high-quality Bevy game projects (and libraries).
We've shaved the yaks for you!

The licenses here are provided for template purposes: this repository itself is provided under MIT-0.
Feel free to use, hack and adopt this freely: no attribution needed.

## Instructions

### Getting started

[Use this template](https://github.com/Leafwing-Studios/template-repo/generate) by pressing the big green "Use this template" button in the top right corner of [this repo](https://github.com/Leafwing-Studios/template-repo) to create a new repository.

This repository has dynamic linking enabled for much faster incremental compile times.
If you're on Windows, you'll need to use the `nightly` Rust compiler.
Swap by using `rustup default nightly`.

If you are making a game:

1. Enable the features you need from Bevy in `Cargo.toml`.
2. Delete the `examples` folder.
3. Start writing your game. Your logic should be stored in `lib.rs` (and other files that are pulled in from it).
Then, add all of the plugins and build your `App` in `main.rs`.
4. If you only care about your game working on `nightly`, remove `stable` from the `toolchain` field in `.github/workflows/ci.yml`.

If you are making a standalone library:

1. Delete `main.rs` and the `[[bin]]` section of the top-level `Cargo.toml`.
2. Change `default-features` to `false` for the `bevy` dependency to avoid unnecessarily pulling in extra features for your users.

Finally:

1. Rename the lib and bin in `Cargo.toml` (and all imports to ensure your code compiles).
2. Double check that the LICENSE matches your intent.
3. Update this README to match your project, modifying `About`, `Getting Started` and other sections as needed.
4. Consider cleaning up the issue and PR templates found in the `.github` folder to better match your needs.

### Running your game

Use `cargo run`.
This repo is set up to always build with full optimizations, so there's no need for a `--release` flag in most cases.
Dynamic linking is enabled to ensure build times stay snappy.

To run an example, use `cargo run --example_name`, where `example_name` is the file name of the example without the `.rs` extension.

## Contributing

This repository is open to community contributions!
There are a few options if you'd like to help:

1. File issues for bugs you find or new features you'd like.
2. Read over and discuss issues, then make a PR that fixes them. Use "Fixes #X" in your PR description to automatically close the issue when the PR is merged.
3. Review existing PRs, and leave thoughtful feedback. If you think a PR is ready to merge, hit "Approve" in your review!

Any contributions made are provided under the license(s) listed in this repo at the time of their contribution, and do not require separate attribution.

### Testing

1. Use doc tests aggressively to show how APIs should be used.
You can use `#` to hide a setup line from the doc tests.
2. Unit test belong near the code they are testing. Use `#[cfg(test)]` on the test module to ignore it during builds, and `#[test]` on the test functions to ensure they are run.
3. Integration tests should be stored in the top level `tests` folder, importing functions from `lib.rs`.

Use `cargo test` to run all tests.

### CI

The CI will:

1. Ensure the code is formatted with `cargo fmt`.
2. Ensure that the code compiles.
3. Ensure that (almost) all `clippy` lints pass.
4. Ensure all tests pass on Windows, MacOS and Ubuntu.

Check this locally with:

1. `cargo run -p ci`
2. `cargo test --workspace`

To manually rerun CI:

1. Navigate to the `Actions` tab.
2. Use the dropdown menu in the CI run of interest and select "View workflow file".
3. In the top-right corner, select "Rerun workflow".

### Documentation

Reference documentation is handled with standard Rust doc strings.
Use `cargo doc --open` to build and then open the docs.

Design docs (or other book-format documentation) is handled with [mdBook](https://rust-lang.github.io/mdBook/index.html).
Install it with `cargo install mdbook`, then use `mdbook serve --open` to launch the docs.

### Benchmarking

To run the benchmarks, use `cargo bench`.

For more documentation on making your own benchmarks, check out [criterion's docs](https://bheisler.github.io/criterion.rs/book/index.html).
