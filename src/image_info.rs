use crate::iiif_image::IIIFImage;
use std::fmt;

/**
 * This class provides information on the scale and sizes of the tiles.
 */
#[derive(Debug, PartialEq)]
pub struct ImageInfo<'a> {
    _tile_width: i32,
    _tile_height: i32,
    _zoom_levels: i32,
    _image: &'a IIIFImage,
    _scale_factors: Vec<i32>,
    _sizes: Vec<(i32, i32)>,
}

impl<'a> ImageInfo<'a> {
    pub fn new(image: &'a IIIFImage, tile_width: i32, tile_height: i32, zoom_level: i32) -> Self {
        let mut info = ImageInfo {
            _image: image,
            _tile_width: tile_width,
            _tile_height: tile_height,
            _zoom_levels: zoom_level,
            _scale_factors: Vec::new(),
            _sizes: Vec::new(),
        };
        info.initialize_image_info();
        info
    }

    pub fn fit_to_zoom_level(&mut self) {
        self.initialize_image_info();
    }

    pub fn fit_to_max_file_no(&mut self, p_max_file_no: i32) {
        let t_max_zoom = 4;
        let t_max_tile_size_factor = 5;
        let t_file_count = 0;
        // Find optimal tile size and zoom level
        let (t_zoom_selected, t_tile_size) =
            'outer: loop {
                for j in 1..=t_max_tile_size_factor {
                    let tile_size = j * 256;

                    for t_zoom in (1..=t_max_zoom).rev() {
                        let file_count = self._calculate_file_count(t_zoom, tile_size, tile_size);

                        if file_count < p_max_file_no {
                            log::info!(
                                "Using TileSize: {} Zoom: {} came back with {} files. Target: {}",
                                tile_size,
                                t_zoom,
                                file_count,
                                p_max_file_no
                            );

                            break 'outer (t_zoom, tile_size);
                        } else {
                            log::debug!(
                            "Rejected TileSize: {} Zoom: {} came back with {} files. Target: {}",
                            tile_size, t_zoom, file_count, p_max_file_no
                        );
                        }
                    }
                }
            };
        self.set_tile_width(t_tile_size);
        self.set_tile_height(t_tile_size);
        self.set_zoom_level(t_zoom_selected);
        self.initialize_image_info();
        log::info!(
            "Found combinations {} with a file count of {}",
            self,
            t_file_count
        );
    }

    pub fn calculate_file_count(&self) -> i32 {
        self._calculate_file_count(self._zoom_levels, self._tile_width, self._tile_height)
    }

    pub fn _calculate_file_count(&self, p_zoom: i32, p_tile_width: i32, p_tile_height: i32) -> i32 {
        let mut t_file_count = 0;

        // println!("zoom: {} tile width {}", p_zoom, p_tile_width);
        let mut reached_multiple_fullsized_tile = false;

        for t_zoom in 0..p_zoom {
            let t_zoom_factor = 2i32.pow(t_zoom as u32); // Math.pow(2, tZoom) equivalent
            let t_width = self._image.get_width() / t_zoom_factor;
            let t_height = self._image.get_height() / t_zoom_factor;

            // Calculate number of tiles needed
            let t_tile_x_count = (t_width as f64 / p_tile_width as f64).ceil() as i32;
            let t_tile_y_count = (t_height as f64 / p_tile_height as f64).ceil() as i32;

            // println!("Zoomfactor {} tiles-x {} tiles-y {} width = {} tileCount = {}",
            //         t_zoom_factor, t_tile_x_count, t_tile_y_count, t_width,
            //         t_tile_x_count * t_tile_y_count);

            // Each tile creates 4 files: 3 directories and 1 image
            if t_width < p_tile_width && t_height < p_tile_height {
                t_file_count += t_tile_x_count * t_tile_y_count * 3;
                reached_multiple_fullsized_tile = true;
            } else {
                t_file_count += t_tile_x_count * t_tile_y_count * 4;
            }
        }

        // If the tile is bigger than the image size we add 3 directories but
        // for one we need to add 4.
        if reached_multiple_fullsized_tile {
            t_file_count += 1;
        }

        // println!("Total tiles {}", t_file_count);

        // Add full sizes (1 full directory then three sub directories (size/rotation/file))
        // And full w,h
        // println!("Sizes {} should be 16", ((p_zoom + 2) * 3) + 4);
        t_file_count += ((p_zoom + 2) * 3) + 4;

        // Add info.json
        t_file_count += 1;

        // Add containing directory
        t_file_count += 1;

        t_file_count
    }

    fn initialize_image_info(&mut self) {
        self._scale_factors = Vec::new();
        self._sizes = Vec::new();
        for i in (0..=self._zoom_levels).rev() {
            let scale = 2i32.pow(i as u32);
            let width = ((self._image.get_width() as f64) / (scale as f64)).ceil() as i32;
            let height = ((self._image.get_height() as f64) / (scale as f64)).ceil() as i32;
            self._sizes.push((width, height));
            self._scale_factors.push(scale);
        }
    }

    pub fn id(&self) -> String {
        self._image.id()
    }

    pub fn get_scale_factors(&self) -> Vec<i32> {
        self._scale_factors.clone()
    }

    pub fn get_sizes(&self) -> Vec<(i32, i32)> {
        self._sizes.clone()
    }

    pub fn get_width(&self) -> i32 {
        self._image.get_width()
    }

    pub fn get_height(&self) -> i32 {
        self._image.get_height()
    }

    pub fn get_tile_width(&self) -> i32 {
        self._tile_width
    }

    pub fn set_tile_width(&mut self, p_tile_width: i32) {
        self._tile_width = p_tile_width;
    }

    pub fn get_tile_height(&self) -> i32 {
        self._tile_height
    }

    pub fn set_tile_height(&mut self, p_tile_height: i32) {
        self._tile_height = p_tile_height;
    }

    pub fn set_zoom_level(&mut self, p_zoom_level: i32) {
        self._zoom_levels = p_zoom_level;
    }

    pub fn get_image(&self) -> &IIIFImage {
        self._image
    }
}

impl fmt::Display for ImageInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image info:\n\tTile size; width: {}, height: {}\n\tZoom levels: {}\n\t * Sizes: {}\n\t * Scale Factors: {}", self._tile_width, self._tile_height, self._zoom_levels, self._sizes.len(), self._scale_factors.len())
    }
}
