package enum

import (
	"binterop/helpers"
)
var _ = binterop.NewVector[byte]()

type Color int32
const (
	Red Color = iota
	Green
	Blue
)

