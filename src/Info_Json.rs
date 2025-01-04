/**
 * This class generates the IIIF info.json for an image
 */
use serde_json::{json, Value, Map};

use crate::Image_Info::ImageInfo;

#[derive(Debug, PartialEq)]
pub enum IIIFVersion {
    VERSION3,
    VERSION211,
 }

 impl Default for IIIFVersion {
    fn default() -> Self {
        IIIFVersion::VERSION211
    }
 }

#[derive(Debug, PartialEq)]
pub struct InfoJSON<'a> {
    image_info: &'a ImageInfo<'a>,
    uri: String,
    version: &'a IIIFVersion
}

impl<'a> InfoJSON<'a> {
    pub fn new(image_info: &'a ImageInfo, uri: &str, version: &'a IIIFVersion) -> InfoJSON<'a> {
        InfoJSON {
            image_info,
            uri: uri.to_string(),
            version: version
        }
    }

    pub fn id(&self) -> String {
        format!("{}{}", self.uri, self.image_info.id())
    }

    pub fn width(&self) -> i32 {
        self.image_info.get_width()
    }

    pub fn height(&self) -> i32 {
        self.image_info.get_height()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let mut info_json = Map::new();

        match self.version {
            IIIFVersion::VERSION3 => {
                info_json.insert("@context".to_owned(), Value::String("http://iiif.io/api/image/3/context.json".to_owned()));
                info_json.insert("id".to_owned(), Value::String(self.id()));
                info_json.insert("type".to_owned(), Value::String("ImageService3".to_string()));
                info_json.insert("profile".to_owned(), Value::String("level0".to_string())); 
            } 
            IIIFVersion::VERSION211 => {
                info_json.insert("@context".to_owned(), Value::String("http://iiif.io/api/image/2/context.json".to_owned()));
                info_json.insert("id".to_owned(), Value::String(self.id()));
                info_json.insert("profile".to_owned(), Value::String("http://iiif.io/api/image/2/level0.json".to_owned()));
            }
        }

        info_json.insert("protocol".to_owned(), Value::String("http://iiif.io/api/image".to_owned()));
        info_json.insert("width".to_owned(), Value::Number(self.width().into()));
        info_json.insert("height".to_owned(), Value::Number(self.height().into()));
        
        // Add sizes
        let sizes_json: Vec<Value> = self
            .image_info
            .get_sizes()
            .iter()
            .map(|&(x, y)| json!({ "height": y, "width": x }))
            .collect();
        info_json.insert("sizes".to_owned(), Value::Array(sizes_json));

        // Add tiles
        let tiles_json = vec![json!({
            "width": self.image_info.get_tile_width(),
            "height": self.image_info.get_tile_height(),
            "scaleFactors": self.image_info.get_scale_factors()
        })];
        info_json.insert("tiles".to_owned(), Value::Array(tiles_json));

        serde_json::to_string(&info_json)
    }
}
