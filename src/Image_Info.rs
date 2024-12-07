use std::fmt;
use crate::IIIF_Image::IIIFImage;

/**
 * This class provides information on the scale and sizes of the tiles. 
 */
pub struct ImageInfo<'a>{
    _tile_width: i32,
    _tile_height: i32,
    _zoom_levels: i32,
    _image: &'a IIIFImage,
    _scale_factors: Vec<i32>,
    _sizes: Vec<(i32,i32)>
}

impl<'a> ImageInfo<'a> {
    pub fn from_image(image: &'a IIIFImage) -> Self {
        let mut info = ImageInfo {
            _image:image,
            _tile_width:1024,
            _tile_height:1024,
            _zoom_levels:5,
            _scale_factors: Vec::new(),
            _sizes: Vec::new()
        };
        info.initialize_image_info();
        info
    }

    pub fn new(image: &'a IIIFImage, tile_width: i32, tile_height: i32, zoom_level: i32) -> Self {
        let mut info = ImageInfo {
            _image:image,
            _tile_width:tile_width,
            _tile_height:tile_height,
            _zoom_levels:zoom_level,
            _scale_factors: Vec::new(),
            _sizes: Vec::new()
        };
        info.initialize_image_info();
        info
    }

    pub fn fit_to_zoom_level(&mut self) {
        self.initialize_image_info();
    }

    pub fn fit_to_max_file_no(&mut self, p_max_file_no: i32) {
        let t_max_zoom = 4;
        let t_zoom = 0;
        let t_max_tile_size_factor = 5;
        let mut t_tile_size = 0;
        let mut t_found = false;
        let mut t_file_count = 0;
        for j in 1..=t_max_tile_size_factor {
            for t_zoom in (1..=t_max_zoom).rev() {
                t_tile_size = j * 256;
                t_file_count = self._calculate_file_count(t_zoom, t_tile_size, t_tile_size);
                if t_file_count < p_max_file_no {
                    println!("Using TileSize: {} Zoom: {} came back with {} files. Target: {}", t_tile_size, t_zoom, t_file_count, p_max_file_no);
                    t_found = true;
                    break;
                } else {
                    println!("Rejected TileSize: {} Zoom: {} came back with {} files. Target: {}", t_tile_size, t_zoom, t_file_count, p_max_file_no);
                }
            }
        }
        if t_found {
            self.set_tile_width(t_tile_size);
            self.set_tile_height(t_tile_size);
            self.set_zoom_level(t_zoom);
            self.initialize_image_info();
            println!("Found combinations {} with a file count of {}", self, t_file_count);
        } else {
            panic!("Failed to find combination under {} files", p_max_file_no);
        }
    }

    pub fn calculate_file_count(&self) -> i32 {
        self._calculate_file_count(self._zoom_levels, self._tile_width, self._tile_height)
    }

    fn _calculate_file_count(&self, p_zoom: i32, p_tile_width: i32, p_tile_height: i32) -> i32 {
        let mut t_file_count = 0;
        let mut reached_multiple_fullsized_tile = false;
        for t_zoom in 0..p_zoom {
            let t_zoom_factor = 2i32.pow(t_zoom as u32);
            let t_width = self._image.get_width() / t_zoom_factor;
            let t_height = self._image.get_height() / t_zoom_factor;
            let t_tile_xcount = (t_width as f64 / p_tile_width as f64).ceil() as i32;
            let t_tile_ycount = (t_height as f64 / p_tile_height as f64).ceil() as i32;
            if t_width < p_tile_width && t_height < p_tile_height {
                t_file_count += t_tile_xcount * t_tile_ycount * 3;
                reached_multiple_fullsized_tile = true;
            } else {
                t_file_count += t_tile_xcount * t_tile_ycount * 4;
            }
        }
        if reached_multiple_fullsized_tile {
            t_file_count += 1;
        }
        t_file_count += ((p_zoom + 1) * 3) + 4;
        t_file_count += 1;
        t_file_count += 1;
        t_file_count
    }

    fn initialize_image_info(&mut self) {
        self._scale_factors = Vec::new();
        self._sizes = Vec::new();
        for i in (0..=self._zoom_levels).rev() {
            let scale = 2i32.pow(i as u32);
            self._sizes.push((self._image.get_width() / scale, self._image.get_height() / scale));
            self._scale_factors.push(scale);
        }
    }

    pub fn id(&self) -> String {
        self._image.id()
    }

    pub fn get_scale_factors(&self) -> Vec<i32> {
        self._scale_factors.clone()
    }

    pub fn get_sizes(&self) -> Vec<(i32,i32)> {
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

    pub fn get_zoom_level(&self) -> i32 {
        self._zoom_levels
    }

    pub fn set_zoom_level(&mut self, p_zoom_level: i32) {
        self._zoom_levels = p_zoom_level;
    }

    pub fn get_image(&self) -> IIIFImage {
        self._image.clone()
    }

    fn set_image(&mut self, p_image: &'a IIIFImage) {
        self._image = p_image;
        self.initialize_image_info();
    }
}

impl fmt::Display for ImageInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image info:\n\tTile size; width: {}, height: {}\n\tZoom levels: {}\n\t * Sizes: {}\n\t * Scale Factors: {}", self._tile_width, self._tile_height, self._zoom_levels, self._sizes.len(), self._scale_factors.len())
    }
}