#include <stdio.h>
#include "wtx_tools.h"
#include <fstream>
using namespace std;

int main() {
    TextureBuffer x = generate_desert_spec_wtx("TopLeft Top Center Bottom BottomLeft BottomLeftEnd");
    printf("got %lu bytes wtx file from rust.\n", x.len);
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

