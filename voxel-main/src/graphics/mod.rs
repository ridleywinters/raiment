use image;
use kiss3d::resource::TextureManager;

// TODO: This saves the atlas to file and reloads it for two reasons:
// - Debugability of being able see the file after it is created
// - I never wrote the in-memory only version :)
//
fn create_tile_atlas(atlas_filename: &str) {
    // for each in src/assets/tiles with n > 0
    // load & paste into larger image
    // save image

    use glob::glob;

    let mut paths = Vec::new();
    for entry in glob("src/assets/tiles/[0-9][0-9]*.png").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => paths.push(path),
            Err(e) => println!("{:?}", e),
        }
    }

    paths.sort_by(|a, b| alphanumeric_sort::compare_path(a, b));

    let mut imgbuf = image::ImageBuffer::new(256, 256);

    // Skip the 00_ tile
    for i in 1..paths.len() {
        let path = &paths[i];

        let img1 = image::open(path).unwrap();
        let img = img1.as_rgba8().unwrap();

        // The dimensions method returns the images width and height.
        let dims = img.dimensions();
        println!("Loaded tile {}x{} {}", dims.0, dims.1, path.display());

        let offset = (i as u32 - 1) * 16;
        let ox: u32 = offset % 256;
        let oy: u32 = offset / 256;
        for (x, y, pixel) in img.enumerate_pixels() {
            let dst = imgbuf.get_pixel_mut(x + ox, y + oy);
            *dst = *pixel;
        }
    }
    imgbuf.save(atlas_filename).unwrap();
}

pub fn create_texture_atlas() -> TextureManager {
    let filename = "texture_atlas.png";

    create_tile_atlas(filename);

    let mut tm = TextureManager::new();
    tm.add(std::path::Path::new(filename), "tiles");
    tm
}
