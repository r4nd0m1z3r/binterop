// Compile with wasi-sdk like this:
// $WASI_SDK_PATH/bin/clang -Wl,--no-entry -mexec-model=reactor -o guest.wasm lib.c

#include "../../schemas/main.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

__attribute__((export_name("alloc"))) uint8_t* alloc(uint32_t len)
{
    return malloc(len);
}

__attribute__((export_name("dealloc"))) void dealloc(uint8_t* ptr)
{
    free(ptr);
}

__attribute__((export_name("process_data"))) GuestToHost*
process_data(HostToGuest* host_to_guest_ptr)
{
    HostToGuest host_to_guest = *host_to_guest_ptr;

    fprintf(stderr, "Got data from host: (a: %c, b: %f, c: %f)\n",
        host_to_guest.a, host_to_guest.b, host_to_guest.c);

    Vectoru8 msg = Vectoru8_new(128);
    sprintf((char*)msg.ptr, "Char: %c | %f + %f = %f", host_to_guest.a,
        host_to_guest.b, host_to_guest.c, host_to_guest.b + host_to_guest.c);

    GuestToHost* output = (GuestToHost*)alloc(sizeof(GuestToHost));
    memcpy(output, &(GuestToHost) { msg }, sizeof(GuestToHost));

    return output;
}
