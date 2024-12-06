extern crate image;
mod Info_Json;
use Info_Json::InfoJSON;
mod Image_Info;
use Image_Info::ImageInfo;
mod IIIF_Image;
use IIIF_Image::IIIFImage;
mod tiler;
use tiler::Tiler;

fn main() {
    let test_path = "src/test/brazil.jpg";
    println!{"Loading image from: {}", test_path};
    let img = IIIFImage::new(test_path);
    println!("Created image with ID {}", img.id());

    let info = ImageInfo::from_image(&img);
    println!("Created image info {}", info);

    let json = InfoJSON::new(&info, "test_image/".to_string(), "3.0".to_string());
    println!("Created info json {}", json.id());
    // println!("{}", json.to_json());

    let tiler = Tiler::new(&info,"3.0");
    tiler.generate_tiles("iiif");
}
