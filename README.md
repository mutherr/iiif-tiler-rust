# IIIF Tiler Rust

This project generates tiles for IIIF images as well as creating a manifest.json file. 

It may be run as follows:

`iiif-tiler-rust [options] <image path>`

So an example invocation would be (assuming the binary is in the user's PATH)

`iiif-tiler-rust example.png`

which would create a folder `iiif/example` to store the image's `info.json` and the tiles with 5 zoom levels, 1024 pixel tiles using the identifier `http://localhost:8887/iiif/` in the manifest using IIIF version 2

# Options

and supports the following options

```bash
  -u, --uri <URI>                    Set the identifier in the mainfest [default: http://localhost:8887/iiif/]
  -i, --iiif-version <IIIF_VERSION>  Set the IIIF version, options are `2` or `3` [default: 2]
  -z, --zoom-levels <ZOOM_LEVELS>    Set the number of zoom levels for this image [default: 5]
  -t, --tile-size <TILE_SIZE>        Set the tile size [default: 1024]
  -o, --output-dir <OUTPUT_DIR>      Directory where the image tiles are stored [default: iiif]
  -v, --verbose                      Enable verbose logging
  -h, --help                         Print help
  -V, --version                      Print version
```

The tiler supports jpg/jpeg, png, bmp, and tiff format images

If you have feedback or questions, feel free to reach out to me at ryan_muthefas.harvard.edu.