mod image_tools;
#[macro_use]
extern crate rocket;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::{ Serialize, Deserialize, json::Json };
use rocket::fs::TempFile;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Url<'r> {
    link: &'r str,
    iterations: u8,
}
#[derive(Serialize)]
struct PaletteResponse {
    colors: Vec<String>,
}

#[post("/make_palette", data = "<url>")]
async fn palette_from_url(
    url: Json<Url<'_>>
) -> Result<Json<PaletteResponse>, status::Custom<String>> {
    let result = image_tools::handle_file_from_url(url.link.to_string(), url.iterations).await;
    let result = match result {
        Some(result) => result,
        None => {
            return Err(
                status::Custom(Status::InternalServerError, format!("Internal Server Error"))
            );
        }
    };
    if let Err(error) = result {
        println!("{}", error);
        return Err(status::Custom(Status::InternalServerError, format!("{}", error)));
    } else {
        println!("HEX code for colors from URL:");
        let palette = image_tools::name_from_rgb(result.as_ref().unwrap());
        println!("Success!");
        Some(
            Json(PaletteResponse {
                colors: palette
                    .iter()
                    .map(|color| color.to_string())
                    .collect(),
            })
        ).ok_or(
            status::Custom(
                rocket::http::Status::InternalServerError,
                "Internal Server Error".to_string()
            )
        )
    }
}

#[post("/palette_from_image?<iterations>", format = "plain", data = "<file>")]
fn palette_from_image(file: TempFile<'_>, iterations: u8) {
    let result = image_tools::handle_file(
        file.path().unwrap().to_string_lossy().to_string(),
        iterations
    );
    if let Err(error) = result {
        println!("{}", error);
    } else {
        println!("HEX code for colors from local image:");
        image_tools::name_from_rgb(result.as_ref().unwrap());
        println!("Success!");
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![palette_from_url]).mount("/", routes![palette_from_image])
}

// fn main() {
//     let input_file = String::from(
//         "/Users/bendigiorgio/Documents/Programming/_RUST/image-to-palette/src/test.png"
//     );
//     let output_file = String::from("./output.png");
//     let input_url = String::from(
//         "https://cdn.midjourney.com/d04e1165-c3a9-4983-8513-e25e2fdba946/0_0.png"
//     );

//     let result = image_tools::handle_file(input_file, 4);
//     if let Err(error) = result {
//         println!("{}", error);
//     } else {
//         println!("HEX code for colors from local image:");
//         image_tools::name_from_rgb(result.as_ref().unwrap());
//         image_tools::create_image(output_file, result.unwrap());
//         println!("Success!");
//     }

//     let result = image_tools::handle_file_from_url(input_url, 4);
//     if let Err(error) = result {
//         println!("{}", error);
//     } else {
//         println!("HEX code for colors from URL:");
//         image_tools::name_from_rgb(result.as_ref().unwrap());
//         println!("Success!");
//     }
// }
