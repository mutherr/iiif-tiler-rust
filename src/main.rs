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

fn main() -> std::io::Result<()>{
    let args = Arguments::parse();

    // determine which IIIF version we're working with
    let mut iiif_version = IIIFVersion::VERSION211;
    match args.iiif_version.as_str() {
        "2" => {}
        "3" => {iiif_version = IIIFVersion::VERSION3}
        _ => {panic!("Unrecognized IIIF version. Please provide 2 or 3")}
    }
    println!("{:?}", args);
    // TODO: integrate command arguments with the program itself

    let test_path = "src/test/brazil.jpg";
    println!{"Loading image from: {}", test_path};
    let img = IIIFImage::new(test_path);

    let info = ImageInfo::from_image(&img);

    let json = Tiler::create_image(&info, "iiif", "http://localhost:8887/iiif/", IIIFVersion::VERSION3)?;
    // TODO: Output json file in image's directory with tile folders
    println!("{}",json);
    // Open a file to write the pretty-printed JSON
    let file_path = format!("{}/{}.xml",args.output_dir,info.id());
    let file = File::create(file_path)?;
    let json_value: Value = serde_json::from_str(&json).expect("Invalid JSON");

    // Write the pretty-printed JSON to the file
    to_writer_pretty(file, &json_value)?;

    Ok(())
}
