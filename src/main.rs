mod frame;

use frame::*;

fn main() {
    let map: BitMapping = {
        let mut tmp = HashMap::new();
        tmp.insert(0, 400);
        tmp.insert(1, 600);
        tmp.insert(2, 800);
        tmp.insert(3, 1000);
        tmp
    };

    let mut fb = FrameBuilder::new(
        100, 200,
        Reference { frequency: 300, amplitude: 1.0 },
        map
    );

    println!("{:#?}", fb);
    println!("{:#?}", fb.make_frame(&[false, true, false, true]));
    println!("{:#?}", fb);
}
