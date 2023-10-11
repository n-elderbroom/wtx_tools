#include <stdio.h>
#include "wtx_tools.h"

int main() {
    TextureBuffer x = generate_desert_spec_wtx("TopLeft Top Center Bottom BottomLeft BottomLeftEnd");
    printf("got %lu bytes wtx file from rust.\n", x.len);
    free_texbuf(x); //rust lib has to be the one to call free() on that memory?
}

