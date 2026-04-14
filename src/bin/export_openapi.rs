use stim_server::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let document = ApiDoc::openapi();
    println!("{}", serde_json::to_string_pretty(&document).unwrap());
}
