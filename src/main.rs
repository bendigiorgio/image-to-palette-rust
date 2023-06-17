mod image_tools;
#[macro_use]
extern crate rocket;

use std::fs;
use std::time::Duration;
use rocket::fairing::{ Fairing, Info, Kind };
use rocket::fs::{ NamedFile, TempFile };
use rocket::http::{ Header, Method, Status };
use rocket::{ Request, Response };
use rocket::response::status;
use rocket::serde::{ json::Json, Deserialize, Serialize };

use tokio::{ task, time };

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

pub struct CORS;
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if request.method() == Method::Options {
            response.set_status(Status::NoContent);
            response.set_header(
                Header::new("Access-Control-Allow-Methods", "POST, PATCH, GET, DELETE")
            );
            response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        }

        response.set_header(Header::new("Access-Control-Allow-Origin", "http://localhost:3000"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
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

#[post("/make_palette_image?<id>", data = "<url>")]
async fn make_palette_image_from_url(
    url: Json<Url<'_>>,
    id: String
) -> Result<Option<NamedFile>, status::Custom<String>> {
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
        let output_file = format!("./output/{}.png", id);
        image_tools::create_image(&output_file, result.unwrap());

        // Remove file after 45 seconds
        task::spawn(async move {
            time::sleep(Duration::from_secs(45)).await;
            fs::remove_file(format!("./output/{}.png", id)).ok();
        });

        Some(NamedFile::open(output_file).await.ok()).ok_or(
            status::Custom(
                rocket::http::Status::InternalServerError,
                "Internal Server Error".to_string()
            )
        )
    }
}

#[launch]
fn rocket() -> _ {
    rocket
        ::build()
        .attach(CORS)
        .mount("/", routes![palette_from_url, make_palette_image_from_url, palette_from_image])
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
