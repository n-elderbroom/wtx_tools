use std::ffi::CStr;
use std::ffi::c_char;
use color_eyre::owo_colors::colors::xterm::JungleMist;
use libc::size_t;
use std::slice;
use image::{ImageBuffer, Rgba, Pixel};
use raqote::*;



#[repr(C)]
/// C-and-Rust readable struct. Contains wtx-formatted texture.
pub struct TextureBuffer {
    data: *mut u8,
    len: usize,
}

#[repr(C)]
/// C-and-Rust readable struct. Contains an image, png/jpeg/etc, to be converted to a wtx texture.
/// image can be any format readable by rust's `image` crate.
pub struct ImgFileBuffer {
    data: *const c_char, //really u8 or i8. safe-ish to convert? but as c_char the C side won't complain about types
    len: usize,
}

#[repr(C)]
// TODO remove this
/// enum defining color of a 'stone'. Used internally.
#[derive(PartialEq, Clone, Copy)]
pub enum WtxColor {
    NoColor,

    TricolorWhite,
    TricolorPurple,
    TricolorGreen,

    TricolorNewWhite,
    TricolorNewPink,
    TricolorNewBlue,
    TricolorNewYellow,
}

#[repr(C)]
pub struct WtxPuzzle3x3 {
    grid: [WtxColor; 9],
}


#[repr(C)]
pub enum WtxFormat {
    DXT5,
    DXT1,
}


#[repr(C)]
#[derive(PartialEq)]
/// Enum used to decide which background to give a generated color-panel image
pub enum ColorPanelBackground {
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
}

/// Generates a complete 'wtx' file from a `_grid`, with background `bg`
/// The `*cost u32` in the arguments should pbe the start of a structure equivalent to  `_grid` from a `Panel`
/// It should be flattened to a contiguous array first, so that this rust code can read it.
/// Rust recalculates the size through the width and height. Width and height here is of the grid array - not
/// what you would probably consider the size of the puzzle. For a 3x3 puzzle for instance, thats (3*2 +1) in each dimension on the array, so 7x7.
#[no_mangle]
pub extern "C" fn wtx_tools_generate_colorpanel_from_grid(grid: *const u32, width: size_t, height:size_t, bg: ColorPanelBackground) -> TextureBuffer {
    let just_stones_vec = collect_stones_from_grid(grid, width, height);
    generate_tricolor_panel_wtx(just_stones_vec, bg, None)
}

/// This function is intended to be called by witness randomizer code
/// It is the same as `wtx_tools_generate_colorpanel_from_grid` but with an extra `id` argument.
/// this will save the generated image to disk as ./generated_{id}.png
#[no_mangle]
pub extern "C" fn wtx_tools_generate_colorpanel_from_grid_and_save(grid: *const u32, width: size_t, height:size_t, bg: ColorPanelBackground, id: i32) -> TextureBuffer {
    let just_stones_vec = collect_stones_from_grid(grid, width, height);
    generate_tricolor_panel_wtx(just_stones_vec, bg, Some(id))
}


fn collect_stones_from_grid(grid: *const u32, width: size_t, height:size_t) -> Vec<WtxColor>{
    let gridflat = unsafe {
        assert!(!grid.is_null());

        slice::from_raw_parts(grid, height as usize * width as usize)
    };
    // assert!(gridflat.len() == 49); //TODO panels larger than 3x3

    let mut grid = Vec::<Vec<u32>>::new();
    for i in 0..height {
        let mut row = Vec::<u32>::new();
        for j in 0..width {
            row.push(gridflat[i + j*height]) // this is the problem
        }
        grid.push(row);
    }
    //now we have rebuilt a nice vector for the whole grid
    let mut just_stones_vec = Vec::new(); //we want to ignore most of the grid - only look for the stones
    for (rownum, row) in grid.into_iter().enumerate() {
        for (colnum, cell) in row.into_iter().enumerate() {
            if (rownum % 2 != 0) && (colnum % 2 != 0) {
                //this is between two lines
                if cell & 0x100 > 0 {
                    //stone here
                    just_stones_vec.push(match cell & 0xF {
                        // 0x0 => WtxColor::NoColor,
                        0x2 => WtxColor::TricolorWhite,
                        0x4 => WtxColor::TricolorPurple,
                        0x5 => WtxColor::TricolorGreen,
                        // 0x6 => WtxColor::TricolorNewBlue, //CYAN
                        0x7 => WtxColor::TricolorNewPink,
                        0x8 => WtxColor::TricolorNewYellow,
                        0x9 => WtxColor::TricolorNewBlue,
                        _ => todo!() //panic if unknown color
                    })
                } else {
                    just_stones_vec.push(WtxColor::NoColor)
                }
            }
        }
    }
    just_stones_vec
}


///Internal function to generate Imagebuffer from a vec of colors
///This function takes shapes 3x3, 4x4, or 4x5
fn generate_colordots_panel(stones : Vec<WtxColor>, background: ColorPanelBackground, filename_id: Option<i32>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut dt: DrawTarget = DrawTarget::new(1024, 1024);

    let dot_coordinates = match stones.len() {
        9  => vec![(280.0,280.0),(512.0,280.0),(744.0,280.0),
                    (280.0,512.0),(512.0,512.0),(744.0,512.0),
                    (280.0,744.0),(512.0,744.0),(744.0,744.0)],
        16 => vec![(238.0,238.0),(421.0,238.0),(604.0,238.0),(787.0,238.0),
                    (238.0,421.0),(421.0,421.0),(604.0,421.0),(787.0,421.0),
                    (238.0,604.0),(421.0,604.0),(604.0,604.0),(787.0,604.0),
                    (238.0,787.0),(421.0,787.0),(604.0,787.0),(787.0,787.0),],
        20 => vec![(212.0,288.0),(362.0,288.0),(512.0,288.0),(662.0,288.0),(812.0,288.0),
                    (212.0,437.0),(362.0,437.0),(512.0,437.0),(662.0,437.0),(812.0,437.0),
                    (212.0,586.0),(362.0,586.0),(512.0,586.0),(662.0,586.0),(812.0,586.0),
                    (212.0,736.0),(362.0,736.0),(512.0,736.0),(662.0,736.0),(812.0,736.0)],
        _ => unimplemented!()
    };
    let scale = match stones.len() {
        9 => 32.0,
        16 => 30.0,
        20 => 20.0,
        _ => unimplemented!()
    };
    for (coords, color) in std::iter::zip(dot_coordinates, stones) {
        if color != WtxColor::NoColor {
            let realcolor = match color {
                WtxColor::TricolorWhite => SolidSource{r: 0xff, g: 0xff, b:0xff, a:0xFF},
                WtxColor::TricolorPurple => SolidSource{r: 0xa5, g: 0x51, b:0xff, a:0xFF},
                WtxColor::TricolorGreen => SolidSource{r: 0x6e, g: 0xab, b:0x5d, a:0xFF},
                WtxColor::TricolorNewWhite => SolidSource{r: 0xff, g: 0xff, b:0xff, a:0xFF},
                WtxColor::TricolorNewPink => SolidSource{r: 0xa4, g: 0x37, b:0xf0, a:0xFF},
                WtxColor::TricolorNewBlue => SolidSource{r: 0x00, g: 0xa8, b:0xe9, a:0xFF},
                WtxColor::TricolorNewYellow => SolidSource{r: 0xf9, g: 0xf8, b:0x45, a:0xFF},
                WtxColor::NoColor => unreachable!()
            };

            let mut pb = PathBuilder::new();
            pb.move_to(coords.0 - 20.0, coords.1 - 20.0);
            pb.line_to(coords.0 - 20.0, coords.1 + 20.0);
            pb.line_to(coords.0 + 20.0, coords.1 + 20.0);
            pb.line_to(coords.0 + 20.0, coords.1 - 20.0);
            pb.line_to(coords.0 - 20.0, coords.1 - 20.0);
            pb.close();
            let path = pb.finish();
            dt.fill(&path, &Source::Solid(realcolor), &DrawOptions::new());
            dt.stroke(&path, &Source::Solid(realcolor),&StrokeStyle {
                cap: LineCap::Round,
                join: LineJoin::Round,
                width: scale,
                miter_limit: 2.,
                dash_array: vec![50.0, 0.0],
                dash_offset: 0.0,
            }, &DrawOptions::new());
            // println!("[rust] placed a dot");
        }
    }
    let mut img_of_dots: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(1024,1024,dt.get_data_u8().to_vec()).unwrap();
    for pixel in img_of_dots.pixels_mut() {
        pixel.channels_mut().swap(0, 2); //fix pixel order
    }

    let bg_img_bytes: &[u8] = match background {
        ColorPanelBackground::Blueprint => include_bytes!("images/color_bunker_blueprint_bg.png"),
        ColorPanelBackground::White => include_bytes!("images/color_bunker_whitepaper.png"),
        ColorPanelBackground::LightGrey => include_bytes!("images/color_bunker_greyred_light.png"),
        ColorPanelBackground::DarkGrey => include_bytes!("images/color_bunker_greyred_dark.png"),
        ColorPanelBackground::Elevator => include_bytes!("images/color_bunker_elevator.png"),
    };
    let mut bg_img = image::load_from_memory(bg_img_bytes).unwrap().to_rgba8();
    image::imageops::overlay(&mut bg_img, &img_of_dots, 0, 0);

    if let Some(id) = filename_id {
        bg_img.save(format!("./generated_{:x}.png", id)).unwrap(); //save BEFORE we strip alpha channel
    }
    for pixel in bg_img.pixels_mut() {
        pixel.apply_with_alpha(|color| color, |_| 0);
    }
    // bg_img.save("/tmp/genimg.png").unwrap(); //debug preview
    println!("[Rust]: generated a colored dots panel");
    bg_img
}

#[no_mangle]
/// Converts ImgFileBuffer to a TextureBuffer containing an wtx-formatted image
pub extern "C" fn image_to_wtx(image : ImgFileBuffer, gen_mipmaps: bool, format: WtxFormat, bits: u8) -> TextureBuffer {
    let slice :&[u8] = unsafe { std::slice::from_raw_parts(image.data as *const u8, image.len)};
    let img = image::load_from_memory(slice).unwrap().to_rgba8();
    println!("[Rust]: Recieved image. ({:?} bytes, {:?}x{:?})", slice.len(), img.width(), img.height());
    let mut buf = generate_wtx_from_image(img, gen_mipmaps, format, bits);
    let data = buf.as_mut_ptr();
    let len = buf.len();
    std::mem::forget(buf);
    TextureBuffer { data, len }
}

#[no_mangle]
pub extern "C" fn generate_desert_spec_wtx(instructions : *const c_char) -> TextureBuffer {
    let inst = parse_instructions(instructions);
    let img = generate_desert_spec_hexagon_image(inst);
    // let mut buf = generate_desert_spec_hexagon_wtx(inst).into_boxed_slice();
    let mut buf = generate_wtx_from_image(img, true, WtxFormat::DXT1, 0x05); 
    let data = buf.as_mut_ptr();
    let len = buf.len();
    std::mem::forget(buf);
    TextureBuffer { data, len }
}

#[no_mangle]
/// Old function - to be removed. Generates only 3x3 grid, takes a struct containing array of 9 enums.
pub extern "C" fn generate_tricolor_panel_3x3_wtx(grid : WtxPuzzle3x3, background: ColorPanelBackground) -> TextureBuffer {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>>  = generate_colordots_panel_3x3(grid, background);

    let mut buf = generate_wtx_from_image(img, true, WtxFormat::DXT5, 0x01); 
    let data = buf.as_mut_ptr();
    let len = buf.len();
    std::mem::forget(buf);
    TextureBuffer { data, len }
}

fn generate_tricolor_panel_wtx(stoneslist: Vec<WtxColor>, background: ColorPanelBackground, filename_id : Option<i32>) -> TextureBuffer {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>>  = generate_colordots_panel(stoneslist, background, filename_id);

    let mut buf = generate_wtx_from_image(img, true, WtxFormat::DXT5, 0x01); 
    let data = buf.as_mut_ptr();
    let len = buf.len();
    std::mem::forget(buf);
    TextureBuffer { data, len }
}




#[no_mangle]
/// Call this to free a rust-allocated TextureBuffer
/// Rust will keep track of memory it allocated and must be informed to free it.
pub extern "C" fn free_texbuf(buf: TextureBuffer) {
    let s = unsafe { std::slice::from_raw_parts_mut(buf.data, buf.len) };
    let s = s.as_mut_ptr();
    unsafe {
        drop(Box::from_raw(s));
    }
}

fn parse_instructions(instructions : *const c_char) -> Vec<u8> {
    let cstr = unsafe { CStr::from_ptr(instructions) };
    // Get copy-on-write Cow<'_, str>, then guarantee a freshly-owned String allocation
    let instr_string : String = String::from_utf8_lossy(cstr.to_bytes()).to_string();
    // println!("got strings {:?}",&instr_string);
    let mut instr_vec = Vec::new();
    for substring in instr_string.split(" ") {
        // println!("Substring is '{:?}'", substring);
        //      8
        //  7   1   9
        //    0   2 
        //      6
        //    5   3
        //  12  4  10
        //      11
        instr_vec.push(match substring {
            "TopLeft" => 0,
            "Top" => 1,
            "TopRight" => 2,
            "BottomRight" => 3,
            "Bottom" => 4,
            "BottomLeft" => 5,
            "Center" => 6,
            "TopLeftEnd" => 7,
            "TopEnd" => 8,
            "TopRightEnd" => 9,
            "BottomRightEnd" => 10,
            "BottomEnd" => 11,
            "BottomLeftEnd" => 12,
            _ => panic!("unexpected isntruction string"),
        });
    };

    instr_vec
}

fn generate_desert_spec_hexagon_image<'a>(points: Vec<u8>) -> ImageBuffer<Rgba<u8>, Vec<u8>> { //i dont like this static lifetime
    let r = 256. - 90.;
    let half_side_length = r / 3_f32.sqrt();
    let long_r = 2. * half_side_length;
    let r_2 = 256. - 40.;
    let half_side_length_2 = r_2 / 3_f32.sqrt();
    let long_r_2 = 2. * half_side_length_2;
    let (center_x, center_y) = (260., 256.);
    let coordinates: [(f32, f32); 13] = [
        (center_x - r, center_y - half_side_length),
        (center_x, center_y - long_r),
        (center_x + r, center_y - half_side_length),
        (center_x + r, center_y + half_side_length),
        (center_x, center_y + long_r),
        (center_x - r, center_y + half_side_length),
        (center_x + 0., center_y + 0.),
        (center_x - r_2, center_y - half_side_length_2),
        (center_x, center_y - long_r_2),
        (center_x + r_2, center_y - half_side_length_2),
        (center_x + r_2, center_y + half_side_length_2),
        (center_x, center_y + long_r_2),
        (center_x - r_2, center_y + half_side_length_2),
    ];

    let mut dt = DrawTarget::new(512, 512);
    let mut pb = PathBuilder::new();

    pb.move_to(
        coordinates[points.as_slice()[0] as usize].0,
        coordinates[points.as_slice()[0] as usize].1,
    );
    for node in &points.as_slice()[1..] {
        pb.line_to(coordinates[*node as usize].0, coordinates[*node as usize].1);
    }
    // pb.line_to(260., 512-center_ege_distance.); bottom center
    let path = pb.finish();

    // let bg_img_bytes = include_bytes!("desertspecpanel_square_bg.png");
    // let bg_img = image::load_from_memory(bg_img_bytes).unwrap().to_rgba;
   
    pb = PathBuilder::new();
    pb.move_to(
        coordinates[points.as_slice()[0] as usize].0,
        coordinates[points.as_slice()[0] as usize].1,
    );
    pb.line_to(
        coordinates[points.as_slice()[0] as usize].0,
        coordinates[points.as_slice()[0] as usize].1,
    );
    pb.arc(
        coordinates[points.as_slice()[0] as usize].0,
        coordinates[points.as_slice()[0] as usize].1,
        20.,
        0.,
        180.,
    );
    let dotpath = pb.finish();
    dt.stroke(
        &dotpath,
        &Source::Solid(SolidSource {
            r: 0x00,
            g: 0x00,
            b: 0x00,
            a: 0xff,
        }),
        &StrokeStyle {
            cap: LineCap::Round,
            join: LineJoin::Round,
            width: 30.,
            miter_limit: 0.,
            dash_array: vec![50., 0.],
            dash_offset: 0.,
        },
        &DrawOptions::new(),
    );
    dt.stroke(
        &path,
        &Source::Solid(SolidSource {
            r: 0x00,
            g: 0x00,
            b: 0x00,
            a: 0xff,
        }),
        &StrokeStyle {
            cap: LineCap::Round,
            join: LineJoin::Round,
            width: 30.,
            miter_limit: 2.,
            dash_array: vec![50., 0.],
            dash_offset: 0.,
        },
        &DrawOptions::new(),
    );


    
    let img_of_line = ImageBuffer::from_raw(512,512,dt.get_data_u8().to_vec()).unwrap();
    let blurred = image::imageops::blur(&img_of_line, 5.);

    let bg_img_bytes = include_bytes!("images/desertspecpanel_square_bg.png");
    let mut bg_img = image::load_from_memory(bg_img_bytes).unwrap().to_rgba8();
    image::imageops::overlay(&mut bg_img, &blurred, 0, 0);
    //  bg_img.save("/tmp/genimg.png").unwrap(); //debug preview
    println!("[Rust]: generated a desert spec map");
    bg_img
}

fn generate_colordots_panel_3x3(grid: WtxPuzzle3x3, background: ColorPanelBackground) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut dt: DrawTarget = DrawTarget::new(1024, 1024);

    let dot_coordinates = vec![(280.0,280.0),(512.0,280.0),(744.0,280.0),
                                (280.0,512.0),(512.0,512.0),(744.0,512.0),
                                (280.0,744.0),(512.0,744.0),(744.0,744.0)];
    for (coords, color) in std::iter::zip(dot_coordinates,grid.grid) {
        if color != WtxColor::NoColor {
            let realcolor = match color {
                WtxColor::TricolorWhite => SolidSource{r: 0xff, g: 0xff, b:0xff, a:0xFF},
                WtxColor::TricolorPurple => SolidSource{r: 0xa5, g: 0x51, b:0xff, a:0xFF},
                WtxColor::TricolorGreen => SolidSource{r: 0x6e, g: 0xab, b:0x5d, a:0xFF},
                WtxColor::TricolorNewWhite => SolidSource{r: 0xff, g: 0xff, b:0xff, a:0xFF},
                WtxColor::TricolorNewPink => SolidSource{r: 0xa4, g: 0x37, b:0xf0, a:0xFF},
                WtxColor::TricolorNewBlue => SolidSource{r: 0x00, g: 0xa8, b:0xe9, a:0xFF},
                WtxColor::TricolorNewYellow => SolidSource{r: 0xf9, g: 0xf8, b:0x45, a:0xFF},
                WtxColor::NoColor => unreachable!()
            };

            let mut pb = PathBuilder::new();
            pb.move_to(coords.0 - 20.0, coords.1 - 20.0);
            pb.line_to(coords.0 - 20.0, coords.1 + 20.0);
            pb.line_to(coords.0 + 20.0, coords.1 + 20.0);
            pb.line_to(coords.0 + 20.0, coords.1 - 20.0);
            pb.line_to(coords.0 - 20.0, coords.1 - 20.0);
            pb.close();
            let path = pb.finish();
            dt.fill(&path, &Source::Solid(realcolor), &DrawOptions::new());
            dt.stroke(&path, &Source::Solid(realcolor),&StrokeStyle {
                cap: LineCap::Round,
                join: LineJoin::Round,
                width: 32.,
                miter_limit: 2.,
                dash_array: vec![50.0, 0.0],
                dash_offset: 0.0,
            }, &DrawOptions::new());
            // println!("[rust] placed a dot");
        }
    }

    let mut img_of_dots: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(1024,1024,dt.get_data_u8().to_vec()).unwrap();
    // img_of_dots.save("/tmp/genimgdots.png").unwrap(); //debug preview
    for pixel in img_of_dots.pixels_mut() {
        pixel.channels_mut().swap(0, 2); //fix pixel order
    }
    
    let bg_img_bytes: &[u8] = match background {
        ColorPanelBackground::Blueprint => include_bytes!("images/color_bunker_blueprint_bg.png"),
        ColorPanelBackground::White => include_bytes!("images/color_bunker_whitepaper.png"),
        ColorPanelBackground::LightGrey => include_bytes!("images/color_bunker_greyred_light.png"),
        ColorPanelBackground::DarkGrey => include_bytes!("images/color_bunker_greyred_dark.png"),
        ColorPanelBackground::Elevator => include_bytes!("images/color_bunker_elevator.png"),
    };
    let mut bg_img = image::load_from_memory(bg_img_bytes).unwrap().to_rgba8();
    image::imageops::overlay(&mut bg_img, &img_of_dots, 0, 0);

    
    for pixel in bg_img.pixels_mut() {
        pixel.apply_with_alpha(|color| color, |_| 0);
    }
    // bg_img.save("/tmp/genimg.png").unwrap(); //debug preview

    println!("[Rust]: generated a colored dots panel");
    bg_img
}


pub fn generate_wtx_from_image(mut img: ImageBuffer<Rgba<u8>, Vec<u8>>, gen_mipmaps: bool, format: WtxFormat, bits: u8) -> Vec<u8> {
    image::imageops::flip_vertical_in_place(&mut img);
    let mut r_amt = 0.;
    let mut g_amt = 0.;
    let mut b_amt = 0.;
    let mut a_amt = 0.;

    for p in img.pixels() {
        r_amt += p[0] as f64 / 255.;
        g_amt += p[1] as f64 / 255.;
        b_amt += p[2] as f64 / 255.;
        a_amt += p[3] as f64 / 255.;
    };
    r_amt /= img.pixels().len() as f64;
    g_amt /= img.pixels().len() as f64;
    b_amt /= img.pixels().len() as f64;
    a_amt /= img.pixels().len() as f64;

    //downcast floats to f32
    let (r_float,g_float,b_float, _a_float) = (r_amt as f32, g_amt as f32, b_amt as f32, a_amt as f32);
    let bytes_r = r_float.to_le_bytes().to_vec();
    let bytes_g = g_float.to_le_bytes().to_vec();
    let bytes_b = b_float.to_le_bytes().to_vec();
    // let mut bytes_a = a_float.to_le_bytes().to_vec();
    let bytes_a = 1.0_f32.to_le_bytes().to_vec(); //hardcoded b/c i dont want rounding errors


    let mipmaps = match gen_mipmaps {
        true => image_dds::Mipmaps::GeneratedAutomatic,
        false => image_dds::Mipmaps::GeneratedExact(1),
    };
    let img_format = match format {
        WtxFormat::DXT5 => image_dds::ImageFormat::BC3RgbaUnorm,
        WtxFormat::DXT1 => image_dds::ImageFormat::BC1RgbaUnorm,
    };
    
    let image_dds = image_dds::dds_from_image(
        &img,
        img_format,
        image_dds::Quality::Fast,
        mipmaps,
        // image_dds::Mipmaps::GeneratedAutomatic, //generate mipmaps
    ).unwrap();
    
    
    //Create the WTX header
    let mut wtx_data = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, //header always constant
        0x00, 0x00, 0x00, 0x00, //these 4 = length of rest of data
        0x00, 0x00, //width 
        0x00, 0x00, //height
        0x01, 0x00, //"depth" always (0x01, 0x00) as these are 2d images
        0x00, 0x00, //number of mipmaps 
        bits, //weird bitmask thing
        0x00, 0x00, 0x00, //these 3 always zero (part of bitmask but always 0?)
        0x00,0x00,0x00,0x00, //float R
        0x00,0x00,0x00,0x00, //float G
        0x00,0x00,0x00,0x00, //float B
        0x00,0x00,0x00,0x00, //float A
        0x00,0x00,0x00,0x00, //image format
    ];
    wtx_data.splice(8..12, (image_dds.data.len() as u32 + 32_u32).to_le_bytes().to_vec());
    wtx_data.splice(12..14, (img.width() as u16).to_le_bytes().to_vec());
    wtx_data.splice(14..16, (img.height() as u16).to_le_bytes().to_vec());
    wtx_data.splice(18..20, (image_dds.get_num_mipmap_levels() as u16).to_le_bytes().to_vec());
    
    wtx_data.splice(24..28, bytes_r);
    wtx_data.splice(28..32, bytes_g);
    wtx_data.splice(32..36, bytes_b);
    wtx_data.splice(36..40, bytes_a);
    let format_id = match format {
        WtxFormat::DXT5 => vec![0x44, 0x58, 0x54, 0x35,],
        WtxFormat::DXT1 => vec![0x44, 0x58, 0x54, 0x31,],
    };
    wtx_data.splice(40..44, format_id);
    
    // println!("rgba floats are {:?}",[r_float, g_float, b_float, a_float]);
    let mut data = image_dds.get_data(0).unwrap().to_vec();
    wtx_data.append(&mut data);

    println!("[Rust]: generated a custom texture.",);
    wtx_data    
}

