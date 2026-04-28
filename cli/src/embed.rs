//! Embedded Superpowers content baked into the binary at compile time.
//!
//! `DIST_USER` is the entire `../dist/user/` tree from the project root.
//! At runtime it provides read-only access to skill files, rules, and references.

use include_dir::{include_dir, Dir};

pub static DIST_USER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist/user");
