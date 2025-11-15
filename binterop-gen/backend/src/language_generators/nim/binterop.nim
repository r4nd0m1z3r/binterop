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

iterator items*[T](v: var Vector[T]): var T =
  for i in 0..<v.len:
    yield v.pointer[i]

iterator items*[T](v: Vector[T]): T =
  for i in 0..<v.len:
    yield v.pointer[i]

iterator pairs*[T](v: Vector[T]): (uint64, T) =
  for i in 0..<v.len:
    yield (i, v.pointer[i])

iterator pairs*[T](v: var Vector[T]): (uint64, var T) =
  for i in 0..<v.len:
    yield (i, v.pointer[i])

proc freeVector*[T](vec: sink Vector[T]) =
  vec.pointer.freeShared()

proc toVector*[T](seq: seq[T]): Vector[T] =
  var vec = newVector[T](seq.capacity.uint64)
  for item in seq:
    vec.push(item)
  vec

template toOpenArray*[T](vec: var Vector[T]): openArray[T] =
  @(vec.pointer.toOpenArray(0, vec.len.int - 1))

proc toSeq*[T](vec: sink Vector[T]): seq[T] =
  @(vec.toOpenArray)

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
  for ch in str.toOpenArray():
    result.add(ch.char)
