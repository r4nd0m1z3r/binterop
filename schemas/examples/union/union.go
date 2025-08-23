package union

import (
	"binterop/helpers"
)
var _ = binterop.NewVector[byte]()

type SomeOtherType struct {
	A float64
}

type Test struct {
	B uint32
	A [69]uint8
}

type SomeStruct struct {
	SomeBool bool
	SomeUint uint16
	SomeFloat float32
	SomeInt int64
	SomePointer *SomeOtherType
	SomeOtherType SomeOtherType
	SomeOtherTypeArray [3]SomeOtherType
	SomeOtherTypeVector binterop.Vector[SomeOtherType]
	SomeString binterop.String
	SomeFloatArray [10]float32
}

type Color int32
const (
	Red Color = iota
	Green
	Blue
)

type SomeUnionVariant int32
const (
	ColorVariant SomeUnionVariant = iota
	SomeStructVariant
)

type SomeUnion struct {
  Variant SomeUnionVariant
  Data [143]byte
}

