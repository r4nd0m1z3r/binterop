#include <stdint.h>
#include <stdbool.h>

typedef struct __attribute__((packed)) {
	double a;
} SomeOtherType;

typedef struct __attribute__((packed)) {
	int16_t some_uint;
	int64_t some_int;
	bool some_bool;
	float some_float;
	SomeOtherType some_other_type;
} SomeStruct;

typedef enum {
	Red,
	Green,
	Blue,
} Color;

typedef struct __attribute__((packed)) {
	int8_t repr;
	union {
		Color color;
		SomeStruct some_struct;
	};
} SomeUnion;

