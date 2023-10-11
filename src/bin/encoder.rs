use clap::Parser;
use color_eyre::eyre::Result;
use image::io::Reader as ImageReader;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(
        long,
        default_value_t = false,
        help = "include mipmaps. will auto generate all possible sizes"
    )]
    include_mipmaps: bool,
    #[arg(
        long,
        default_value_t = String::from("DXT5"),
        help = "format (either DXT5 or DXT1)"
    )]
    format: String,
    #[arg(
        long,
        default_value_t = 0,
        help = "value of header byte 20 (some weird bitflags)"
    )]
    bits: u8,
    files: Vec<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let mut errs = vec![];
    for arg in &args.files {
        let path = Path::new(&arg);
        let enumformat = match args.format.as_str() {
            "DXT1" => wtx_tools::WtxFormat::DXT1,
            "DXT5" => wtx_tools::WtxFormat::DXT5,
            _ => panic!("unsupported format"),
        };
        let result = convert_to_wtx(&path, args.include_mipmaps, enumformat, args.bits);
        match result {
            Ok(_) => println!("success"),
            Err(e) => {
                println!("fail {:?}", e);
                errs.push(e);
            }
        }
    }
    println!(
        "finished processing {} files. {} failed to process.",
        args.files.len(),
        errs.len()
    );
    Ok(())
}

fn convert_to_wtx(
    filename: &std::path::Path,
    gen_mipmaps: bool,
    format: wtx_tools::WtxFormat,
    bits: u8,
) -> Result<()> {
    let img = ImageReader::open(filename)?.decode()?;

    let bytes = wtx_tools::generate_wtx_from_image(img.to_rgba8(), gen_mipmaps, format, bits);

    let mut newpath = PathBuf::from(filename);
    assert_eq!(newpath.set_extension("wtx"), true);
    let mut file = File::create(&newpath)?;
    file.write_all(&bytes)?;
    println!("Saved {:?}", newpath);
    Ok(())
}
