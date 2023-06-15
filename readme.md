# Palette Extractor

Welcome to the Palette Extractor application! This Rust-based program will take an image (either from a local file or a URL) and create a color palette based on that image. The output will be a collection of hexadecimal color values which are derived from the dominant colors in the image.

The application uses the Rocket framework to expose two RESTful endpoints, allowing you to extract color palettes by sending either a direct image file or a URL pointing to an image.

## Dependencies

This program depends on several Rust crates:

- Rocket for creating web endpoints.
- Reqwest for handling HTTP requests.
- Image for image processing.
- Hex-color for converting RGB values to hexadecimal color values.
- Anyhow for error handling.

## How to Use

1. Start the application by running the command `cargo run`.
2. The application will launch a local server at `localhost:8000`.

There are two endpoints you can use:

### Endpoint 1: /make_palette

- Method: POST
- Data: JSON object containing "link" (string) and "iterations" (integer)
- Example usage with curl:

  ```shell
  curl -X POST -H "Content-Type: application/json" -d '{"link":"https://example.com/image.jpg", "iterations":5}' http://localhost:8000/make_palette
  ```

- Description: This endpoint will download the image from the provided URL, generate a color palette, and return the palette as a JSON array of hexadecimal color values.

### Endpoint 2: /palette_from_image

- Method: POST
- Data: multipart form-data with the image file and "iterations" as query parameter
- Example usage with curl:

  ```shell
  curl -X POST -F "file=@path/to/image.jpg" http://localhost:8000/palette_from_image?iterations=5
  ```

- Description: This endpoint will read the uploaded image file, generate a color palette, and returns a named file which is deleted after 45 seconds.

## Note

The number of colors in the palette is determined by the "iterations" parameter. The actual number of colors produced will be 2 to the power of the number of iterations.

For example, if you provide "iterations" as 4, the number of colors in the palette will be 2^4 = 16.

## Limitations

This program does not yet handle all potential error conditions. Also, it does not provide extensive customization for color palette creation. It is a simple application for extracting dominant colors from an image.

Enjoy creating beautiful palettes!
