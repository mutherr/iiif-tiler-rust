use core::panic;
use std::path::{Path, PathBuf};
use std::fs::create_dir_all;

use crate::Info_Json::{IIIFVersion,InfoJSON};
use crate::Image_Info::ImageInfo;
use anyhow::{Error, Result};
use image::{DynamicImage, ImageError};

pub struct Tiler<'a> {
    image: &'a ImageInfo<'a>,
    version: &'a IIIFVersion
}

impl<'a> Tiler<'a> {
    pub fn new(image: &'a ImageInfo, version: &'a IIIFVersion) -> Tiler<'a> {
        Tiler {
            image,
            version
        }
    }

    pub fn get_output_dir(&self, p_image_dir: &str) -> String {
        format!("{}/{}", p_image_dir, self.image.id())
    }

    pub fn generate_tiles(&self, image_dir: &str) {
        self._generate_tiles(image_dir, &self.image.id());
    }

    fn _generate_tiles(&self, image_dir: &str, filename: &str) -> Result<(),Error> {
        let img_dir = format!("{}/{}", image_dir, filename);
        println!("Using image info {}", self.image);
        self._generate_sizes(&img_dir)?;
        self._generate_scale_tiles(&img_dir)?;
        Ok(())
    }

    fn _generate_sizes(&self, image_dir: &str) -> Result<(),Error> {
        for size in self.image.get_sizes() {
            let size_str = format!("{},{}", size.0, size.1);
            let scaled_image = self
                .image
                .get_image()
                .get_image()
                .resize(size.0 as u32, size.1 as u32, image::imageops::FilterType::Nearest);

            let output_path = PathBuf::from(image_dir)
                .join("full")
                .join(size_str)
                .join("0")
                .join("default.jpg");
            save_image(&scaled_image, &output_path)?;
            if size.0 == self.image.get_width() && size.1 == self.image.get_height() {
                let max_full_str = if *self.version == IIIFVersion::VERSION3 { "max" } else { "full" };
                let max_output_path = PathBuf::from(image_dir)
                    .join("full")
                    .join(max_full_str)
                    .join("0")
                    .join("default.jpg");
                save_image(&scaled_image, &max_output_path)?;
            }    
        }
        Ok(())
    }

    fn _generate_scale_tiles(&self, p_image_dir: &str) -> Result<(),Error> {
        for scale in self.image.get_scale_factors() {
            //height in units of scale rather than px
            let t_scale_level_width = (self.image.get_width() as f32 / scale as f32).floor() as i32;
            let t_scale_level_height = (self.image.get_height() as f32 / scale as f32).floor() as i32;
            //calculate number of tiles along either axis
            let mut t_tile_num_width = (t_scale_level_width as f32 / self.image.get_tile_width() as f32).floor() as i32;
            let mut t_tile_num_height = (t_scale_level_height as f32 / self.image.get_tile_height() as f32).floor() as i32;
            
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
                        tiled_height_calc = (scaled_tile_height as f32 / scale as f32).ceil() as i32;
                    }
                    
                    let url = if *self.version == IIIFVersion::VERSION3 { 
                        // formatting path for v3
                        format!("./{},{},{},{}/{},{}/0/default.jpg", tile_x, tile_y, scaled_tile_width, scaled_tile_height, tiled_width_calc, tiled_height_calc) 
                    } else { 
                        // formatting path for v2.1
                        format!("./{},{},{},{}/{},/0/default.jpg", tile_x, tile_y, scaled_tile_width, scaled_tile_height, tiled_width_calc) 
                    };

                    let t_output_file = PathBuf::from(format!("{}/{}", p_image_dir, url));
                    if let Some(parent_dir) = t_output_file.parent() {
                        if let Err(e) = create_dir_all(parent_dir) {
                            eprintln!("Failed to create directory {}: {}", parent_dir.display(), e)
                        }
                    }

                    let tile_image = self.image.get_image().get_image()
                                    .crop_imm(tile_x as u32, tile_y as u32, scaled_tile_width as u32, scaled_tile_height as u32)
                                    .into_rgb8();

                    let mut scaled_image = DynamicImage::ImageRgb8(tile_image.clone());
                    if tile_image.width() == tiled_width_calc as u32 && tile_image.height() == tiled_height_calc as u32 {
                        scaled_image = DynamicImage::ImageRgb8(tile_image);
                    } else if tiled_width_calc > 3 && tiled_height_calc > 3 {
                        scaled_image = DynamicImage::ImageRgb8(tile_image).resize(tiled_width_calc as u32, tiled_height_calc as u32, image::imageops::FilterType::Nearest);
                    } else {
                        scaled_image = scaled_image.resize(tiled_width_calc as u32, tiled_height_calc as u32, image::imageops::FilterType::Lanczos3);
                    }

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
    pub fn create_image(image: &ImageInfo, output_dir: &str, uri: &str, version: IIIFVersion) -> Result<String,serde_json::Error> {
        let tiler = Tiler::new(image, &version);
        tiler.generate_tiles(output_dir);
        let info = InfoJSON::new(&image, uri, version);
        
        info.to_json()
    }

}

// helper function for image saving
fn save_image(image: &DynamicImage, path: &Path) -> Result<(), Error> {
    if let Some(parent_dir) = path.parent() {
        create_dir_all(parent_dir).map_err(|_e| {
            Error::msg(format!("Failed to create directory: {:?}", parent_dir))
        })?;
    }

    image.save(path).map_err(|_e| {
        Error::msg(format!("Failed to save image: {:?}", path))
    })
}








// /**
//  * Class that converts Images to IIIF tiles. It has static methods createImage and createImages to create the IIIF images
//  */
// public class Tiler {

//     /** 
//      * Pass a file to convert it to a IIIF static image.
//      * @param pImageFile the image file to convert
//      * @param pOutputDir the output directory for the IIIF images. Note a sub directory will be created for each image 
//      * @param pURI the identifier to use in the @id of the info.json. Note this method will add the identifier for the IIIF image to the end of this URL. So if the image file is a file called picture.jpg the URI could be http://localhost:8887/iiif and the identifier in the info.json would be http://localhost:8887/iiif/picture
//      * @param pVersion either InfoJson.VERSION211 or InfoJson.VERSION3 
//      * @return the directory that contains the IIIF image tiles
//      * @throws IOException if there is an issue loading the source image or writing the IIIF image
//      */
//     public static File createImage(final File pImageFile, final File pOutputDir,  final String pURI, final String pVersion) throws IOException {
//         IIIFImage tImage = new IIIFImage(pImageFile);

//         ImageInfo tImageInfo = new ImageInfo(tImage);

//         return createImage(tImageInfo, pOutputDir, pURI, pVersion);
//     }

//     /** 
//      * Pass a ImageInfo to convert it to a IIIF static image. This method allows you to customise the zoom level and image idenfifier of the resulting IIIF image. To change the IIIF image idenifier use pImageFile.setId()
//      * @param pImageFile the image file to convert
//      * @param pOutputDir the output directory for the IIIF images. Note a sub directory will be created for each image 
//      * @param pURI the identifier to use in the @id of the info.json. Note this method will add the identifier for the IIIF image to the end of this URL. So if the image file is a file called picture.jpg the URI could be http://localhost:8887/iiif and the identifier in the info.json would be http://localhost:8887/iiif/picture
//      * @param pVersion either InfoJson.VERSION211 or InfoJson.VERSION3 
//      * @return the directory that contains the IIIF image tiles
//      * @throws IOException if there is an issue loading the source image or writing the IIIF image
//      */
//     public static File createImage(final ImageInfo pImageFile, final File pOutputDir,  final String pURI, final String pVersion) throws IOException {
//         Tiler tTiler = new Tiler(pImageFile, pVersion);
//         tTiler.generateTiles(pOutputDir);

//         InfoJson tInfo = new InfoJson(pImageFile, pURI, pVersion);
//         Map tInfoJson = tInfo.toJson();

//         JsonUtils.writePrettyPrint(new FileWriter(new File(tTiler.getOutputDir(pOutputDir),"info.json")), tInfoJson);

//         return tTiler.getOutputDir(pOutputDir);
//     }

//     /** 
//      * Pass a list of files to convert to IIIF static images
//      * @param pFiles a list of files to convert
//      * @param pOutputDir the output directory for the IIIF images. Note a sub directory will be created for each image 
//      * @param pZoomLevel the maximum amount of zoom levels to include in the IIIF image. A good value is 5 which works with Leaflet
//      * @param pMaxFileNo if you want the number of tiles and info.json to fit into a maximum supply this variable. 
//      * The number of zoom levels and tile sizes will be adjusted to try and fit the number of files under this limit. Set it to -1 to priortise the zoom level. 
//      * @param pTileSize the width and heigh of the tile. Note tiles can only be square in this implmentation. Use -1 for default of 1024.
//      * @param pVersion either InfoJson.VERSION211 or InfoJson.VERSION3 
//      * @throws IOException if there is an issue loading the source image or writing the IIIF image
//      */
//     public static void createImages(final List<File> pFiles, final File pOutputDir, final String pIdentifier, final int pZoomLevel, final int pMaxFileNo, final int pTileSize, final String pVersion) throws IOException {
//         for (File tInputFile : pFiles) {
//             IIIFImage tImage = new IIIFImage(tInputFile);

//             ImageInfo tImageInfo = new ImageInfo(tImage);
//             tImageInfo.setZoomLevel(pZoomLevel);
//             if (pMaxFileNo != -1) {
//                 tImageInfo.fitToMaxFileNo(pMaxFileNo);
//             } else {
//                 tImageInfo.fitToZoomLevel();
//             }

//             if (pTileSize != -1) {
//                 tImageInfo.setTileWidth(pTileSize);
//                 tImageInfo.setTileHeight(pTileSize);
//             }

//             File tImageOutput = createImage(tImageInfo, pOutputDir, pIdentifier, pVersion);
//             System.out.println("Converted " + tInputFile.getPath() + " to " + tImageOutput.getPath());
//         }
//     }

//     public static void main(final String[] pArgs) throws IOException {
//         int tZoom = 5;
//         String tVersion = InfoJson.VERSION211;
//         int tTilesize = 1024;
//         String outputDir = "iiif";
//         String identifier_root = "http://localhost:8887/iiif/";

//         Options tOptions = new Options();
//         tOptions.addOption("identifier", true, "Set the identifier in the info.json. The default is " + identifier_root);
//         tOptions.addOption("zoom_levels", true, "set the number of zoom levels for this image. The default is " + tZoom);
//         tOptions.addOption("version", true, "set the IIIF version. Default is " + tVersion + " and options are 2 or 3");
//         tOptions.addOption("tile_size", true, "set the tile size. Default is " + tTilesize);
//         tOptions.addOption("output", true, "Directory where the IIIF images are generated. Default: " + outputDir);
//         tOptions.addOption("help", false, "Show this help message");

//         // create the parser
//         CommandLineParser parser = new DefaultParser();
//         HelpFormatter formatter = new HelpFormatter();
//         CommandLine tCmd = null;
//         try {
//             // parse the command line arguments
//             tCmd = parser.parse(tOptions, pArgs);
//         } catch(ParseException exp ) {
//             // oops, something went wrong
//             System.err.println("Parsing failed.  Reason: " + exp.getMessage() );
//             System.out.println(exp.getMessage());
//             formatter.printHelp("iiif-tiler", tOptions);
//         }

//         if (tCmd.hasOption("help")) {
//             formatter.printHelp("iiif-tiler", tOptions);
//             System.exit(1);
//         }

//         List<File> tInputFiles = new ArrayList<File>();
//         if (!tCmd.getArgList().isEmpty()) {
//             for (String tFile : tCmd.getArgList()) {
//                 tInputFiles.add(new File(tFile));
//             }
//         } else {
//             System.out.println("Looking for images in current directory");
//             // Look for supported files in the current directory
//             File tCurrentDir = new File(System.getProperty("user.dir"));
//             File[] tFiles = tCurrentDir.listFiles(new FilenameFilter() {
//                 public boolean accept(final File dir, final String name) {
//                     String[] imageFormats = ImageIO.getReaderFormatNames();
//                     for (int i = 0; i < imageFormats.length; i++) {
//                         if (name.endsWith(imageFormats[i])) {
//                             return true;
//                         }
//                     }
//                     return false;
//                 }
//             });
//             for (int i = 0; i < tFiles.length; i++) {
//                 tInputFiles.add(tFiles[i]);
//             }
//             if (tInputFiles.size() == 0) {
//                 System.err.println("Failed to find any images to process");
//                 System.err.println("Exiting....");
//                 System.exit(-1);
//             }
//             System.out.println("Found " + tInputFiles.size() + " image files.");
//         }
//         if (tCmd.hasOption("identifier")) {
//             identifier_root = tCmd.getOptionValue("identifier");
//         }
//         int tMaxFileNo = -1;
//         if (tCmd.hasOption("zoom_levels")) {
//             tZoom = Integer.parseInt(tCmd.getOptionValue("zoom_levels"));
//         }

//         if (tCmd.hasOption("version")) {
//             if (tCmd.getOptionValue("version").contains("2")) {
//                 tVersion = InfoJson.VERSION211;
//             } else if  (tCmd.getOptionValue("version").contains("3")) {
//                 tVersion = InfoJson.VERSION3;
//             } else {
//                 System.err.println("Unrecognised version '" + tCmd.getOptionValue("version") + "' value can either be 2 or 3.");

//                 formatter.printHelp("iiif-tiler", tOptions);
//             }
//         }

//         if (tCmd.hasOption("tile_size")) {
//             tTilesize = Integer.parseInt(tCmd.getOptionValue("tile_size"));
//         }

//         System.out.println("Zoom level " + tZoom);
//         File tOutputDir = new File(outputDir);
//         if (tCmd.hasOption("output")) {
//             tOutputDir = new File(tCmd.getOptionValue("output"));
//         }

//         //String tVersion = InfoJson.VERSION3;
//         createImages(tInputFiles, tOutputDir, identifier_root, tZoom, tMaxFileNo, tTilesize, tVersion);
//     }
// }