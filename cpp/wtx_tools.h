#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


enum class WtxColor {
  NoColor,
  TricolorWhite,
  TricolorPurple,
  TricolorGreen,
  TricolorNewWhite,
  TricolorNewPink,
  TricolorNewBlue,
  TricolorNewYellow,
};

enum class WtxFormat {
  DXT5,
  DXT1,
};

struct TextureBuffer {
  uint8_t *data;
  size_t len;
};

struct WtxPuzzle3x3 {
  WtxColor grid[9];
};

struct ImgFileBuffer {
  const char *data;
  size_t len;
};


extern "C" {

void free_texbuf(TextureBuffer buf);

TextureBuffer generate_desert_spec_wtx(const char *instructions);

TextureBuffer generate_tricolor_panel_wtx(WtxPuzzle3x3 grid);

TextureBuffer image_to_wtx(ImgFileBuffer image, bool gen_mipmaps, WtxFormat format, uint8_t bits);

} // extern "C"
