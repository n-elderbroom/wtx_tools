#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


enum class ColorPanelBackground {
  Blueprint,
  White,
  LightGrey,
  DarkGrey,
  Elevator,
};

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

TextureBuffer generate_tricolor_panel_3x3_wtx(WtxPuzzle3x3 grid, ColorPanelBackground background);

TextureBuffer image_to_wtx(ImgFileBuffer image, bool gen_mipmaps, WtxFormat format, uint8_t bits);

TextureBuffer wtx_tools_generate_colorpanel_from_grid(const uint32_t *grid,
                                                      size_t width,
                                                      size_t height,
                                                      ColorPanelBackground bg);

} // extern "C"
