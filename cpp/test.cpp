#include <stdio.h>
#include "wtx_tools.h"
#include <fstream>
using namespace std;

int main() {
    
    WtxPuzzle3x3 puzzle = { {
                WtxColor::NoColor,              WtxColor::TricolorGreen,    WtxColor::NoColor,
                WtxColor::TricolorPurple,       WtxColor::NoColor,          WtxColor::TricolorGreen,
                WtxColor::TricolorWhite,        WtxColor::TricolorPurple,   WtxColor::NoColor} };

    TextureBuffer x = generate_tricolor_panel_3x3_wtx(puzzle, ColorPanelBackground::Blueprint);


    
    // TextureBuffer x = generate_desert_spec_wtx("Center Top TopLeft BottomLeft Bottom BottomRight TopRight TopRightEnd");
    
    printf("got %lu bytes wtx file from rust.\n", x.len);

    ofstream outputBuffer("./color_panel_custom.wtx", ios::out | ios::binary);
    outputBuffer.write((const char *) x.data, x.len);
    // outputBuffer.
    outputBuffer.close();


    free_texbuf(x); //rust lib has to be the one to call free() on that memory?


    ifstream fileBuffer("./Cursor_1_Happy_right.png", ios::ate | ios::binary);

    if (fileBuffer.is_open()) {
        std::streamsize length = fileBuffer.tellg();
        printf("opened image file, it has %ld length\n", length);
        fileBuffer.seekg(0, std::ios::beg);
        char * buffer = new char [length];
        fileBuffer.read (buffer,length);
        ImgFileBuffer inputfile;
        inputfile.data = buffer;
        inputfile.len = length;

        TextureBuffer y = image_to_wtx(inputfile, true, WtxFormat::DXT1, 0x05);
        printf("got %lu bytes wtx file from rust.\n", y.len);
        free_texbuf(y); //rust lib has to be the one to call free() on that memory?

    }   
    
}

