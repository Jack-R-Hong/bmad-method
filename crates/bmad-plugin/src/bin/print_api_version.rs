// #[path] lets this binary share the exact constant the lib target uses,
// avoiding any hardcoded value that could silently drift from the real version.
#[path = "../pulse_api_stub.rs"]
#[allow(dead_code)]
mod pulse_api_stub;

fn main() {
    println!("{}", pulse_api_stub::PLUGIN_API_VERSION);
}
