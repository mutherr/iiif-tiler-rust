use std::{fs::read_dir, fs::File, path::Path};

use clap::Parser;
extern crate image;
pub mod info_json;
use info_json::IIIFVersion;
pub mod image_info;
use image_info::ImageInfo;
pub mod iiif_image;
use iiif_image::IIIFImage;
pub mod tiler;
use anyhow::{Error, Result};
use log::info;
use serde_json::{to_writer_pretty, Value};
use tiler::Tiler;

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

    /// Set the identifier in the mainfest.
    #[arg(short, long, default_value = DEFAULT_URI)]
    uri: String,

    /// Set the IIIF version, options are `2` or `3`.
    #[arg(short, long, default_value = DEFAULT_VERSION)]
    iiif_version: String,

    /// Set the number of zoom levels for this image.
    #[arg(short, long, default_value_t = DEFAULT_ZOOM_LEVELS)]
    zoom_levels: i32,

    /// Set the tile size.
    #[arg(short, long, default_value_t = DEFAULT_TILE_SIZE)]
    tile_size: i32,

    /// Directory where the image tiles are stored.
    #[arg(short, long, default_value = DEFAULT_OUTPUT_DIR)]
    output_dir: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn process_directory(
    args: &Arguments,
    dir_path: &str,
    iiif_version: &IIIFVersion,
) -> Result<(), Error> {
    let dir_path = Path::new(dir_path);

    // Read the directory
    let entries = read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?; // Handle `Result<DirEntry, Error>`
        let path = entry.path();

        // Process only files with valid extensions
        if path.is_file() && is_image_file(&path) {
            // Use the path as a string safely
            if let Some(path_str) = path.to_str() {
                process_image(args, path_str, iiif_version)?;
            } else {
                return Err(Error::msg(format!(
                    "Invalid UTF-8 in file path: {:?}",
                    path
                )));
            }
        }
    }
    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "bmp" | "tiff"
        ),
        None => false,
    }
}

fn process_image(
    args: &Arguments,
    img_path: &str,
    iiif_version: &IIIFVersion,
) -> Result<(), Error> {
    info!("Loading image from: {}", img_path);
    let img = IIIFImage::new(img_path);

    let info = ImageInfo::new(&img, args.tile_size, args.tile_size, args.zoom_levels);

    let manifest = Tiler::create_image(&info, &args.output_dir, &args.uri, iiif_version)?;
    write_manifest(args, &info, &manifest)?;

    info!("Successfully processed image: {}", img_path);
    Ok(())
}

fn write_manifest(args: &Arguments, info: &ImageInfo, manifest: &str) -> Result<(), Error> {
    let file_path = format!("{}/{}.xml", args.output_dir, info.id());
    let file = File::create(file_path)?;
    let json_manifest: Value = serde_json::from_str(manifest)?;

    // Write the pretty-printed JSON to the file
    to_writer_pretty(file, &json_manifest)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Arguments::parse();

    if args.verbose {
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    } else {
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // determine which IIIF version we're working with
    let iiif_version = match args.iiif_version.as_str() {
        "2" => Ok(IIIFVersion::VERSION211),
        "3" => Ok(IIIFVersion::VERSION3),
        _ => Err(Error::msg(format!(
            "Unrecognized IIIF version: '{}'. Please provide '2' or '3'.",
            args.iiif_version
        ))),
    }?;

    let path = Path::new(args.path.as_str());

    if path.is_file() {
        process_image(&args, args.path.as_str(), &iiif_version)?;
    } else if path.is_dir() {
        process_directory(&args, path.to_str().unwrap(), &iiif_version)?;
    } else {
        println!(
            "{:?} does not exist or is neither a file nor a directory.",
            path
        );
    }

    Ok(())
}
