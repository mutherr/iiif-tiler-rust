use std::{error::Error, fs::File};

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

#[derive(Parser,Default,Debug)]
#[command(author = "Ryan Muther", version, about="IIIF Image Tiler")]
struct Arguments {
    #[arg(help = "Set the identifier in the info.json. Default: http://localhost:8887/iiif/")]
    #[arg(short, long, default_value="http://localhost:8887/iiif/")]
    uri: String,
    #[arg(help = "set the IIIF version, options are 2 or 3. Default: 2.11")]
    #[arg(short, long, default_value="2")]
    iiif_version: String,
    #[arg(help = "Set the number of zoom levels for this image. Default: 5")]
    #[arg(short, long,default_value_t=5)]
    zoom_levels: i32,
    #[arg(help = "Set the tile size. Default: 1024")]
    #[arg(short, long, default_value_t=1024)]
    tile_size: i32,
    #[arg(help = "Directory where the IIIF images are stored. Default: iiif")]
    #[arg(short, long, default_value="iiif")]
    output_dir: String
}

fn write_manifest(args: &Arguments, info: &ImageInfo, manifest: &String) -> Result<(),std::io::Error> {
    let file_path = format!("{}/{}.xml",args.output_dir,info.id());
    let file = File::create(file_path).expect(format!("Cannot create manifest file",).as_str());
    let json_manifest: Value = serde_json::from_str(&manifest).expect("Invalid JSON");

    // Write the pretty-printed JSON to the file
    to_writer_pretty(file, &json_manifest)?;
    Ok(())
}

fn main() -> std::io::Result<()>{
    let args = Arguments::parse();

    // determine which IIIF version we're working with
    let mut iiif_version ;
    match args.iiif_version.as_str() {
        "2" => {iiif_version = IIIFVersion::VERSION211}
        "3" => {iiif_version = IIIFVersion::VERSION3}
        _ => {panic!("Unrecognized IIIF version. Please provide 2 or 3")}
    }
    println!("{:?}", args);
    // TODO: integrate command arguments with the program itself

    let test_path = "src/test/brazil.jpg";
    println!{"Loading image from: {}", test_path};
    let img = IIIFImage::new(test_path);

    let info = ImageInfo::from_image(&img);

    let manifest = Tiler::create_image(&info, "iiif", "http://localhost:8887/iiif/", IIIFVersion::VERSION3)?;
    write_manifest(&args, &info, &manifest)?;

    Ok(())
}
