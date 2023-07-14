use rand::{self, Rng};
pub struct AllJANumberGen;
impl Iterator for AllJANumberGen {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let allja_nr = [
            (101..=114).map(|x| x.to_string()).collect::<Vec<_>>(),
            (2..=48)
                .map(|x| {
                    if x < 10 {
                        format!("0{}", x)
                    } else {
                        x.to_string()
                    }
                })
                .collect::<Vec<_>>(),
        ]
        .concat();
        let mut rng = rand::thread_rng();
        let s = "5NN ".to_owned() + &allja_nr[rng.gen_range(0..allja_nr.len())];
        Some(match rand::random::<u8>() {
            0..=99 => s + "H",
            100..=199 => s + "M",
            200..=255 => s + "P",
        })
    }
}

#[test]
fn test_allja_number() {
    let mut gen = AllJANumberGen;
    for _ in 0..100 {
        println!("{}", gen.next().unwrap());
    }
}
