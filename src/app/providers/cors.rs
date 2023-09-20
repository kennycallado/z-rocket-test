use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

use crate::app::providers::config_getter::ConfigGetter;

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to response",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = ConfigGetter::get_origin_url().unwrap_or("*".to_string());

        response.set_header(Header::new("Access-Control-Allow-Origin", origin));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PUT, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
