#![feature(vec_into_raw_parts)]

use std::{env, sync::LazyLock};

static GENERATOR_DEBUG: LazyLock<bool> = LazyLock::new(|| {
    env::var("BINTEROP_GENERATOR_DEBUG")
        .map(|debug| debug.contains('1'))
        .unwrap_or(false)
});

pub mod generator;
pub mod helpers;
pub mod language_generators;
pub mod optimization;
pub mod tokenizer;
pub mod tokenizer_chumsky;
