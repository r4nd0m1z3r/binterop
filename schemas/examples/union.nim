import helpers

type SomeOtherType = object
  a*: float64

type Test = object
  b*: uint32
  a*: array[69, uint8]

type SomeStruct = object
  some_bool*: bool
  some_uint*: uint16
  some_float*: float32
  some_int*: int64
  some_pointer*: ptr SomeOtherType
  some_other_type*: SomeOtherType
  some_other_type_array*: array[3, SomeOtherType]
  some_other_type_vector*: Vector[SomeOtherType]
  some_string*: String
  some_float_array*: array[10, float32]

type Color = enum
  Red
  Green
  Blue

type SomeUnionVariant = enum
  ColorVariant
  SomeStructVariant

type SomeUnion = object
  case variant: SomeUnionVariant
  of ColorVariant:
    Color*: Color
  of SomeStructVariant:
    SomeStruct*: SomeStruct


