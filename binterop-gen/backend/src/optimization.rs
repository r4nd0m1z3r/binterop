use binterop::schema::Schema;
use std::alloc::Layout;
use std::{env, slice};

#[derive(Copy, Clone)]
pub struct SchemaOptimizations {
    pub data_type_layout: bool,
    pub add_padding: bool,
}
impl Default for SchemaOptimizations {
    fn default() -> Self {
        let args = env::args().collect::<Vec<_>>();

        Self {
            data_type_layout: !args.contains(&"--dont-optimize-layout".to_string()),
            add_padding: !args.contains(&"--dont-add-padding".to_string()),
        }
    }
}
impl SchemaOptimizations {
    pub fn new(data_type_layout: bool, add_padding: bool) -> Self {
        Self {
            data_type_layout,
            add_padding,
        }
    }
}

fn optimize_data_type_layouts(schema: &mut Schema) {
    let mut field_sizes = schema
        .types
        .iter()
        .flat_map(|data_type| data_type.fields.iter().map(|field| field.size(schema)))
        .collect::<Vec<_>>();
    let mut field_sizes_cursor = 0;

    for data_type in &mut schema.types {
        let field_sizes =
            &mut field_sizes[field_sizes_cursor..field_sizes_cursor + data_type.fields.len()];
        field_sizes_cursor += data_type.fields.len();

        let mut permutation = permutation::sort_unstable_by(&field_sizes, |f1, f2| f1.cmp(f2));
        permutation.apply_slice_in_place(&mut data_type.fields);
        permutation.apply_slice_in_place(field_sizes);

        let mut field_offset = 0;
        for (field, &size) in data_type.fields.iter_mut().zip(field_sizes.iter()) {
            field.offset = field_offset;
            field_offset += size;
        }
    }
}

fn add_padding(schema: &mut Schema) {
    let types = unsafe { slice::from_raw_parts_mut(schema.types.as_mut_ptr(), schema.types.len()) };

    for data_type in types {
        let mut layout = Layout::from_size_align(0, 1).unwrap();
        for field in &mut data_type.fields {
            let field_layout = field.layout(schema);
            let (new_layout, offset) = layout.extend(field_layout).unwrap();
            layout = new_layout;
            field.offset = offset;
        }
    }

    schema.is_packed = false;
}

pub fn optimize_schema(schema: &mut Schema, optimizations: SchemaOptimizations) {
    if optimizations.data_type_layout {
        optimize_data_type_layouts(schema);
    }
    if optimizations.add_padding {
        add_padding(schema);
    }
}
