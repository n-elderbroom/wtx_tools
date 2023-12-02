use std::ffi::CStr;
use std::ffi::c_char;
use image::{ImageBuffer, Rgba};
use raqote::*;



#[repr(C)]
pub struct TextureBuffer {
    data: *mut u8,
    len: usize,
}

#[repr(C)]
pub struct ImgFileBuffer {
    data: *const c_char, //really u8 or i8. safe-ish to convert? but as c_char the C side won't complain about types
    len: usize,
}


#[repr(C)]
pub enum WtxFormat {
    DXT5,
    DXT1,
}

#[no_mangle]
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
            "TopLetEnd" => 7,
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

    let bg_img_bytes = include_bytes!("desertspecpanel_square_bg.png");
    let mut bg_img = image::load_from_memory(bg_img_bytes).unwrap().to_rgba8();
    image::imageops::overlay(&mut bg_img, &blurred, 0, 0);
    //  bg_img.save("/tmp/genimg.png").unwrap(); //debug preview
    println!("[Rust]: generated a desert spec map");
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
        WtxFormat::DXT5 => image_dds::ImageFormat::BC3Srgb,
        WtxFormat::DXT1 => image_dds::ImageFormat::BC3Srgb,
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

