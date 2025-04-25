use dougu_essentials::runtime::get_build_info;

fn main() -> () {
    println!("version: {}", get_build_info().semantic_version().as_str());
}
