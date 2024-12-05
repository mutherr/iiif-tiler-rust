/**
 * This class generates the IIIF info.json for an image
 */
use std::collections::HashMap;
use std::any::Any;
use crate::Image_Info::ImageInfo;

pub struct InfoJSON {
    image_info: ImageInfo,
    uri: String,
    version: String
}

enum Value {
    Integer(i32),
    String(String),
    Sizes(Vec::<HashMap::<String,i32>>),
    ScaleFactors(Vec::<i32>),
    Tiles(Vec::<HashMap<String, Value>>),
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

    pub fn to_json(&self) -> HashMap<&str, Value> {
        let mut info_json: HashMap<&str, Value> = HashMap::new();

        if self.version == "3.0" {
            info_json.insert("@context", Value::String("http://iiif.io/api/image/3/context.json".to_string()));
            info_json.insert("id", Value::String(self.id()));
            info_json.insert("type", Value::String("ImageService3".to_string()));
            info_json.insert("profile", Value::String("level0".to_string()));
        } else {
            info_json.insert("@context", Value::String("http://iiif.io/api/image/2/context.json".to_string()));
            info_json.insert("id", Value::String(self.id()));
            info_json.insert("profile", Value::String("http://iiif.io/api/image/2/level0.json".to_string()));
        }

        info_json.insert("protocol", Value::String("http://iiif.io/api/image".to_string()));
        info_json.insert("width", Value::Integer(self.width()));
        info_json.insert("height", Value::Integer(self.height()));
        let mut sizes_json = Vec::<HashMap::<String,i32>>::new(); 
        for size in self.image_info.get_sizes() {
            let (x,y) = size;
            let mut size_map = HashMap::<String,i32>::new();
            size_map.insert("width".to_string(),x);
            size_map.insert("height".to_string(),y);
            sizes_json.push(size_map);
        }
        info_json.insert("sizes", Value::Sizes(sizes_json));

        let mut tiles_list = Vec::<HashMap<String, Value>>::new();
        let mut tiles_map = HashMap::<String, Value>::new();
        tiles_map.insert("width".to_string(), Value::Integer(self.image_info.get_tile_width()));
        tiles_map.insert("height".to_string(), Value::Integer(self.image_info.get_tile_height()));
        tiles_map.insert("scaleFactors".to_string(), Value::ScaleFactors(self.image_info.get_scale_factors()));
        tiles_list.push(tiles_map);
        info_json.insert("tiles", Value::Tiles(tiles_list));

        return info_json;
    }
}
