#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

typedef struct __attribute__((packed)) {
	double a;
} SomeOtherType;

typedef struct __attribute__((packed)) {
	SomeOtherType* ptr;
	uint64_t len;
} ArraySomeOtherType;

typedef struct __attribute__((packed)) {
	bool some_bool;
	uint16_t some_uint;
	float some_float;
	int64_t some_int;
	SomeOtherType* some_pointer;
	SomeOtherType some_other_type;
	ArraySomeOtherType some_other_type_heap_array;
	SomeOtherType some_other_type_array[3];
	float some_float_array[10];
} SomeStruct;

ArraySomeOtherType ArraySomeOtherType_new(uint64_t len) { return (ArraySomeOtherType){ malloc(sizeof(SomeOtherType) * len), len }; }
