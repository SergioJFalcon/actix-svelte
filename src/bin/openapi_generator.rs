// This imports the `get_api_doc` function from our main library
use my_actix_server::get_api_doc;

// This binary's sole purpose is to print the OpenAPI JSON to standard output.
fn main() {
    let doc = get_api_doc();
    // Use serde_json to serialize the OpenApi struct into a pretty JSON string
    let json_spec = serde_json::to_string_pretty(&doc).unwrap();
    println!("{}", json_spec);
}