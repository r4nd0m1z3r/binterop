pub mod field;
pub mod schema;
pub mod types;

use crate::schema::Schema;
use crate::types::data::DataType;
use crate::types::primitives::PRIMITIVES;

#[allow(dead_code)]
fn generate_vec3_schema() -> Schema {
    let f32 = PRIMITIVES["f32"];
    let vec3 = DataType::from_primitives("Vec3", &[("x", f32), ("y", f32), ("z", f32)]);

    Schema::new(0, &[vec3], &[], &[], &[])
}

#[test]
fn test_vec3() {
    let schema = generate_vec3_schema();

    let f32_primitive = PRIMITIVES["f32"];

    assert_eq!(schema.root_size(), f32_primitive.size * 3);
}

#[test]
fn test_alloc() {
    let schema = generate_vec3_schema();

    let mut vec3 = schema.allocate_root(1);

    unsafe {
        #[repr(C, packed)]
        #[derive(Copy, Clone, Debug, PartialEq)]
        struct Vec3 {
            x: f32,
            y: f32,
            z: f32,
        }
        let vec3_struct = &mut *vec3.as_mut_ptr().cast::<Vec3>();

        let reference_value = Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        *vec3_struct = reference_value;

        assert_eq!(*vec3_struct, reference_value)
    }
}
