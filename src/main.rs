extern crate image;
mod Info_Json;
use Info_Json::IIIFVersion;
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

    let info = ImageInfo::from_image(&img);

    let json = Tiler::create_image(info, "iiif", "http://localhost:8887/iiif/", IIIFVersion::VERSION3);
    println!("{}",json);
}
