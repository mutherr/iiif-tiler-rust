use std::fs::File;

use clap::Parser;
extern crate image;
mod Info_Json;
use Info_Json::IIIFVersion;
mod Image_Info;
use Image_Info::ImageInfo;
mod IIIF_Image;
use IIIF_Image::IIIFImage;
mod tiler;
use tiler::Tiler;
use serde_json::{to_writer_pretty, Value};
use anyhow::{Error, Result};

const DEFAULT_URI: &str = "http://localhost:8887/iiif/";
const DEFAULT_VERSION: &str = "2";
const DEFAULT_ZOOM_LEVELS: i32 = 5;
const DEFAULT_TILE_SIZE: i32 = 1024;
const DEFAULT_OUTPUT_DIR: &str = "iiif";

#[derive(Parser, Default, Debug)]
#[command(author = "Ryan Muther", version, about = "IIIF Image Tiler")]
struct Arguments {
    /// The file or directory path to the image(s) to be processed
    path: String,

    /// Set the identifier in the `info.json`. Default: `http://localhost:8887/iiif/`
    #[arg(short, long, default_value = DEFAULT_URI)]
    uri: String,

    /// Set the IIIF version, options are `2` or `3`. Default: `2`
    #[arg(short, long, default_value = DEFAULT_VERSION)]
    iiif_version: String,

    /// Set the number of zoom levels for this image. Default: `5`
    #[arg(short, long, default_value_t = DEFAULT_ZOOM_LEVELS)]
    zoom_levels: i32,

    /// Set the tile size. Default: `1024`
    #[arg(short, long, default_value_t = DEFAULT_TILE_SIZE)]
    tile_size: i32,

    /// Directory where the IIIF images are stored. Default: `iiif`
    #[arg(short, long, default_value = DEFAULT_OUTPUT_DIR)]
    output_dir: String,
}

fn process_image(args: Arguments, iiif_version: IIIFVersion) -> Result<(),Error> {
    println!{"Loading image from: {}", args.path};
    let img = IIIFImage::new(args.path.as_str());

    let info = ImageInfo::new(&img, 
                                        args.tile_size, 
                                       args.tile_size, 
                                        args.zoom_levels);

    let manifest = Tiler::create_image(&info, &args.output_dir, "http://localhost:8887/iiif/", iiif_version)?;
    write_manifest(&args, &info, &manifest)?;
    Ok(())
}

fn write_manifest(args: &Arguments, info: &ImageInfo, manifest: &String) -> Result<(),Error> {
    let file_path = format!("{}/{}.xml",args.output_dir,info.id());
    let file = File::create(file_path).expect(format!("Cannot create manifest file",).as_str());
    let json_manifest: Value = serde_json::from_str(&manifest).expect("Invalid JSON");

    // Write the pretty-printed JSON to the file
    to_writer_pretty(file, &json_manifest)?;
    Ok(())
}

fn main() -> Result<()>{
    let args = Arguments::parse();

    // determine which IIIF version we're working with
    let iiif_version = match args.iiif_version.as_str() {
        "2" => Ok(IIIFVersion::VERSION211),
        "3" => Ok(IIIFVersion::VERSION3),
        _ => Err(Error::msg(format!(
            "Unrecognized IIIF version: '{}'. Please provide '2' or '3'.",
            args.iiif_version)))
    }?;
    println!("{:?}", args);

    process_image(args, iiif_version)?;

    Ok(())
}
