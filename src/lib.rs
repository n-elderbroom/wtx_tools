use std::ffi::c_char;
use libc::c_float;
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


///mirrors a point on a panel of dimensions width and height.
///value of `symmetry` matches the enum in the randomizer. (except pillar symmetry options)
fn get_mirrored_point (point: (f32,f32), symmetry: i32, width: f32, height: f32) -> Option<(f32,f32)> {
    let (x,y) = point;

    match symmetry {
        0=> None,
        1=> Some((x, height - y)), //horizontal
        2=> Some((width -x, y)), //vertical
        3=> Some((width -x, height - y)), //rotational 180
        4=> Some((y, width - x)), //rotate left
        5=> Some((height - y, x)), //rotate right
        6=> Some((y,x)), //flip x/y
        7=> Some((height - y, width - x)), //flip neg x/y
        8=> Some((x, (y + (height/2.0) ) % height)), //parallel horizontal
        9=> Some(((x + (width / 2.0)) % width  ,y)), //parallel vertical
        10=> Some((width -x, (y + (height/2.0) ) % height)), //paralel horizontal, flipped
        11=> Some(((x + (width / 2.0)) % width  , height -y)), //parallel vertical, flipped

        // case Symmetry::PillarParallel: return Point(x + _width / 2, y);
		// case Symmetry::PillarHorizontal: return Point(x + _width / 2, _height - 1 - y);
		// case Symmetry::PillarVertical: return Point( _width / 2 - x, y);
		// case Symmetry::PillarRotational: return Point(_width / 2 - x, _height - 1 - y);


        _=>None,
    }

}

#[no_mangle]
pub extern "C" fn generate_desert_spec_line_2(xpoints: *const f32, ypoints: *const f32, numpoints: size_t, xpoints2: *const f32, ypoints2: *const f32, numpoints2: size_t, thickness : c_float) -> TextureBuffer {
    let x_vec = unsafe {
        assert!(!xpoints.is_null());

        slice::from_raw_parts(xpoints, numpoints)
    };
    let y_vec = unsafe {
        assert!(!ypoints.is_null());

        slice::from_raw_parts(ypoints, numpoints)
    };
    let points : Vec<(f32, f32)> = std::iter::zip(x_vec, y_vec).map(|x| (x.0.clone(), x.1.clone())).collect();

    let x_vec_2 = unsafe {
        assert!(!xpoints2.is_null());

        slice::from_raw_parts(xpoints2, numpoints2)
    };
    let y_vec_2 = unsafe {
        assert!(!ypoints2.is_null());

        slice::from_raw_parts(ypoints2, numpoints2)
    };

    let points_2 : Vec<(f32, f32)> = std::iter::zip(x_vec_2, y_vec_2).map(|x| (x.0.clone(), x.1.clone())).collect();

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = generate_desert_spec_line_img(points, thickness, 0);

    let img2 = draw_line_on_image(img, points_2, thickness);
    

    let mut buf = generate_wtx_from_image(img2, true, WtxFormat::DXT1, 0x05); 
    let data = buf.as_mut_ptr();
    let len = buf.len();
    std::mem::forget(buf);
    TextureBuffer { data, len }
}

#[no_mangle]
///Generates an arbitrary spec map with a line pattern according to an array of x/y points with symmetry.
///generated images are 512x512 squares.
///symmetry is an `int` corresponding to the randomizers' existing Symmetry enum.
pub extern "C" fn generate_desert_spec_line_sym(xpoints: *const f32, ypoints: *const f32, numpoints: size_t, thickness : c_float, symmetry : i32) -> TextureBuffer {
    let x_vec = unsafe {
        assert!(!xpoints.is_null());

        slice::from_raw_parts(xpoints, numpoints)
    };
    let y_vec = unsafe {
        assert!(!ypoints.is_null());

        slice::from_raw_parts(ypoints, numpoints)
    };

    let points : Vec<(f32, f32)> = std::iter::zip(x_vec, y_vec).map(|x| (x.0.clone(), x.1.clone())).collect();
    println!("got some points and didnt panic doing things with them");
    for p in &points {
        println!("point {:?}", p);
    }
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = generate_desert_spec_line_img(points, thickness, symmetry);
    // let mut buf = generate_desert_spec_hexagon_wtx(inst).into_boxed_slice();
    let mut buf = generate_wtx_from_image(img, true, WtxFormat::DXT1, 0x05); 
    let data = buf.as_mut_ptr();
    let len = buf.len();
    std::mem::forget(buf);
    TextureBuffer { data, len }
}



#[no_mangle]
///Generates an arbitrary spec map with a line pattern according to an array of x/y points.
///generated images are 512x512 squares 
pub extern "C" fn generate_desert_spec_line(xpoints: *const f32, ypoints: *const f32, numpoints: size_t, thickness : c_float) -> TextureBuffer {
    generate_desert_spec_line_sym(xpoints, ypoints, numpoints, thickness, 0)
}

fn generate_desert_spec_line_img(points : Vec<(f32,f32)>, thickness : f32, symmetry : i32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    
    let bg_img_bytes = include_bytes!("images/desertspecpanel_square_bg.png");
    let bg_img: ImageBuffer<Rgba<u8>, Vec<u8>> = image::load_from_memory(bg_img_bytes).unwrap().to_rgba8();


    let mut img_of_line = draw_line_on_image(bg_img, points.clone(), thickness);
    if symmetry != 0 {
        let mirrored_points : Vec<(f32,f32)>  = points.iter().map(|x| get_mirrored_point(*x, symmetry, 512.0, 512.0).unwrap()).collect();
        let mirrored_line_img = draw_line_on_image(img_of_line, mirrored_points, thickness);
        img_of_line = mirrored_line_img;
    }
    // bg_img.save("./genimg.png").unwrap(); //debug preview
    println!("[Rust]: generated a desert spec map");
    img_of_line
}

//draw a line with dot on an image surface.
//TODO refactor more code to re-use this
fn draw_line_on_image(bg_img: ImageBuffer<Rgba<u8>, Vec<u8>>, points : Vec<(f32,f32)>, thickness : c_float) -> ImageBuffer<Rgba<u8>, Vec<u8>>{

    let mut dt = DrawTarget::new(512, 512);
    let mut pb = PathBuilder::new();

    let scaledpoints : Vec<(f32,f32)> = points.into_iter().map(|x| (x.0 * 512.0, x.1 * 512.0)).collect();


    pb.move_to(scaledpoints[0].0, scaledpoints[0].1);
    for point in &scaledpoints.as_slice()[1..] {
        pb.line_to(point.0, point.1)
    }
    let path = pb.finish();
    
    //now prepare the dot bit
    pb = PathBuilder::new();
    pb.move_to(scaledpoints[0].0, scaledpoints[0].1);
    pb.line_to(scaledpoints[0].0, scaledpoints[0].1);
    pb.arc(scaledpoints[0].0, scaledpoints[0].1, 0.5*thickness, 0., 360.);
    let dotpath: Path = pb.finish();
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
            width: thickness,
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
            width: thickness,
            miter_limit: 2.,
            dash_array: vec![50., 0.],
            dash_offset: 0.,
        },
        &DrawOptions::new(),
    );

    let img_of_line = ImageBuffer::from_raw(512,512,dt.get_data_u8().to_vec()).unwrap();
    let blurred: ImageBuffer<Rgba<u8>, Vec<u8>> = image::imageops::blur(&img_of_line, 5.);
    let mut new_img = bg_img.clone();
    image::imageops::overlay(&mut new_img, &blurred, 0, 0);
    // new_img.save("./genimg_2.png").unwrap(); //debug preview

    new_img
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

