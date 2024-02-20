//! Modified from [Bevy's CI runner](https://github.com/bevyengine/bevy/tree/main/tools/ci/src)

use bitflags::bitflags;
use itertools::Itertools;
use xshell::{cmd, Shell};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Check: u32 {
        const FORMAT = 0b00000001;
        const CLIPPY = 0b00000010;
        const TEST = 0b00001000;
        const DOC_TEST = 0b00010000;
        const DOC_CHECK = 0b00100000;
        const COMPILE_CHECK = 0b100000000;
    }
}

// This can be configured as needed
const CLIPPY_FLAGS: [&str; 3] = [
    "-Aclippy::type_complexity",
    "-Wclippy::doc_markdown",
    "-Dwarnings",
];

fn main() {
    // When run locally, results may differ from actual CI runs triggered by
    // .github/workflows/ci.yml
    // - Official CI runs latest stable
    // - Local runs use whatever the default Rust is locally

    let arguments = [
        ("lints", Check::FORMAT | Check::CLIPPY),
        ("test", Check::TEST),
        ("doc", Check::DOC_TEST | Check::DOC_CHECK),
        ("compile", Check::COMPILE_CHECK),
        ("format", Check::FORMAT),
        ("clippy", Check::CLIPPY),
        ("doc-check", Check::DOC_CHECK),
        ("doc-test", Check::DOC_TEST),
    ];

    let what_to_run = if let Some(arg) = std::env::args().nth(1).as_deref() {
        if let Some((_, check)) = arguments.iter().find(|(str, _)| *str == arg) {
            *check
        } else {
            println!(
                "Invalid argument: {arg:?}.\nEnter one of: {}.",
                arguments[1..]
                    .iter()
                    .map(|(s, _)| s)
                    .fold(arguments[0].0.to_owned(), |c, v| c + ", " + v)
            );
            return;
        }
    } else {
        Check::all()
    };

    let sh = Shell::new().unwrap();

    if what_to_run.contains(Check::FORMAT) {
        // See if any code needs to be formatted
        cmd!(sh, "cargo fmt --all -- --check")
            .run()
            .expect("Please run 'cargo fmt --all' to format your code.");
    }

    if what_to_run.contains(Check::DOC_CHECK) {
        // Check that building docs work and does not emit warnings
        std::env::set_var("RUSTDOCFLAGS", "-D warnings");
        cmd!(
            sh,
            "cargo doc --workspace --all-features --no-deps --document-private-items"
        )
        .run()
        .expect("Please fix doc warnings in output above.");
    }

    // The features the lib offers
    let lib_features = ["asset", "ui", "no_ui_priority", "egui"];

    // Generate all possible combinations of lib features
    // and convert them into '--features=<FEATURE_A,FEATURE_B,...>'
    let lib_features_options = (1..lib_features.len())
        .flat_map(|combination_length| lib_features.iter().combinations(combination_length))
        .map(|combination| String::from("--features=") + &combination.iter().join(","));

    let default_feature_options = ["--no-default-features", "--all-features"];
    let all_features_options = default_feature_options
        .iter()
        .map(|str| str.to_string())
        .chain(lib_features_options)
        .collect::<Vec<_>>();

    for feature_option in all_features_options {
        if what_to_run.contains(Check::CLIPPY) {
            // See if clippy has any complaints.
            // --all-targets was removed because Emergence currently has no special targets;
            // please add them back as necessary
            cmd!(
                sh,
                "cargo clippy --workspace {feature_option} -- {CLIPPY_FLAGS...}"
            )
            .run()
            .expect("Please fix clippy errors in output above.");
        }

        if what_to_run.contains(Check::TEST) {
            // Run tests (except doc tests and without building examples)
            cmd!(
                sh,
                "cargo test --workspace {feature_option} --lib --bins --tests --benches"
            )
            .run()
            .expect("Please fix failing tests in output above.");
        }

        if what_to_run.contains(Check::DOC_TEST) {
            // Run doc tests
            cmd!(sh, "cargo test --workspace {feature_option} --doc")
                .run()
                .expect("Please fix failing doc-tests in output above.");
        }

        if what_to_run.contains(Check::COMPILE_CHECK) {
            cmd!(sh, "cargo check --workspace {feature_option}")
                .run()
                .expect("Please fix compiler errors in above output.");
        }
    }
}
