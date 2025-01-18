use std::env;

use media::MediaSettings;

fn main() {
    let args: Vec<String> = env::args().collect();
    let name = &args[1];

    let media = MediaSettings::new(name).unwrap();
    println!("{media:#?}");
}
