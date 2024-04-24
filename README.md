# wtx_tools
tools for encoding and generating witness textures

Currently this consists of an `encoder` program, that will take some image and convert it to a .wtx file.
it has a few optional arguments, such as whether to also generate mipmaps, or what format to use for encoding
```bash
./encoder image.png
# creates image.wtx
```

This code also generates a library for use in C/C++ code. 
The library exposes a few main functions at the moment, see the header file `./cpp/wtx_tools.h` 
`image_to_wtx()` takes some bytes representing an image, and generates a wtx similar to the `encoder` binary, returning those bytes to the C code.

Theres also a more experimental `generate_desert_spec_wtx()` which will attempt to generate a desert-puzzle specmap from scratch. It takes arguments representing the position of the puzzle solution. The arguments are not very elegant at the moment. It can change to something more randomizer-friendly if necessary.

Additionally, there is also code for generating color-bunker textures, on some background textures that mostly-match the game's vanilla textures. Example code for that can be found [in this fork of the Witness Archipelago Randomizer](https://github.com/n-elderbroom/The-Witness-Randomizer-for-Archipelago/blob/bfacaebe1e4369cfa64c71ec21425d97abad7cde/Source/TextureLoader.cpp#L8). It takes the randomziers' generated puzzle data and returns a texture.

see `./cpp/test.cpp` for examples
