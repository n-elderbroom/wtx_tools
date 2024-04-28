#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


/// Enum used to decide which background to give a generated color-panel image
enum class ColorPanelBackground {
  /// used on introductory puzzles
  Blueprint,
  /// used in the 2 shipping container puzzles
  White,
  /// First 2 color-filter puzzles
  LightGrey,
  /// third color-filter puzzle. Slightly darker gray
  DarkGrey,
  /// only used on elevator
  Elevator,
};

enum class WtxFormat {
  DXT5,
  DXT1,
};

/// C-and-Rust readable struct. Contains wtx-formatted texture.
struct TextureBuffer {
  uint8_t *data;
  size_t len;
};

/// C-and-Rust readable struct. Contains an image, png/jpeg/etc, to be converted to a wtx texture.
/// image can be any format readable by rust's `image` crate.
struct ImgFileBuffer {
  const char *data;
  size_t len;
};


extern "C" {

/// Call this to free a rust-allocated TextureBuffer
/// Rust will keep track of memory it allocated and must be informed to free it.
void free_texbuf(TextureBuffer buf);

///Generates an arbitrary spec map with a line pattern according to an array of x/y points.
///generated images are 512x512 squares
TextureBuffer generate_desert_spec_line(const float *xpoints,
                                        const float *ypoints,
                                        size_t numpoints,
                                        float thickness);

TextureBuffer generate_desert_spec_line_2(const float *xpoints,
                                          const float *ypoints,
                                          size_t numpoints,
                                          const float *xpoints2,
                                          const float *ypoints2,
                                          size_t numpoints2,
                                          float thickness);

///Generates an arbitrary spec map with a line pattern according to an array of x/y points with symmetry.
///generated images are 512x512 squares.
///symmetry is an `int` corresponding to the randomizers' existing Symmetry enum.
TextureBuffer generate_desert_spec_line_sym(const float *xpoints,
                                            const float *ypoints,
                                            size_t numpoints,
                                            float thickness,
                                            int32_t symmetry);

/// Converts ImgFileBuffer to a TextureBuffer containing an wtx-formatted image
TextureBuffer image_to_wtx(ImgFileBuffer image, bool gen_mipmaps, WtxFormat format, uint8_t bits);

/// Generates a complete 'wtx' file from a `_grid`, with background `bg`
/// The `*cost u32` in the arguments should pbe the start of a structure equivalent to  `_grid` from a `Panel`
/// It should be flattened to a contiguous array first, so that this rust code can read it.
/// Rust recalculates the size through the width and height. Width and height here is of the grid array - not
/// what you would probably consider the size of the puzzle. For a 3x3 puzzle for instance, thats (3*2 +1) in each dimension on the array, so 7x7.
TextureBuffer wtx_tools_generate_colorpanel_from_grid(const uint32_t *grid,
                                                      size_t width,
                                                      size_t height,
                                                      ColorPanelBackground bg);

/// This function is intended to be called by witness randomizer code
/// It is the same as `wtx_tools_generate_colorpanel_from_grid` but with an extra `id` argument.
/// this will save the generated image to disk as ./generated_{id}.png
TextureBuffer wtx_tools_generate_colorpanel_from_grid_and_save(const uint32_t *grid,
                                                               size_t width,
                                                               size_t height,
                                                               ColorPanelBackground bg,
                                                               int32_t id);

} // extern "C"
