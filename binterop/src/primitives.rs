use crate::types::PrimitiveType;
use std::ops::Index;

pub struct Primitives(phf::OrderedMap<&'static str, PrimitiveType>);
impl Primitives {
    const fn new() -> Self {
        let map = phf::phf_ordered_map! {
            "bool" => PrimitiveType {
                name: "bool",
                size: 1,
            },
            "i8" => PrimitiveType {
                name: "i8",
                size: 1,
            },
            "u8" => PrimitiveType {
                name: "u8",
                size: 1,
            },
            "i16" => PrimitiveType {
                name: "i16",
                size: 2,
            },
            "u16" => PrimitiveType {
                name: "u16",
                size: 2,
            },
            "i32" => PrimitiveType {
                name: "i32",
                size: 4,
            },
            "u32" => PrimitiveType {
                name: "u32",
                size: 4,
            },
            "i64" => PrimitiveType {
                name: "i64",
                size: 8,
            },
            "u64" => PrimitiveType {
                name: "u64",
                size: 8,
            },
            "f32" => PrimitiveType {
                name: "f32",
                size: 4,
            },
            "f64" => PrimitiveType {
                name: "f64",
                size: 8,
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
