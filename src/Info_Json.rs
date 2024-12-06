/**
 * This class generates the IIIF info.json for an image
 */
use std::collections::HashMap;
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
        let mut info_json: HashMap<&str,String> = HashMap::new();

        if self.version == "3.0" {
            info_json.insert("@context", "http://iiif.io/api/image/3/context.json".to_string());
            info_json.insert("id", self.id());
            info_json.insert("type", "ImageService3".to_string());
            info_json.insert("profile", "level0".to_string());
        } else {
            info_json.insert("@context", "http://iiif.io/api/image/2/context.json".to_string());
            info_json.insert("id", self.id());
            info_json.insert("profile", "http://iiif.io/api/image/2/level0.json".to_string());
        }

        info_json.insert("protocol", "http://iiif.io/api/image".to_string());
        info_json.insert("width", serde_json::to_string(&self.width()).unwrap());
        info_json.insert("height", serde_json::to_string(&self.height()).unwrap());
        let mut sizes_json = Vec::<HashMap::<String,i32>>::new(); 
        for size in self.image_info.get_sizes() {
            let (x,y) = size;
            let mut size_map = HashMap::<String,i32>::new();
            size_map.insert("height".to_string(),y);
            size_map.insert("width".to_string(),x);
            sizes_json.push(size_map);
        }
        info_json.insert("sizes", serde_json::to_string(&sizes_json).unwrap());

        let mut tiles_list = Vec::<HashMap<String,String>>::new();
        let mut tiles_map = HashMap::<String,String>::new();
        tiles_map.insert("width".to_string(), serde_json::to_string(&self.image_info.get_tile_width()).unwrap());
        tiles_map.insert("height".to_string(), serde_json::to_string(&self.image_info.get_tile_height()).unwrap());
        tiles_map.insert("scaleFactors".to_string(), serde_json::to_string(&self.image_info.get_scale_factors()).unwrap());
        tiles_list.push(tiles_map);
        info_json.insert("tiles", serde_json::to_string(&tiles_list).unwrap());

        return serde_json::to_string(&info_json).unwrap();
    }
}
