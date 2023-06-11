use image::{ Rgba, RgbaImage, ImageBuffer };
use image::error::ImageResult as ImageResult;
use image::io::Reader as ImageReader;
use hex_color::HexColor;
use anyhow::Result;
use rocket::futures::TryFutureExt;
use rocket::http::hyper::body::Buf;
use std::io::{ copy, Cursor };
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChannel {
    R,
    G,
    B,
    A,
}

impl ColorChannel {
    pub const ALL: [ColorChannel; 4] = [Self::R, Self::G, Self::B, Self::A];

    pub fn value(&self, color: &Color) -> u8 {
        match self {
            ColorChannel::R => color.r,
            ColorChannel::G => color.g,
            ColorChannel::B => color.b,
            ColorChannel::A => color.a,
        }
    }
}

impl Color {
    pub fn from_fn<F>(f: F) -> Option<Color> where F: Fn(ColorChannel) -> Option<u8> {
        Some(Color {
            r: f(ColorChannel::R)?,
            g: f(ColorChannel::G)?,
            b: f(ColorChannel::B)?,
            a: f(ColorChannel::A)?,
        })
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        ColorChannel::ALL.iter().all(|channel| channel.value(self) == channel.value(other))
    }
}

/// Returns the color channel with the highest range.
/// IMPORTANT: Ignores alpha channel!
fn highest_range_channel(colors: &Vec<Color>) -> Option<ColorChannel> {
    let ranges = color_ranges(colors)?;
    let channel: &ColorChannel = ColorChannel::ALL.iter().max_by_key(|channel|
        channel.value(&ranges)
    )?;
    Some(*channel)
}

/// Returns the ranges for each color channel
fn color_ranges(colors: &Vec<Color>) -> Option<Color> {
    Color::from_fn(|channel|
        Some(
            colors
                .iter()
                .map(|color| channel.value(color))
                .max()? -
                colors
                    .iter()
                    .map(|color| channel.value(color))
                    .min()?
        )
    )
}

/// Returns median value for a specific `ColorChannel` across `colors`.
fn channel_median(colors: &mut Vec<Color>, channel: &ColorChannel) -> Option<u8> {
    colors.sort_by_key(|a| channel.value(a));

    let mid = colors.len() / 2;
    if colors.len() % 2 == 0 {
        channel_mean(&vec![colors[mid - 1], colors[mid]], channel)
    } else {
        Some(channel.value(colors.get(mid)?))
    }
}

/// Calculate the mean value for a specific color channel on a vector of `Color`.
fn channel_mean(colors: &Vec<Color>, channel: &ColorChannel) -> Option<u8> {
    let number_colors = colors.len() as u32;

    if number_colors == 0 {
        return None;
    }

    let mean = colors.iter().fold(0, |acc: u32, x| (channel.value(x) as u32) + acc) / number_colors;
    Some(mean as u8)
}

/// Performs the median cut on a single vector (bucket) of `Color`.
/// Returns two vectors (buckets) with the colors above and below the chosen median.
fn median_cut(colors: &mut Vec<Color>) -> Option<(Vec<Color>, Vec<Color>)> {
    let mut above_median = Vec::<Color>::new();
    let mut below_median = Vec::<Color>::new();
    let channel = highest_range_channel(&colors)?;
    let median = channel_median(colors, &channel)?;

    for color in colors {
        if channel.value(color) > median {
            above_median.push(*color);
        } else {
            below_median.push(*color);
        }
    }

    return Some((above_median, below_median));
}

/// Perform the median cut algorithm.
/// Returns a palette with 2^iter_count colors.
/// https://en.wikipedia.org/wiki/Median_cut
pub fn make_palette(bucket: &mut Vec<Color>, iter_count: u8) -> Option<Vec<Color>> {
    if iter_count < 1 {
        return Some(vec![Color::from_fn(|channel| channel_mean(bucket, &channel))?]);
    }

    let mut new_buckets = median_cut(bucket)?;
    let mut result = make_palette(&mut new_buckets.0, iter_count - 1)?;
    result.append(&mut make_palette(&mut new_buckets.1, iter_count - 1)?);
    Some(result)
}

/// Convert from an image::RgbaImage to the local representation.
fn read_pixels(image: RgbaImage) -> Vec<Color> {
    let to_color = |p: Rgba<u8>| Color { r: p[0], g: p[1], b: p[2], a: p[3] };
    let mut pixels = Vec::new();
    pixels.extend(image.pixels().map(|p| to_color(*p)));
    pixels
}

/// Read an image from disk. Returns the RgbaImage and its width and height.
fn read_image(filename: String) -> ImageResult<(RgbaImage, u32, u32)> {
    let image = ImageReader::open(&filename)?.decode()?.to_rgba8();
    let (w, h) = (image.width(), image.height());
    Ok((image, w, h))
}

/// Make a copy of colors, with color is replaced by the closest match in palette.
/// Uses Pythagorean distance.
fn _assign_colors(colors: Vec<Color>, palette: Vec<Color>) -> Vec<Color> {
    // Skip the sqrt, we don't need it. Use u32 to prevent overflows.
    let channel_distance = |a: u8, b: u8| ((if a < b { b - a } else { a - b }) as u32).pow(2);
    let mut result = Vec::new();
    result.extend(
        colors.iter().map(
            |c|
                palette
                    .iter()
                    .min_by_key(|p|
                        ColorChannel::ALL.iter()
                            .map(|channel| channel_distance(channel.value(c), channel.value(p)))
                            .sum::<u32>()
                    )
                    .unwrap() // Maybe we should check if the palette is empty?
        )
    );
    result
}

/// Convert from the local representation to an image::RgbaImage.
/// Could theoretically panic in a variety of ways.
fn _gather_pixels(colors: Vec<Color>, width: u32, height: u32) -> RgbaImage {
    let from_c = |c: Color| Rgba::<u8>::from([c.r, c.g, c.b, c.a]);
    RgbaImage::from_fn(width, height, |x, y|
        from_c(colors[usize::try_from(x + width * y).unwrap()])
    )
}

/// The inner "main" function; wraps failure conditions.
fn _handle_file_w_output(input_file: String, output_file: String) -> Result<(), String> {
    let e2str = |e| format!("{}", e);
    let (image, width, height) = read_image(input_file).map_err(e2str)?;
    let pixels = read_pixels(image);
    let palette = make_palette(&mut pixels.clone(), 4).ok_or(
        String::from("There was a problem building the palette.")
    )?;
    println!("{} colors!", palette.len());
    let out_image = _gather_pixels(_assign_colors(pixels, palette), width, height);
    out_image.save(output_file).map_err(e2str)
}

//Number of colors produced is iteration^2
pub fn handle_file(input_file: String, iterations: u8) -> Result<Vec<Color>, String> {
    let e2str = |e| format!("{}", e);
    let (image, _width, _height) = read_image(input_file).map_err(e2str)?;
    let pixels = read_pixels(image);
    let palette = make_palette(&mut pixels.clone(), iterations).ok_or(
        String::from("There was a problem building the palette.")
    );
    palette
}

pub async fn handle_file_from_url(
    input_url: String,
    iterations: u8
) -> Option<Result<Vec<Color>, String>> {
    let e2str = |e| format!("{}", e);
    let response = reqwest
        ::get(input_url).await
        .map_err(e2str)
        .and_then(|response| response.error_for_status().map_err(e2str));

    let response = match response {
        Ok(response) => response.bytes(),
        Err(e) => {
            return Some(Err(e));
        }
    };
    let content = response.await.map_err(e2str);
    let content = match content {
        Ok(content) => content,
        Err(e) => {
            return Some(Err(e));
        }
    };

    // let img_bytes = reqwest::blocking::get(input_url).map_err(e2str)?.bytes().map_err(e2str)?;

    let image = image::load_from_memory(&content);
    let image = match image {
        Ok(image) => image,
        Err(e) => {
            return Some(Err("Could not load image from url".to_string()));
        }
    };

    let pixels = read_pixels(image.to_rgba8());
    let palette = make_palette(&mut pixels.clone(), iterations).ok_or(
        String::from("There was a problem building the palette.")
    );
    Some(palette)
}

pub fn name_from_rgb(colors: &Vec<Color>) -> Vec<HexColor> {
    let mut palette = Vec::new();
    for color in colors {
        palette.push(HexColor::rgb(color.r, color.g, color.b));
    }
    for color in &palette {
        println!("{}", color);
    }
    palette
}

pub fn create_image(output_file: String, colors: Vec<Color>) {
    let square_size: u32 = 50;
    let width = square_size * (colors.len() as u32);
    let height = square_size;

    // Create a new image with the specified width and height
    let mut img: RgbaImage = ImageBuffer::new(width, height);

    // Iterate over the colors and fill each square in the image with a color from the palette
    for (i, color) in colors.iter().enumerate() {
        for x in 0..square_size {
            for y in 0..square_size {
                let pixel = img.get_pixel_mut((i as u32) * square_size + x, y);
                *pixel = Rgba([color.r, color.g, color.b, color.a]);
            }
        }
    }

    // Save the output image
    img.save(output_file).unwrap();
}
