use settings::Settings;

fn main() {
    let s = Settings::new().unwrap();
    println!("{s:#?}");
}
