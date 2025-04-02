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
    (43, 110),
    (85, 219),
    (169, 437),
    (338, 874),
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

        let size_image_path = output_dir.join(format!("test/full/{},/0/default.jpg", width));

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
    let actual_count = actual_files.len();

    assert_eq!(
        predicted_count, actual_count as i32,
        "Predicted number of files is different to the actual."
    );

    Ok(())
}

#[test]
fn test_check_count_large() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/van.jpg";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 512, 512, 4);

    let predicted_count = image_info.calculate_file_count();
    let version = IIIFVersion::VERSION211;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let actual_files = count_files(&output_dir)?;
    let actual_count = actual_files.len();

    assert_eq!(
        predicted_count,
        actual_count as i32, // Convert usize to i32 if needed, based on return type
        "Predicted number of files is different to the actual."
    );

    Ok(())
}

#[test]
fn test_smaller_zoom_level() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/exact_tiles.jpg";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 256, 256, 3);

    let predicted_count = image_info.calculate_file_count() + 1;
    let version = IIIFVersion::VERSION211;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let actual_files = count_files(&output_dir)?;

    let actual_count = actual_files.len();

    assert_eq!(
        predicted_count, actual_count as i32,
        "Predicted number of files is different to the actual."
    );

    Ok(())
}

#[test]
fn test_limit_to_100() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/exact_tiles.jpg";
    let image = IIIFImage::new(image_file);

    let mut image_info = ImageInfo::new(&image, 256, 256, 4);

    image_info.fit_to_max_file_no(100);

    let predicted_count = image_info.calculate_file_count() + 1;
    let version = IIIFVersion::VERSION211;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let actual_files = count_files(&output_dir)?;

    let actual_count = actual_files.len();

    assert_eq!(
        predicted_count, actual_count as i32,
        "Predicted number of files is different to the actual."
    );

    assert!(
        actual_count < 100,
        "Requested less than 100 files but got more"
    );

    Ok(())
}

#[test]
fn test_limit_to_100_large() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/van.jpg";
    let image = IIIFImage::new(image_file);

    let mut image_info = ImageInfo::new(&image, 256, 256, 4);

    image_info.fit_to_max_file_no(100);

    let predicted_count = image_info.calculate_file_count();
    let version = IIIFVersion::VERSION211;

    let tiler = Tiler::new(&image_info, &version);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let actual_files = count_files(&output_dir)?;
    let actual_count = actual_files.len();

    assert_eq!(
        predicted_count, actual_count as i32,
        "Predicted number of files is different to the actual."
    );

    assert!(
        actual_count < 100,
        "Requested less than 100 files but got more"
    );

    Ok(())
}

#[test]
fn test_rounding() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/tractor.jpg";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 1024, 1024, 5);

    let tiler = Tiler::new(&image_info, &IIIFVersion::VERSION3);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let incorrect_path = output_dir.join("tractor/full/503,/0/default.jpg");
    assert!(
        !incorrect_path.exists(),
        "Rounded down instead of UP. Found {} expected tractor/full/504,/0/default.jpg",
        incorrect_path.display()
    );

    let correct_path = output_dir.join("tractor/full/504,/0/default.jpg");
    assert!(
        correct_path.exists(),
        "Correct rounding should exist. Didn't find {}",
        correct_path.display()
    );

    Ok(())
}

#[test]
fn test_zoom_limit() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/van.jpg";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 256, 256, 5);

    let mut estimated_sizes = Vec::new();
    for i in (0..=5).rev() {
        let ratio = 2i32.pow(i as u32);
        estimated_sizes.push((
            ((image.get_width() as f64) / (ratio as f64)).ceil() as i32,
            ((image.get_height() as f64) / (ratio as f64)).ceil() as i32,
        ));
    }

    let published_sizes = image_info.get_sizes();
    assert_eq!(
        published_sizes.len(),
        estimated_sizes.len(),
        "Number of sizes doesn't match"
    );

    for i in 0..published_sizes.len() {
        let published_size = published_sizes[i];
        let expected_size = estimated_sizes[i];

        assert_eq!(published_size, expected_size, "Size {} wasn't expected", i);
    }

    let larger_tile_image_info = ImageInfo::new(&image, 1024, 1024, 5);
    let tiler = Tiler::new(&larger_tile_image_info, &IIIFVersion::VERSION211);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let edge_tile_path = output_dir.join("van/3072,2048,960,976/960,/0/default.jpg");
    let edge_tile = image::open(&edge_tile_path)?;

    assert_eq!(
        edge_tile.width(),
        960,
        "Expected edge tile to be 960 pixels wide."
    );

    Ok(())
}

#[test]
// Tests the case where an image is exactly one pixel larger than a multiple of the tile size
fn test_odd_sized_image() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    // Load test image
    let image_file = "tests/fixtures/odd-sized.jpg";
    let image = IIIFImage::new(image_file);

    // Create image info with specific tile size and zoom level
    let image_info = ImageInfo::new(&image, 512, 512, 1);

    // Generate tiles
    let tiler = Tiler::new(&image_info, &IIIFVersion::VERSION211);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    // Check for the specific bottom 1-pixel height tile
    let edge_tile_path = output_dir.join("odd-sized/0,3072,512,1/512,/0/default.jpg");

    // Assert the file exists
    assert!(
        edge_tile_path.exists(),
        "Missed bottom 1 pixel tile. Did not find {}",
        edge_tile_path.display()
    );

    Ok(())
}

#[test]
//NB: This test takes a while to run as it tiles a large image
fn test_small_tiles() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/brazil.jpg";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 1024, 1024, 5);

    let tiler = Tiler::new(&image_info, &IIIFVersion::VERSION211);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let small_tiles = [
        "0,6144,2048,3/1024,/0/default.jpg",
        "2048,6144,2048,3/1024,/0/default.jpg",
        "4096,6144,2048,3/1024,/0/default.jpg",
        "6144,6144,2048,3/1024,/0/default.jpg",
        "8192,6144,1288,3/644,/0/default.jpg",
    ];

    for tile_path in &small_tiles {
        let full_path = output_dir.join(format!("brazil/{}", tile_path));
        assert!(
            full_path.exists(),
            "Expected tile: {} to exist",
            full_path.display()
        );
    }

    Ok(())
}

#[test]
fn test_alpha_channel() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let output_dir = tmp_dir.path().join("iiif");
    fs::create_dir_all(&output_dir)?;

    let image_file = "tests/fixtures/alpha.png";
    let image = IIIFImage::new(image_file);

    let image_info = ImageInfo::new(&image, 256, 256, 5);

    let tiler = Tiler::new(&image_info, &IIIFVersion::VERSION211);
    tiler.generate_tiles(&output_dir.to_string_lossy())?;

    let tiles_to_check = [
        "0,0,256,256/256,/0/default.jpg",
        "512,256,256,216/256,/0/default.jpg",
        "0,0,256,256/256,/0/default.jpg",
        "512,0,512,472/256,/0/default.jpg",
        "0,0,1024,472/256,/0/default.jpg",
    ];

    for tile_path in &tiles_to_check {
        let full_path = output_dir.join(format!("alpha/{}", tile_path));
        assert!(
            full_path.exists(),
            "Expected tile: {} to exist",
            full_path.display()
        );
    }

    Ok(())
}

//Helper functions for tests

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

//TODO: 1 test to add: make sure we can tile a *big* image
