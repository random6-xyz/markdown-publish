use config::{Config, File, FileFormat};
use lazy_static::lazy_static;
use pulldown_cmark::{html, Parser};
use rocket::{
    build,
    data::{Data, ToByteUnit},
    get,
    http::Status,
    post,
    request::{FromRequest, Outcome, Request},
    response::content,
    routes,
};
use std::{fs, io::Error};

lazy_static! {
    static ref APIKEY: String = {
        let builder = Config::builder().add_source(File::new("config/settings", FileFormat::Toml));

        let config = builder.build().expect("No `config.toml` file");
        config
            .get_string("api_key")
            .expect("No name api_key in config.toml")
            .to_owned()
    };
}

#[allow(dead_code)]
struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(key: &str) -> bool {
            key == APIKEY.as_str()
        }

        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Error((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

#[post("/upload/<file_name>", format = "bytes", data = "<data>")]
async fn upload(_key: ApiKey<'_>, file_name: &str, data: Data<'_>) -> Status {
    if !check_file_name(file_name) {
        return Status::new(422);
    }

    match data
        .open(10.megabytes())
        .into_file(format!("./markdown/{}.md", file_name))
        .await
    {
        Err(_) => return Status::new(422),
        Ok(_) => {
            let html = parse_md_to_html(file_name);
            match save_html_to_file(file_name, html) {
                Err(_) => return Status::new(422),
                Ok(()) => return Status::new(200),
            };
        }
    };
}

#[get("/delete/<file_name>")]
fn delete(_key: ApiKey<'_>, file_name: String) -> Status {
    if !check_file_name(file_name.as_str()) {
        return Status::new(422);
    }

    match fs::remove_file(format!("./markdown/{}", file_name)) {
        Err(_) => return Status::new(422),
        Ok(_) => {
            match fs::remove_file(format!("./html/{}", file_name)) {
                Err(_) => return Status::new(422),
                Ok(_) => return Status::new(200),
            };
        }
    }
}

#[get("/upload_list")]
fn setting(_key: ApiKey<'_>) -> content::RawHtml<String> {
    let mut list_dir: Vec<String> = Vec::new();
    for file in fs::read_dir("./markdown").unwrap() {
        list_dir.push(file.unwrap().file_name().into_string().unwrap());
    }

    let result = list_dir
        .iter()
        .map(|x| format!("<li>{}</li>", x))
        .collect::<Vec<String>>()
        .join("\n");

    content::RawHtml(format!(
        r#"<!DOCTYPE html><html><body><ul>{}</ul></body></html>"#,
        result
    ))
}

#[get("/publish/<file_path>")]
fn publish(file_path: String) -> content::RawHtml<String> {
    match fs::read_to_string(format!("./html/{}.html", file_path)) {
        Err(_) => content::RawHtml("error".to_string()),
        Ok(data) => content::RawHtml(format!(
            r#"<!DOCTYPE html><html><body>{}</body></html>"#,
            data
        )),
    }
}

#[rocket::launch]
fn rocket() -> _ {
    setup().expect("Can't create directory");

    build()
        .configure(rocket::Config::figment().merge(("port", 8080)))
        .mount("/", routes![publish, upload, setting, delete])
}

fn setup() -> Result<(), Error> {
    fs::create_dir_all("./markdown")?;
    fs::create_dir_all("./html")?;

    Ok(())
}

fn check_file_name(file_name: &str) -> bool {
    file_name
        .chars()
        .all(|c| char::is_ascii_alphanumeric(&c) || c == '_' || c == ' ')
}

fn save_html_to_file(file_name: &str, html: String) -> Result<(), Error> {
    let file_path = format!("./html/{}.html", file_name);
    fs::File::create(file_path.clone())?;
    fs::write(file_path, html)?;
    Ok(())
}

fn parse_md_to_html(file_name: &str) -> String {
    let md = fs::read_to_string(format!("./markdown/{}", file_name)).expect("no file found");
    let parser = Parser::new(&md);

    let mut html = String::new();
    html::push_html(&mut html, parser);
    html
}
