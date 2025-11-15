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

proc newVector*[T](capacity: uint64 = 0): Vector[T] =
  Vector[T](
    pointer: cast[ptr UncheckedArray[T]](allocShared(capacity * sizeof(T).uint64)),
    len: 0,
    capacity: capacity
  )

proc freeVector*[T](vec: sink Vector[T]) =
  vec.pointer.freeShared()

proc toVector*[T](seq: seq[T]): Vector[T] =
  var vec = newVector[T](seq.capacity.uint64)
  for item in seq:
    vec.push(item)
  vec

proc toSeq*[T](vec: sink Vector[T]): seq[T] =
  let last = vec.len - 1
  @(vec.pointer.toOpenArray(0, last.int))

proc reserve*[T](vec: var Vector[T], additional: uint64) =
  vec.capacity += additional
  vec.pointer = cast[ptr UncheckedArray[T]](vec.pointer.reallocShared(vec.capacity))

proc push*[T](vec: var Vector[T], value: T) =
  if vec.len == vec.capacity:
    vec.reserve(vec.capacity * 2)
  vec.pointer[vec.len] = value
  vec.len += 1

proc pop*[T](vec: var Vector[T]): Option[T] =
  if vec.len == 0:
    return Option[T](isSome: false)
  let value = vec.pointer[vec.len - 1]
  vec.len -= 1
  Option[T](isSome: true, value: value)

type String* = Vector[uint8]
  
proc freeString*(str: sink String) =
  freeVector(str)

proc toBinteropString*(str: string): String =
  var bytes = newVector[uint8](len(str).uint64)

  for ch in str:
    bytes.push(ch.uint8)

  bytes.String

proc toNimString*(str: sink String): string =
  for ch in str.toSeq():
    result.add(ch.char)
