use dougu_essentials::get_build_info;

fn main() -> () {
    println!("version: {}", get_build_info().semantic_version().as_str());
}
