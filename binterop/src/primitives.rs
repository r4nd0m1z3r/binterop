use crate::types::PrimitiveType;

pub const PRIMITIVES: phf::OrderedMap<&'static str, PrimitiveType> = phf::phf_ordered_map! {
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
    },
};
