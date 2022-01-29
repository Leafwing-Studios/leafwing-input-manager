use cargo_manifest::Manifest;
use proc_macro::TokenStream;
use std::{env, path::PathBuf};

pub(crate) fn get_path() -> syn::Path {
    LeafwingManifest::default().get_path("leafwing_input_manager")
}

struct LeafwingManifest {
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
        self.maybe_get_path(name)
            .unwrap_or_else(|| parse_str("crate"))
    }

    /// Get the path of the crate from the dependencies
    fn maybe_get_path(&self, name: &str) -> Option<syn::Path> {
        // The fallback, in case nothing is found
        let mut path = None;

        // Check direct dependencies
        if let Some(ref dependencies) = self.manifest.dependencies {
            if dependencies.get(name).is_some() {
                path = Some(parse_str(name));
            }
        }

        // Check dev dependencies
        if let Some(ref dependencies) = self.manifest.dev_dependencies {
            if dependencies.get(name).is_some() {
                path = Some(parse_str(name));
            }
        }

        path
    }
}

fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
    syn::parse(path.parse::<TokenStream>().unwrap()).unwrap()
}
