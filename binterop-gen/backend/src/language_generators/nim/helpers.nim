type None = object

type Option*[T] = object
  case isSome*: bool
  of true:
    value*: T
  of false:
    none*: None

type Vector*[T] = object
  pointer*: UncheckedArray[T]
  len*: uint64
  capacity*: uint64

type String* = Vector[uint8]

proc newVector*[T](capacity: uint64 = 0): Vector[T] =
  var vec = Vector[T]()
  vec.ptr = alloc(capacity * sizeof(T))
  vec.len = 0
  vec.capacity = capacity
  vec

proc asSeq*[T](vec: Vector[T]): seq[T] =
  @vec.pointer.toOpenArray(0, vec.len - 1)

proc reserve*[T](vec: ref Vector[T], additional: uint64) =
  vec.pointer = vec.pointer.reallocShared(vec.capacity + additional)

proc push*[T](vec: ref Vector[T], value: T) =
  if vec.len == vec.capacity:
    reserve(vec, vec.capacity + 1)
  vec.pointer[vec.len] = value
  inc(vec.len)

proc pop*[T](vec: ref Vector[T]): Option[T] =
  if vec.len == 0:
    return Option[T](isSome: false)
  let value = vec.pointer[vec.len - 1]
  dec(vec.len)
  Option[T](isSome: true, value: value)
