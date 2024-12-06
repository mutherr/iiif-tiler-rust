/**
 * This class generates the IIIF info.json for an image
 */
use std::collections::HashMap;
use serde_json::{json, Value, Map};

use crate::Image_Info::ImageInfo;

pub struct InfoJSON {
    image_info: ImageInfo,
    uri: String,
    version: String
}

impl InfoJSON {
    pub fn new(image_info: ImageInfo, uri: String, version: String) -> InfoJSON {
        InfoJSON {
            image_info,
            uri,
            version
        }
    }

    pub fn id(&self) -> String {
        return self.uri.clone() + &self.image_info.id();
    }

    pub fn width(&self) -> i32 {
        return self.image_info.get_width()
    }

    pub fn height(&self) -> i32 {
        return self.image_info.get_height()
    }

    pub fn to_json(&self) -> String {
        let mut info_json = Map::new();

        if self.version == "3.0" {
            info_json.insert("@context".to_string(), Value::String("http://iiif.io/api/image/3/context.json".to_string()));
            info_json.insert("id".to_string(), Value::String(self.id()));
            info_json.insert("type".to_string(), Value::String("ImageService3".to_string()));
            info_json.insert("profile".to_string(), Value::String("level0".to_string()));
        } else {
            info_json.insert("@context".to_string(), Value::String("http://iiif.io/api/image/2/context.json".to_string()));
            info_json.insert("id".to_string(), Value::String(self.id()));
            info_json.insert("profile".to_string(), Value::String("http://iiif.io/api/image/2/level0.json".to_string()));
        }

        info_json.insert("protocol".to_string(), Value::String("http://iiif.io/api/image".to_string()));
        info_json.insert("width".to_string(), Value::String(serde_json::to_string(&self.width()).unwrap()));
        info_json.insert("height".to_string(), Value::String(serde_json::to_string(&self.height()).unwrap()));
        let mut sizes_json = Vec::<Value>::new(); 
        for size in self.image_info.get_sizes() {
            let (x,y) = size;
            let size_str = json!({ "height": y, "width": x });
            println!("{}",size_str);
            sizes_json.push(size_str);
        }
        info_json.insert("sizes".to_string(), Value::String(serde_json::to_string(&sizes_json).unwrap()));

        let tiles_list: Vec<Value> = vec![json!({
            "width": self.image_info.get_tile_width(),
            "height": self.image_info.get_tile_height(),
            "scaleFactors": self.image_info.get_scale_factors()
        })];
        info_json.insert("tiles".to_string(), Value::Array(tiles_list));

        return serde_json::to_string(&info_json).unwrap();
    }
}
