use bili_video::Remover;

fn main() {
    let r = Remover::new("examples/data/trailer.mp4", vec![(40, 60)]);
    let to = format!("~/Downloads/1-{}.mp4", lazytool::current_timestamp());
    r.output(lazytool::expand_user(to)).unwrap();

    let r = Remover::new("examples/data/trailer.mp4", vec![(0, 20), (40, 60)]);
    let to = format!("~/Downloads/2-{}.mp4", lazytool::current_timestamp());
    r.output(lazytool::expand_user(to)).unwrap();

    let r = Remover::new("examples/data/trailer.mp4", vec![(10, 20), (40, 45)]);
    let to = format!("~/Downloads/2-{}.mp4", lazytool::current_timestamp());
    r.output(lazytool::expand_user(to)).unwrap();
}
