#include <stdint.h>
#include <stdbool.h>

typedef struct __attribute__((packed)) {
	double a;
} SomeOtherType;

typedef struct __attribute__((packed)) {
	bool some_bool;
	int16_t some_uint;
	float some_float;
	int64_t some_int;
	SomeOtherType some_other_type;
	SomeOtherType some_other_type_array[3];
	float some_float_array[10];
} SomeStruct;

