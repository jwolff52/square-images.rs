use std::fs;

use image::{GenericImageView, GenericImage};

const DEFAULT_OLD_DIR: &str = "../images/old";
const DEFAULT_NEW_DIR: &str = "../images/new";
const DEFAULT_TILE_IMAGE: &str = "../images/tile.png";

fn main() {
    let image_names = get_image_names(&DEFAULT_OLD_DIR);
    let tile_image = get_tile_image(&DEFAULT_TILE_IMAGE);
    let old_images = get_old_images(&image_names);

    for x in 0..old_images.len() {
        create_new_image(&old_images.get(x).unwrap(), &tile_image, &image_names.get(x).unwrap());
    }
}

fn create_new_image(image: &image::DynamicImage, tile_image: &image::DynamicImage, image_name: &String) {
    let (width, height) = image.dimensions();
    let max_dimension = width.max(height);
    let mut new_image = image::DynamicImage::new_rgb8(max_dimension, max_dimension);
    for x in (0..max_dimension).step_by(tile_image.dimensions().0.try_into().unwrap()) {
        for y in (0..max_dimension).step_by(tile_image.dimensions().1.try_into().unwrap()) {
            new_image.copy_from(tile_image, x, y).unwrap();
        }
    }
    new_image.copy_from(image, (max_dimension - width) / 2, (max_dimension - height) / 2).unwrap();
    new_image.save(format!("{}/{}", DEFAULT_NEW_DIR, image_name)).unwrap();
}

fn get_old_images(image_names: &[String]) -> Vec<image::DynamicImage> {
    let mut old_images = Vec::new();
    for image_name in image_names {
        let image_path = format!("{}/{}", DEFAULT_OLD_DIR, image_name);
        let image = image::open(image_path);
        match &image {
            Ok(image) => {
                old_images.push(image.clone());
                println!("Image {} loaded", image_name)
            },
            Err(_) => println!("File {} is not an image", image_name),
        }
    }
    old_images
}

fn get_tile_image(default_tile_image: &str) -> image::DynamicImage {
    image::open(default_tile_image).unwrap()
}

fn get_image_names(dir: &str) -> Vec<String> {
    let mut image_names = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        image_names.push(file_name.to_string());
    }
    image_names
}
