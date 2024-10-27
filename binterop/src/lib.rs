use std::Vector;

use schema::Schema;
use types::pointer::PointerType;
use types::primitives::PRIMITIVES;
use types::{array::ArrayType, vector::VectorType, WrappedType};

pub mod field;
pub mod schema;
pub mod std;
pub mod types;

pub trait Binterop {
    fn binterop_type(schema: &mut Schema) -> WrappedType;
}

macro_rules! binterop_primitive {
    ($primitive_type:ty) => {
        impl Binterop for $primitive_type {
            fn binterop_type(_: &mut Schema) -> WrappedType {
                WrappedType::Primitive(PRIMITIVES[stringify!($primitive_type)])
            }
        }
    };
}

binterop_primitive!(i8);
binterop_primitive!(u8);
binterop_primitive!(i16);
binterop_primitive!(u16);
binterop_primitive!(i32);
binterop_primitive!(u32);
binterop_primitive!(i64);
binterop_primitive!(u64);

binterop_primitive!(f32);
binterop_primitive!(f64);

impl<T: Binterop, const N: usize> Binterop for [T; N] {
    fn binterop_type(schema: &mut Schema) -> WrappedType {
        let inner_type = T::binterop_type(schema);
        let inner_type_index = schema
            .wrapped_type_index(&inner_type)
            .expect("Provided schema doesnt contain this type!");

        let array_type = ArrayType::new(inner_type.r#type(), inner_type_index, N);
        schema.arrays.push(array_type);

        WrappedType::Array(ArrayType::new(inner_type.r#type(), inner_type_index, N))
    }
}

impl<T: Binterop> Binterop for *const T {
    fn binterop_type(schema: &mut Schema) -> WrappedType {
        let inner_type = T::binterop_type(schema);
        let inner_type_index = schema
            .wrapped_type_index(&inner_type)
            .expect("Provided schema doesnt contain this type!");

        WrappedType::Pointer(PointerType::new(inner_type.r#type(), inner_type_index))
    }
}
