use cargo_manifest::Manifest;
use proc_macro::{Span, TokenStream};
use std::{env, path::PathBuf};

pub(crate) fn get_path() -> syn::Path {
    LeafwingManifest::default().get_path("leafwing_input_manager")
}

pub struct LeafwingManifest {
    manifest: Manifest,
}

impl Default for LeafwingManifest {
    fn default() -> Self {
        Self {
            manifest: env::var_os("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .map(|mut path| {
                    path.push("Cargo.toml");
                    Manifest::from_path(path).unwrap()
                })
                .unwrap(),
        }
    }
}

impl LeafwingManifest {
    /// Get the path of the crate, based on the dependencies
    ///
    /// If it cannot be found, assume that the usage is internal
    fn get_path(&self, name: &str) -> syn::Path {
        // Check if we are in associated crate
        if let Some(ref package) = self.manifest.package {
            let package_name = formatted_package_name(&package.name);

            if package_name == name {
                let call_path = Span::call_site().source_file().path();
                let call_path_str = call_path.to_string_lossy();

                if call_path_str.contains("tests") || call_path_str.contains("examples") {
                    // If we are in the integration tests or examples of the crate,
                    // the import uses `name`
                    return parse_str(name);
                } else {
                    // If we are in the unit tests and code of the crate,
                    // the import uses `crate`
                    return parse_str("crate");
                }
            }
        }

        // Check direct dependencies
        if let Some(ref dependencies) = self.manifest.dependencies {
            if dependencies.get(name).is_some() {
                return parse_str(name);
            }
        }

        // Check dev dependencies
        if let Some(ref dependencies) = self.manifest.dev_dependencies {
            if dependencies.get(name).is_some() {
                return parse_str(name);
            }
        }

        panic!("The package {name} was not found in `Cargo.toml`. Did you forget to add it?")
    }
}

fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
    syn::parse(path.parse::<TokenStream>().unwrap()).unwrap()
}

/// The set of valid package names and rust identifiers are not identical
///
/// Convert the package name to match the rust identifier by making it lowercase
/// and converting hyphens to underscores.
fn formatted_package_name(string: &str) -> String {
    string.to_lowercase().replace("-", "_")
}
