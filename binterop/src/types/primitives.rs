use serde::{Deserialize, Serialize};
use std::mem::{align_of, size_of};
use std::ops::Index;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PrimitiveType {
    pub name: &'static str,
    pub size: usize,
    pub align: usize,
}

pub struct Primitives(phf::OrderedMap<&'static str, PrimitiveType>);
impl Primitives {
    const fn new() -> Self {
        let map = phf::phf_ordered_map! {
            "bool" => PrimitiveType {
                name: "bool",
                size: size_of::<bool>(),
                align: align_of::<bool>()
            },
            "i8" => PrimitiveType {
                name: "i8",
                size: size_of::<i8>(),
                align: align_of::<i8>()
            },
            "u8" => PrimitiveType {
                name: "u8",
                size: size_of::<u8>(),
                align: align_of::<u8>()
            },
            "i16" => PrimitiveType {
                name: "i16",
                size: size_of::<i16>(),
                align: align_of::<i16>()
            },
            "u16" => PrimitiveType {
                name: "u16",
                size: size_of::<u16>(),
                align: align_of::<u16>()
            },
            "i32" => PrimitiveType {
                name: "i32",
                size: size_of::<i32>(),
                align: align_of::<i32>()
            },
            "u32" => PrimitiveType {
                name: "u32",
                size: size_of::<u32>(),
                align: align_of::<u32>()
            },
            "i64" => PrimitiveType {
                name: "i64",
                size: size_of::<i64>(),
                align: align_of::<i64>()
            },
            "u64" => PrimitiveType {
                name: "u64",
                size: size_of::<u64>(),
                align: align_of::<u64>()
            },
            "f32" => PrimitiveType {
                name: "f32",
                size: size_of::<f32>(),
                align: align_of::<f32>()
            },
            "f64" => PrimitiveType {
                name: "f64",
                size: size_of::<f64>(),
                align: align_of::<f64>()
            }
        };

        Self(map)
    }

    pub fn index(&self, index: usize) -> Option<PrimitiveType> {
        self.0.index(index).map(|(_, &r#type)| r#type)
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.0.get_index(name)
    }

    pub fn name_of(&self, index: usize) -> Option<&str> {
        self.0.index(index).map(|(&name, _)| name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.0.keys().copied().collect()
    }

    pub fn types(&self) -> Vec<PrimitiveType> {
        self.0.values().copied().collect()
    }
}
impl Index<&str> for Primitives {
    type Output = PrimitiveType;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

pub const PRIMITIVES: Primitives = Primitives::new();
pub const INTEGER_PRIMITIVE_NAMES: [&str; 8] =
    ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"];
