package binterop

import (
	"unsafe"
	"strings"
	"fmt"
)

type Vector[T any] struct {
	ptr      uint64
	length   uint64
	capacity uint64
}
func (v *Vector[T]) String() string {
	var sb strings.Builder
	sb.WriteString("[")
	for i, v := range v.AsSlice() {
		if i > 0 {
			sb.WriteString(", ")
		}
		sb.WriteString(fmt.Sprintf("%v", v))
	}
	sb.WriteString("]")
	return sb.String()
}

type String struct {
	data Vector[byte]
}

func NewVector[T any]() *Vector[T] {
	return &Vector[T]{}
}

func VectorWithCapacity[T any](capacity uint64) *Vector[T] {
	vec := &Vector[T]{}
	if capacity > 0 {
		slice := make([]T, 0, capacity)
		vec.ptr = uint64(uintptr(unsafe.Pointer(&slice[:1][0])))
		vec.capacity = capacity
	}
	return vec
}

func (v *Vector[T]) AsSlice() []T {
	if v.ptr == 0 || v.length == 0 {
		return []T{}
	}
	
	slice := (*[1 << 30]T)(unsafe.Pointer(uintptr(v.ptr)))[:v.length:v.length]
	return slice
}

func (v *Vector[T]) Length() uint64 {
	return v.length
}

func (v *Vector[T]) Capacity() uint64 {
	return v.capacity
}

func (v *Vector[T]) IsEmpty() bool {
	return v.length == 0
}

func (v *Vector[T]) Push(elem T) {
	appendedSlice := append(v.AsSlice(), elem)
	
	v.capacity = uint64(cap(appendedSlice))
	v.ptr = uint64(uintptr(unsafe.Pointer(&appendedSlice[0])))
	v.length++
}

func (v *Vector[T]) Pop() (T, bool) {
	var zero T
	if v.ptr == 0 || v.length == 0 {
		return zero, false
	}

	v.length--
	return zero, true
}

func NewString() *String {
	return &String{}
}

func StringFromBytes(data []byte) *String {
	s := &String{}
	if len(data) > 0 {
		s.data.capacity = uint64(len(data))
		s.data.length = uint64(len(data))
		s.data.ptr = uint64(uintptr(unsafe.Pointer(&data[0])))
	}
	return s
}

func (s *String) AsBytes() []byte {
	if s.data.ptr == 0 || s.data.length == 0 {
		return []byte{}
	}

	slice := (*[1 << 30]byte)(unsafe.Pointer(uintptr(s.data.ptr)))[:s.data.length:s.data.length]
	return slice
}

func (s *String) AsString() string {
	if s.data.ptr == 0 || s.data.length == 0 {
		return ""
	}

	return string(s.AsBytes())
}

func FromGoString(str string) *String {
	if str == "" {
		return NewString()
	}

	data := []byte(str)
	return StringFromBytes(data)
}

func VectorFromSlice[T any](slice []T) *Vector[T] {
	vec := &Vector[T]{}
	if len(slice) > 0 {
		vec.capacity = uint64(len(slice))
		vec.length = uint64(len(slice))
		vec.ptr = uint64(uintptr(unsafe.Pointer(&slice[0])))
	}
	return vec
}

func (v *Vector[T]) GetVectorData() unsafe.Pointer {
	return unsafe.Pointer(uintptr(v.ptr))
}

func (s *String) GetStringData() unsafe.Pointer {
	return unsafe.Pointer(uintptr(s.data.ptr))
}
