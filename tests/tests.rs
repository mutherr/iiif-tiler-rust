// tests for the iiif tile generator

use serde_json::Value;
use iiif_tiler_rust::Info_Json::{InfoJSON, IIIFVersion};
use iiif_tiler_rust::Image_Info::ImageInfo;
use iiif_tiler_rust::IIIF_Image::IIIFImage;

#[test]
fn test_version_2_json() {
    let img = IIIFImage::new("tests/fixtures/test.jpg");
    let info = ImageInfo::new(&img, 1024, 1024, 5);
    let info_json = InfoJSON::new(&info, "http://localhost:8887/iiif/", &IIIFVersion::VERSION211);
    let json = info_json.to_json().unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["@context"], "http://iiif.io/api/image/2/context.json");
    assert_eq!(parsed["id"], "http://localhost:8887/iiif/test");
    assert_eq!(parsed["profile"], "http://iiif.io/api/image/2/level0.json");
    assert_eq!(parsed["protocol"], "http://iiif.io/api/image");
    assert_eq!(parsed["width"], 9480);
    assert_eq!(parsed["height"], 6147);
    assert_eq!(parsed["sizes"][0]["height"], 192);
    assert_eq!(parsed["sizes"][0]["width"], 296);
    assert_eq!(parsed["tiles"][0]["width"], 1024);
    assert_eq!(parsed["tiles"][0]["height"], 1024);
    let scale_factors = parsed["tiles"][0]["scaleFactors"]
        .as_array()
        .unwrap() // Safely unwrap or handle the case where it's not an array
        .iter()
        .map(|v| v.as_i64().unwrap()) // Convert each Value to i64
        .collect::<Vec<i64>>();
    assert_eq!(scale_factors, vec![32, 16, 8, 4, 2, 1]);
}

#[test]
fn test_version_3_json() {
    let img = IIIFImage::new("tests/fixtures/test.jpg");
    let info = ImageInfo::new(&img, 1024, 1024, 5);
    let info_json = InfoJSON::new(&info, "http://localhost:8887/iiif/", &IIIFVersion::VERSION3);
    let json = info_json.to_json().unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["@context"], "http://iiif.io/api/image/3/context.json");
    assert_eq!(parsed["id"], "http://localhost:8887/iiif/test");
    assert_eq!(parsed["type"], "ImageService3");
    assert_eq!(parsed["profile"], "level0");
    assert_eq!(parsed["protocol"], "http://iiif.io/api/image");
    assert_eq!(parsed["width"], 9480);
    assert_eq!(parsed["height"], 6147);
    assert_eq!(parsed["sizes"][0]["height"], 192);
    assert_eq!(parsed["sizes"][0]["width"], 296);
    assert_eq!(parsed["tiles"][0]["width"], 1024);
    assert_eq!(parsed["tiles"][0]["height"], 1024);
    let scale_factors = parsed["tiles"][0]["scaleFactors"]
        .as_array()
        .unwrap() // Safely unwrap or handle the case where it's not an array
        .iter()
        .map(|v| v.as_i64().unwrap()) // Convert each Value to i64
        .collect::<Vec<i64>>();
    assert_eq!(scale_factors, vec![32, 16, 8, 4, 2, 1]);
}