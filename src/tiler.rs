use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::image_info::ImageInfo;
use crate::info_json::{IIIFVersion, InfoJSON};
use anyhow::{Error, Result};
use image::DynamicImage;
use log::info;

pub struct Tiler<'a> {
    image: &'a ImageInfo<'a>,
    version: &'a IIIFVersion,
}

impl<'a> Tiler<'a> {
    pub fn new(image: &'a ImageInfo, version: &'a IIIFVersion) -> Tiler<'a> {
        Tiler { image, version }
    }

    pub fn get_output_dir(&self, p_image_dir: &str) -> String {
        format!("{}/{}", p_image_dir, self.image.id())
    }

    pub fn generate_tiles(&self, image_dir: &str) -> Result<(), Error> {
        self._generate_tiles(image_dir, &self.image.id())?;
        Ok(())
    }

    fn _generate_tiles(&self, image_dir: &str, filename: &str) -> Result<(), Error> {
        let img_dir = format!("{}/{}", image_dir, filename);
        info!("Using {}", self.image);
        info!("Creating full scaled images...");
        self._generate_sizes(&img_dir)?;
        info!("Creating tiles...");
        self._generate_scale_tiles(&img_dir)?;
        Ok(())
    }

    fn _generate_sizes(&self, image_dir: &str) -> Result<(), Error> {
        for size in self.image.get_sizes() {
            let size_str = format!("{},", size.0);
            let scaled_image = self.image.get_image().get_image().resize(
                size.0 as u32,
                size.1 as u32,
                image::imageops::FilterType::Nearest,
            );

            let output_path = PathBuf::from(image_dir)
                .join("full")
                .join(size_str)
                .join("0")
                .join("default.jpg");
            save_image(&scaled_image, &output_path)?;
            if size.0 == self.image.get_width() && size.1 == self.image.get_height() {
                let max_full_str = if *self.version == IIIFVersion::VERSION3 {
                    "max"
                } else {
                    "full"
                };
                let output_path = PathBuf::from(image_dir)
                    .join("full")
                    .join(max_full_str)
                    .join("0")
                    .join("default.jpg");
                save_image(&scaled_image, &output_path)?;
            }
        }
        Ok(())
    }

    fn _generate_scale_tiles(&self, p_image_dir: &str) -> Result<(), Error> {
        for scale in self.image.get_scale_factors() {
            //height in units of scale rather than px
            let t_scale_level_width = (self.image.get_width() as f32 / scale as f32).floor() as i32;
            let t_scale_level_height =
                (self.image.get_height() as f32 / scale as f32).floor() as i32;
            //calculate number of tiles along either axis
            let mut t_tile_num_width =
                (t_scale_level_width as f32 / self.image.get_tile_width() as f32).floor() as i32;
            let mut t_tile_num_height =
                (t_scale_level_height as f32 / self.image.get_tile_height() as f32).floor() as i32;

            //add extra images on either axis as needed if the tile size doesn't evenly divide the axis length
            if (t_scale_level_width % self.image.get_tile_width()) != 0 {
                t_tile_num_width += 1;
            }
            if (t_scale_level_height % self.image.get_tile_height()) != 0 {
                t_tile_num_height += 1;
            }

            //make tiles
            for x in 0..t_tile_num_width {
                for y in 0..t_tile_num_height {
                    let tile_x = x * self.image.get_tile_width() * scale;
                    let tile_y = y * self.image.get_tile_height() * scale;
                    let scaled_tile_width = self.image.get_tile_width() * scale;
                    let mut tiled_width_calc = self.image.get_tile_width();
                    if tile_x + scaled_tile_width > self.image.get_width() {
                        let scaled_tile_width = self.image.get_width() - tile_x;
                        tiled_width_calc = (scaled_tile_width as f32 / scale as f32).ceil() as i32;
                    }
                    let scaled_tile_height = self.image.get_tile_height() * scale;
                    let mut tiled_height_calc = self.image.get_tile_height();
                    if tile_y + scaled_tile_height > self.image.get_height() {
                        let scaled_tile_height = self.image.get_height() - tile_y;
                        tiled_height_calc =
                            (scaled_tile_height as f32 / scale as f32).ceil() as i32;
                    }

                    let url = if *self.version == IIIFVersion::VERSION3 {
                        // formatting path for v3
                        format!(
                            "./{},{},{},{}/{},{}/0/default.jpg",
                            tile_x,
                            tile_y,
                            tiled_width_calc,
                            tiled_height_calc,
                            tiled_width_calc,
                            tiled_height_calc
                        )
                    } else {
                        // formatting path for v2.1
                        format!(
                            "./{},{},{},{}/{},/0/default.jpg",
                            tile_x, tile_y, tiled_width_calc, tiled_height_calc, tiled_width_calc
                        )
                    };

                    let t_output_file = PathBuf::from(format!("{}/{}", p_image_dir, url));
                    if let Some(parent_dir) = t_output_file.parent() {
                        if let Err(e) = create_dir_all(parent_dir) {
                            eprintln!("Failed to create directory {}: {}", parent_dir.display(), e)
                        }
                    }

                    let tile_image = self
                        .image
                        .get_image()
                        .get_image()
                        .crop_imm(
                            tile_x as u32,
                            tile_y as u32,
                            scaled_tile_width as u32,
                            scaled_tile_height as u32,
                        )
                        .into_rgb8();

                    let scaled_image = if tile_image.width() == tiled_width_calc as u32
                        && tile_image.height() == tiled_height_calc as u32
                    {
                        // No resize needed, use original image
                        DynamicImage::ImageRgb8(tile_image)
                    } else {
                        // Choose filter type based on target dimensions
                        let filter_type = if tiled_width_calc > 3 && tiled_height_calc > 3 {
                            image::imageops::FilterType::CatmullRom
                        } else {
                            image::imageops::FilterType::Lanczos3
                        };

                        // Resize with selected filter type
                        DynamicImage::ImageRgb8(tile_image).resize(
                            tiled_width_calc as u32,
                            tiled_height_calc as u32,
                            filter_type,
                        )
                    };

                    match scaled_image.save(&t_output_file) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!(
                                "Failed to write: '{:?}' ({},{}) - Error: {}",
                                t_output_file.display(),
                                scaled_image.width(),
                                scaled_image.height(),
                                e
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // Tiles a single image, returning the manifest in json form
    pub fn create_image(
        image: &ImageInfo,
        output_dir: &str,
        uri: &str,
        version: &IIIFVersion,
    ) -> Result<String, Error> {
        let tiler = Tiler::new(image, version);
        match tiler.generate_tiles(output_dir) {
            Ok(_) => {}
            Err(e) => return Err(Error::msg(format!("Failed to generate tiles: {}", e))),
        }
        let info = InfoJSON::new(image, uri, version);

        Ok(info.to_json()?)
    }
}

// helper function for image saving
fn save_image(image: &DynamicImage, path: &Path) -> Result<(), Error> {
    if let Some(parent_dir) = path.parent() {
        create_dir_all(parent_dir)
            .map_err(|_e| Error::msg(format!("Failed to create directory: {:?}", parent_dir)))?;
    }

    image
        .save(path)
        .map_err(|_e| Error::msg(format!("Failed to save image: {:?}", path)))
}
