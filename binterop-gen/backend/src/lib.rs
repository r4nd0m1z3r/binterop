use std::{env, sync::LazyLock};

static TIME: LazyLock<bool> = LazyLock::new(|| {
    env::var("BINTEROP_TIME")
        .map(|debug| debug.contains('1'))
        .unwrap_or(false)
});

pub mod generator;
pub mod helpers;
pub mod language_generators;
pub mod optimization;
pub mod tokenizer;
