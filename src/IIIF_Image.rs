use std::{fs::File, io::BufReader, path::Path};
use anyhow::Error;
use image::{DynamicImage,ImageReader};

/**
 * This class stores the source image as a DynamicImage and also works out the IIIF image identifier from the filename
 */

 #[derive(Debug, PartialEq)]
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

   pub fn get_width(&self) -> i32 {
      self.image.width() as i32
   }

   pub fn get_height(&self) -> i32 {
      self.image.height() as i32
   }

   pub fn get_image(&self) -> DynamicImage {
      self.image.clone()
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

fn load_image(img_path: &str) -> Result<(DynamicImage, String),Error> {
   // open the file and create a buffered reader
   let file = File::open(img_path)?;
   let reader = BufReader::new(file);

   // read in the image and convert it to rbg8
   let img = ImageReader::new(reader).with_guessed_format()?.decode()?.into_rgb8();
   let rgb_img = DynamicImage::ImageRgb8(img);

   // extract the file name as the image id
   let file_path = Path::new(img_path);
   let image_id = file_path
        .file_stem()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| file_name.to_string())
        .ok_or_else(|| {
         Error::msg(format!(
             "Failed to extract file name from path: {}",
             file_path.display()
         ))
     })?;

   Ok((rgb_img, image_id.to_string()))
}
