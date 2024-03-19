#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

typedef struct {
	double a;
} SomeOtherType;

typedef struct {
	SomeOtherType* ptr;
	uint64_t len;
} ArraySomeOtherType;

typedef struct {
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

static inline ArraySomeOtherType ArraySomeOtherType_new(uint64_t len) { return (ArraySomeOtherType){ (SomeOtherType*)calloc(len, sizeof(SomeOtherType)), len }; }
static inline void ArraySomeOtherType_resize(ArraySomeOtherType* array, uint64_t new_len) { array->ptr = realloc(array->ptr, sizeof(SomeOtherType) * new_len); array->len = new_len; }
