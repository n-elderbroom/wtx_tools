use std::ffi::CStr;
use std::ffi::c_char;
use image::{ImageBuffer, Rgba};
use raqote::*;

#[repr(C)]
pub struct TextureBuffer {
    data: *mut u8,
    len: usize,
}


#[no_mangle]
pub extern "C" fn generate_desert_spec_wtx(instructions : *const c_char) -> TextureBuffer {
    let inst = parse_instructions(instructions);
    let mut buf = generate_wtx_bytes(inst).into_boxed_slice();
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

fn generate_image<'a>(points: Vec<u8>) -> ImageBuffer<Rgba<u8>, Vec<u8>> { //i dont like this static lifetime
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
     bg_img
}

fn generate_wtx_bytes(instructions: Vec<u8>) -> Vec<u8> {
    let mut img :ImageBuffer<Rgba<u8>, Vec<u8>> =generate_image(instructions);
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


    let image_dds = image_dds::dds_from_image(
        &img,
        image_dds::ImageFormat::BC1Unorm,
        image_dds::Quality::Fast,
        image_dds::Mipmaps::GeneratedExact(10),
        // image_dds::Mipmaps::GeneratedAutomatic, //generate mipmaps
    ).unwrap();
    
    
    //This header basically says
    //format is DXT1 / BC1
    //file is 512x512
    //file has 10 mipmaps
    //also some floats saying R/G/B/A amounts. for now its (1/3) for R/G/B and 1 for A,
    //idk if we need to calculate those? does game use that info for anything?
    let mut wtx_data = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0xd8, 0xaa, 0x02, 0x00, 0x00, 0x02, 0x00,
        0x02, 0x01, 0x00, 0x0a, 0x00, 0x05, 0x00, 0x00, 0x00,
    ];

    //downcast floats to f32
    let (r_float,g_float,b_float, _a_float) = (r_amt as f32, g_amt as f32, b_amt as f32, a_amt as f32);

    let mut format_id = vec![0x44, 0x58, 0x54, 0x31,];
    let mut bytes_r = r_float.to_le_bytes().to_vec();
    let mut bytes_g = g_float.to_le_bytes().to_vec();
    let mut bytes_b = b_float.to_le_bytes().to_vec();
    // let mut bytes_a = a_float.to_le_bytes().to_vec();
    let mut bytes_a = 1.0_f32.to_le_bytes().to_vec(); //hardcoded b/c i dont want rounding errors

    // println!("rgba floats are {:?}",[r_float, g_float, b_float, a_float]);
    let mut data = image_dds.get_data(0).unwrap().to_vec();
    wtx_data.append(&mut bytes_r);
    wtx_data.append(&mut bytes_g);
    wtx_data.append(&mut bytes_b);
    wtx_data.append(&mut bytes_a);
    wtx_data.append(&mut format_id);
    wtx_data.append(&mut data);
    
    println!("[Rust]:generated a custom texture. (Desert puzzle spec map)",);
    wtx_data    
}

