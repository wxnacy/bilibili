use bili_video::Spliter;

fn main() {
    let s = Spliter::new("examples/data/trailer.mp4");
    s.set_parts(5).with_quick(false).output("/Users/wxnacy/test.mp4");
}
