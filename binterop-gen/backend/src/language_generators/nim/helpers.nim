type None = object

type Option*[T] = object
  case isSome*: bool
  of true:
    value*: T
  of false:
    none*: None

type Vector*[T] = object
  pointer*: ptr UncheckedArray[T]
  len*: uint64
  capacity*: uint64

type String* = Vector[uint8]

proc newVector*[T](capacity: uint64 = 0): Vector[T] =
  return Vector[T](
    pointer: cast[ptr UncheckedArray[T]](allocShared(capacity * sizeof(T).uint64)),
    len: 0,
    capacity: capacity
  )

proc asSeq*[T](vec: var Vector[T]): seq[T] =
  @vec.pointer.toOpenArray(0, vec.len - 1)

proc reserve*[T](vec: var Vector[T], additional: uint64) =
  vec.capacity += additional
  vec.pointer = cast[ptr UncheckedArray[T]](vec.pointer.reallocShared(vec.capacity))

proc push*[T](vec: var Vector[T], value: T) =
  if vec.len == vec.capacity:
    reserve(vec, vec.capacity + 1)
  vec.pointer[vec.len] = value
  inc(vec.len)

proc pop*[T](vec: var Vector[T]): Option[T] =
  if vec.len == 0:
    return Option[T](isSome: false)
  let value = vec.pointer[vec.len - 1]
  dec(vec.len)
  Option[T](isSome: true, value: value)
