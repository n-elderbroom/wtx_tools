#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


struct TextureBuffer {
  uint8_t *data;
  size_t len;
};


extern "C" {

void free_texbuf(TextureBuffer buf);

TextureBuffer generate_desert_spec_wtx(const char *instructions);

} // extern "C"
