# wtx_tools
tools for encoding and generating witness textures

Currently this consists of an `encoder` program, that will take some image and convert it to a .wtx file.
it has a few optional arguments, such as whether to also generate mipmaps, or what format to use for encoding
```bash
./encoder image.png
# creates image.wtx
```

This code also generates a library for use in C/C++ code. 
The library exposes two main functions at the moment. `image_to_wtx()` takes some bytes representing an image, and generates a wtx similar to the `encoder` binary, returning those bytes to the C code.
Theres also a more experimental `generate_desert_spec_wtx()` which will attempt to generate a desert-puzzle specmap from scratch. It takes arguments representing the position of the puzzle solution.

see `./cpp/test.cpp` for examples
