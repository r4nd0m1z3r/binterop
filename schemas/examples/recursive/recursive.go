package recursive

import (
	"binterop/helpers"
)
var _ = binterop.NewVector[byte]()

type Recursive struct {
	Recursive bool
	Depth uint32
}

