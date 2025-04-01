// tests for the iiif tile generator
use std::fs;
use std::path::Path;

use iiif_tiler_rust::iiif_image::IIIFImage;
use iiif_tiler_rust::image_info::ImageInfo;
use iiif_tiler_rust::info_json::{IIIFVersion, InfoJSON};
use iiif_tiler_rust::tiler::Tiler;

use serde_json::Value;
use tempfile::TempDir;

const EXPECTED_SIZES: [(i32, i32); 6] = [
    (42, 109),
    (84, 218),
    (168, 437),
    (337, 874),
    (675, 1748),
    (1350, 3496),
];

#[test]
fn test_version_2_json() {
    let img = IIIFImage::new("tests/fixtures/test.jpg");
    let info = ImageInfo::new(&img, 1024, 1024, 5);
    let info_json = InfoJSON::new(
        &info,
        "http://localhost:8887/iiif/",
        &IIIFVersion::VERSION211,
    );
    let json = info_json.to_json().unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed["@context"],
        "http://iiif.io/api/image/2/context.json"
    );
    assert_eq!(parsed["id"], "http://localhost:8887/iiif/test");
    assert_eq!(parsed["profile"], "http://iiif.io/api/image/2/level0.json");
    assert_eq!(parsed["protocol"], "http://iiif.io/api/image");

    // check image dimensions
    assert_eq!(parsed["width"], 3496);
    assert_eq!(parsed["height"], 1350);

    // check sizes
    let iter = parsed["sizes"]
        .as_array()
        .unwrap()
        .iter()
        .zip(EXPECTED_SIZES.iter());
    for (size, expected) in iter {
        assert_eq!(size["height"], expected.0);
        assert_eq!(size["width"], expected.1);
    }

    // check tiles
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
    assert_eq!(
        parsed["@context"],
        "http://iiif.io/api/image/3/context.json"
    );
    assert_eq!(parsed["id"], "http://localhost:8887/iiif/test");
    assert_eq!(parsed["type"], "ImageService3");
    assert_eq!(parsed["profile"], "level0");
    assert_eq!(parsed["protocol"], "http://iiif.io/api/image");

    // check image dimensions
    assert_eq!(parsed["width"], 3496);
    assert_eq!(parsed["height"], 1350);

    // check sizes
    let iter = parsed["sizes"]
        .as_array()
        .unwrap()
        .iter()
        .zip(EXPECTED_SIZES.iter());
    for (size, expected) in iter {
        assert_eq!(size["height"], expected.0);
        assert_eq!(size["width"], expected.1);
    }

    // check tiles
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
fn test_sizes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image = IIIFImage::new("tests/fixtures/test.jpg");

    let image_info = ImageInfo::new(&image, 1024, 1024, 5);
    let version = IIIFVersion::VERSION3;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    // Test that canonical sizes exist
    let sizes = image_info.get_sizes();
    for size in sizes {
        let width = size.0;
        let height = size.1;

        let size_image_path =
            output_dir.join(format!("test/full/{},{}/0/default.jpg", width, height));

        assert!(
            size_image_path.exists(),
            "Size mentioned in the info.json is missing: {}",
            size_image_path.display()
        );
    }

    Ok(())
}

#[test]
fn test_exact_tile_match() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image = IIIFImage::new("tests/fixtures/exact_tiles.jpg");

    let image_info = ImageInfo::new(&image, 1024, 1024, 1);
    let version = IIIFVersion::VERSION211;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let img_dir = output_dir.join("exact_tiles");

    let expected_files = [
        "0,0,1024,1024",
        "0,0,2048,2048",
        "0,1024,1024,1024",
        "1024,0,1024,1024",
        "1024,1024,1024,1024",
        "full",
    ];

    let mut actual_files: Vec<String> = fs::read_dir(&img_dir)?
        .filter_map(|entry| {
            entry
                .ok()
                .and_then(|e| e.file_name().to_str().map(String::from))
        })
        .collect();

    let mut expected_files_vec: Vec<String> =
        expected_files.iter().map(|&s| s.to_string()).collect();
    expected_files_vec.sort();
    actual_files.sort();

    assert_eq!(
        expected_files_vec, actual_files,
        "Unexpected files from exact tile match image"
    );

    Ok(())
}

#[test]
fn test_check_count() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/exact_tiles.jpg";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 256, 256, 4);

    let predicted_count = image_info.calculate_file_count();
    let version = IIIFVersion::VERSION211;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let actual_files = count_files(&output_dir)?;
    let actual_count = actual_files.len() as i32;

    assert_eq!(
        predicted_count, actual_count,
        "Predicted number of files is different to the actual."
    );

    Ok(())
}

/// Recursively count files in a directory
fn count_files(directory: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();

    // Optional debug output
    // if fs::read_dir(directory)?.count() > 1 {
    //     println!("Multiple {}", directory.display());
    // }

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(path_str) = path.to_str() {
            files.push(path_str.to_string());
        }

        if path.is_dir() {
            let mut sub_files = count_files(&path)?;
            files.append(&mut sub_files);
        }
    }

    Ok(files)
}

//TODO: 9 tests left to reimplement
//TODO: 1 test to add: make sure we can tile a *big* image
