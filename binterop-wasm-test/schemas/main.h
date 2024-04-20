#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

typedef struct {
	uint8_t a;
	double b;
	double c;
} HostToGuest;

typedef struct {
	uint8_t* ptr;
	uint64_t len;
	uint64_t capacity;
} Vectoru8;

typedef struct {
	Vectoru8 msg;
} GuestToHost;

static inline Vectoru8 Vectoru8_new(uint64_t len) {
                    return (Vectoru8){ (uint8_t*)calloc(len, sizeof(uint8_t)), len, len };
                }

                static inline void Vectoru8_resize(Vectoru8* array, uint64_t new_len) {
                    array->ptr = realloc(array->ptr, sizeof(uint8_t) * new_len);
                    array->len = new_len;
                    array->capacity = new_len;
                }
