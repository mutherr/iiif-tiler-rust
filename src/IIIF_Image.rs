use std::{fs::File, io::BufReader, path::Path};
use image::{DynamicImage,ImageReader};

/**
 * This class stores the source image as a DynamicImage and also works out the IIIF image identifier from the filename
 */

pub struct IIIFImage {
   image: DynamicImage,
   id: String,
}

impl IIIFImage {

   pub fn new(img_path: &str) -> IIIFImage {
      let loaded = load_image(img_path);
      match loaded {
         Ok((img,id)) => {
            IIIFImage {
               image: img,
               id,
            }
         }
         Err(e) => {
            panic!("Error loading image: {}", e);
         }   
      }
   }

   pub fn id(&self) -> String {
      self.id.clone()
   }

   pub fn set_id(&mut self, p_id: String) {
      self.id = p_id;
   }

   pub fn get_width(&self) -> i32 {
      self.image.width() as i32
   }

   pub fn get_height(&self) -> i32 {
      self.image.height() as i32
   }

   pub fn get_image(&self) -> DynamicImage {
      self.image.clone()
   }

   pub fn set_image(&mut self, p_image: DynamicImage) {
      self.image = p_image;
   }

}

// Implement Clone for IIIFImage
impl Clone for IIIFImage {
   fn clone(&self) -> Self {
       IIIFImage {
           image: self.image.clone(),
           id: self.id.clone(),
       }
   }
}

fn load_image(img_path: &str) -> Result<(DynamicImage, String),Box<dyn std::error::Error>> {
   let file = File::open(img_path)?;
   let reader = BufReader::new(file);
   // discard alpha channel if it exists
   let img = ImageReader::new(reader).with_guessed_format()?.decode()?.into_rgb8();
   let rgb_img = DynamicImage::ImageRgb8(img);

   let file_path = Path::new(img_path);
   let mut image_id = "";
   // Get the file name (last part of the path)
   if let Some(file_name) = file_path.file_stem() {
      image_id = file_name.to_str().unwrap();
   } else {
      println!("Could not extract the file name.");
   }

   Ok((rgb_img, image_id.to_string()))
}
