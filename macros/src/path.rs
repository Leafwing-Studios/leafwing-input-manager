//! This logic is shamelessly stolen from `bevy_macro_utils`
//! Link: https://github.com/bevyengine/bevy/tree/main/crates/bevy_macro_utils
//! Copyright 2022, Bevy Contributors
//! Used under the MIT License

use cargo_manifest::{DepsSet, Manifest};
use proc_macro::TokenStream;
use std::{env, path::PathBuf};

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
        self.maybe_get_path(name)
            .unwrap_or_else(|| Self::parse_str("crate"))
    }

    /// Get the path of the crate from the dependencies
    fn maybe_get_path(&self, name: &str) -> Option<syn::Path> {
        let find_in_deps = |deps: &DepsSet| -> Option<syn::Path> {
            if let Some(dep) = deps.get(name) {
                Some(Self::parse_str(dep.package().unwrap_or(name)))
            } else {
                None
            }
        };

        let deps = self.manifest.dependencies.as_ref();
        let deps_dev = self.manifest.dev_dependencies.as_ref();

        deps.and_then(find_in_deps)
            .or_else(|| deps_dev.and_then(find_in_deps))
    }

    fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
        syn::parse(path.parse::<TokenStream>().unwrap()).unwrap()
    }
}

pub(crate) fn get_path() -> syn::Path {
    LeafwingManifest::default().get_path("leafwing_input_manager")
}
