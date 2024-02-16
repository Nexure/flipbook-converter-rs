use std::path::PathBuf;

use image::{DynamicImage, GenericImage};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 's', long)]
    spritesheet: PathBuf,
    #[arg(short = 'g', long)]
    spritesheet_grid: String,

    #[arg(short, long)]
    output: PathBuf,
    #[arg(short = 'p', long)]
    output_grid: String,
    #[arg(short = 'v', long)]
    output_size: Option<String>,
}

fn read_grid(grid: String) -> (u32, u32) {
    let mut split = grid.split('x');
    let width = split.next().unwrap().parse().unwrap();
    let height = split.next().unwrap().parse().unwrap();
    (width, height)
}

fn split_images(image: &DynamicImage, grid: (u32, u32)) -> Vec<DynamicImage> {
    let width = image.width();
    let height = image.height();

    let mut images = Vec::new();

    for y in 0..grid.1 {
        for x in 0..grid.0 {
            let x = x * width / grid.0;
            let y = y * height / grid.1;

            let image = image.crop_imm(x, y, width / grid.0, height / grid.1);
            images.push(image);
        }
    }

    images
}

fn fill_images(image: &mut DynamicImage, images: Vec<DynamicImage>, grid: (u32, u32)) {
    let thumb_width = image.width() / grid.0;
    let thumb_height = image.height() / grid.1;

    let mut index = 0;

    for y in 0..grid.1 {
        for x in 0..grid.0 {
            let x = x * thumb_width;
            let y = y * thumb_height;

            let sub_image = match images.get(index) {
                Some(image) => image,
                None => break,
            };

            let resized = sub_image.resize_exact(
                thumb_width,
                thumb_height,
                image::imageops::FilterType::Gaussian,
            );

            image.copy_from(&resized, x, y).unwrap();
            index += 1;
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let input_grid = read_grid(args.spritesheet_grid);
    let output_grid = read_grid(args.output_grid);
    let mut output_size = match args.output_size {
        Some(size) => read_grid(size),
        None => (0, 0),
    };

    assert!(
        input_grid.0 * input_grid.1 <= output_grid.0 * output_grid.1,
        "Output grid is too small"
    );

    let input_image = image::open(args.spritesheet).unwrap();
    if output_size.0 == 0 && output_size.1 == 0 {
        output_size = (input_image.width(), input_image.height());
    }

    let split_image = split_images(&input_image, input_grid);
    let mut output_image = DynamicImage::new(output_size.0, output_size.1, input_image.color());

    fill_images(&mut output_image, split_image, output_grid);
    output_image.save(args.output).unwrap();
}
