use std::ops::RangeInclusive;

use rand::{self, Rng};
pub struct ACAGNumberGen {
    acag_nr: Vec<String>,
}

impl ACAGNumberGen {
    pub fn new() -> Self {
        let acag_nr = [
            //北海道
            filter_and_to_string_numbers(
                vec![101..=136, 1001..=1081, 10101..=10110],
                &[
                    132, 1004, 1012, 1017, 1029, 1032, 1044, 1054, 1056, 1057, 1058, 1061, 1065,
                    1068,
                ],
                |x| format!("0{}", x),
            ),
            //青森
            filter_and_to_string_numbers(vec![201..=210, 2001..=2008], &[], |x| format!("0{}", x)),
            //岩手
            filter_and_to_string_numbers(
                vec![301..=316, 3001..=3013],
                &[306, 312, 3003, 3011, 3012],
                |x| format!("0{}", x),
            ),
            //秋田
            filter_and_to_string_numbers(
                vec![401..=415, 4001..=4009],
                &[405, 408, 4003, 4006, 4009],
                |x| format!("0{}", x),
            ),
            //山形
            filter_and_to_string_numbers(vec![501..=513, 5001..=5011], &[5004, 5009, 5010], |x| {
                format!("0{}", x)
            }),
            //宮城
            filter_and_to_string_numbers(
                vec![602..=616, 6001..=6016, 60101..=60105],
                &[604, 610, 6005, 6007, 6009, 6011, 6012, 6015],
                |x| format!("0{}", x),
            ),
            //福島
            filter_and_to_string_numbers(
                vec![701..=720, 7001..=7017],
                &[704, 706, 709, 710, 712, 713, 716, 7001, 7004, 7008, 7009],
                |x| format!("0{}", x),
            ),
            //新潟
            filter_and_to_string_numbers(
                vec![802..=828, 8001..=8016, 80101..=80108],
                &[
                    803, 807, 814, 815, 817, 819, 820, 821, 8003, 8005, 8006, 8009, 8010, 8012,
                    8014,
                ],
                |x| format!("0{}", x),
            ),
            //長野
            filter_and_to_string_numbers(
                vec![901..=921, 9001..=9017],
                &[916, 917, 9007, 9016],
                |x| format!("0{}", x),
            ),
            //東京
            filter_and_to_string_numbers(
                vec![100101..=100123, 1002..=1030, 10002..=10007],
                &[1017, 1018, 1027, 10003],
                |x| x.to_string(),
            ),
            //神奈川
            filter_and_to_string_numbers(
                vec![
                    1102..=1119,
                    11001..=11007,
                    110101..=110118,
                    110301..=110307,
                    111001..=111003,
                ],
                &[1103, 1110, 11005],
                |x| x.to_string(),
            ),
            //千葉
            filter_and_to_string_numbers(
                vec![1202..=1239, 12001..=12011, 120101..=120106],
                &[1209, 1214, 12003, 12005, 12007, 12009, 12010, 12012],
                |x| x.to_string(),
            ),
            //埼玉
            filter_and_to_string_numbers(
                vec![1302..=1346, 13001..=13009, 134401..=134410],
                &[1305, 1313, 1320, 1326, 1335, 1344, 13005],
                |x| x.to_string(),
            ),
            //茨城
            filter_and_to_string_numbers(
                vec![1401..=1437, 14001..=14014],
                &[
                    1406, 1409, 1411, 1413, 1418, 14002, 14006, 14007, 14009, 14010, 14011, 14013,
                ],
                |x| x.to_string(),
            ),
            //栃木
            filter_and_to_string_numbers(vec![1501..=1516, 15004..=15008], &[1507, 1512], |x| {
                x.to_string()
            }),
            //群馬
            filter_and_to_string_numbers(
                vec![1601..=1612, 16001..=16010],
                &[16002, 16006, 16008],
                |x| x.to_string(),
            ),
            //群馬
            filter_and_to_string_numbers(
                vec![1601..=1612, 16001..=16010],
                &[16002, 16006, 16008],
                |x| x.to_string(),
            ),
            //山梨
            filter_and_to_string_numbers(
                vec![1701..=1714, 17002..=17008],
                &[1703, 17005, 17006],
                |x| x.to_string(),
            ),
            //静岡
            filter_and_to_string_numbers(
                vec![1803..=1827, 18006..=18011, 180101..=180103, 180201..=180207],
                &[1804, 1810, 1818, 1819, 18007],
                |x| x.to_string(),
            ),
            //岐阜
            filter_and_to_string_numbers(
                vec![1901..=1921, 19001..=19018],
                &[
                    19002, 19004, 19006, 19009, 19010, 19013, 19014, 19016, 19018,
                ],
                |x| x.to_string(),
            ),
            //岐阜
            filter_and_to_string_numbers(
                vec![1901..=1921, 19001..=19018],
                &[
                    19002, 19004, 19006, 19009, 19010, 19013, 19014, 19016, 19018,
                ],
                |x| x.to_string(),
            ),
            //愛知
            filter_and_to_string_numbers(
                vec![2002..=2040, 20001..=20010, 200101..=200116],
                &[2018, 2020, 20002, 20006, 20008],
                |x| x.to_string(),
            ),
            //三重
            filter_and_to_string_numbers(
                vec![2101..=2117, 21006..=21016],
                &[2106, 2113, 2114, 21007, 21010, 21011, 21013],
                |x| x.to_string(),
            ),
            //京都
            filter_and_to_string_numbers(
                vec![2202..=2215, 22003..=22014, 220101..=220111],
                &[22004, 22005, 22007, 22009, 22011, 22013],
                |x| x.to_string(),
            ),
            //滋賀
            filter_and_to_string_numbers(vec![2301..=2314, 23002..=23004], &[2305], |x| {
                x.to_string()
            }),
            //奈良
            filter_and_to_string_numbers(
                vec![2401..=2412, 24001..=24010],
                &[24003, 24006, 24008],
                |x| x.to_string(),
            ),
            //大阪
            filter_and_to_string_numbers(
                vec![2503..=2536, 25002..=25007, 250101..=250127, 250201..=250207],
                &[2505, 2519, 2520, 25005, 250105, 250110, 250112],
                |x| x.to_string(),
            ),
            //和歌山
            filter_and_to_string_numbers(vec![2601..=2609, 26001..=26007], &[26004], |x| {
                x.to_string()
            }),
            //兵庫
            filter_and_to_string_numbers(
                vec![2702..=2731, 27001..=27020, 270101..=270109],
                &[
                    2712, 2722, 27002, 27003, 27004, 27006, 27008, 27009, 27012, 27014, 27015,
                    27017, 27018, 27019,
                ],
                |x| x.to_string(),
            ),
            //富山
            filter_and_to_string_numbers(vec![2801..=2811, 28003..=28004], &[2803], |x| {
                x.to_string()
            }),
            //福井
            filter_and_to_string_numbers(
                vec![2901..=2910, 29002..=29012],
                &[2903, 29004, 29005, 29006, 29007],
                |x| x.to_string(),
            ),
            //石川
            filter_and_to_string_numbers(
                vec![3001..=3012, 30003..=30009],
                &[3008, 30005, 30008],
                |x| x.to_string(),
            ),
            //岡山
            filter_and_to_string_numbers(
                vec![3102..=3118, 31001..=31020, 310101..=310104],
                &[
                    3105, 3106, 3108, 31002, 31004, 31005, 31008, 31009, 31011, 31012, 31013,
                    31014, 31018,
                ],
                |x| x.to_string(),
            ),
            //島根
            filter_and_to_string_numbers(
                vec![3201..=3209, 32003..=32012],
                &[3208, 32005, 32007, 32009, 32010, 32011],
                |x| x.to_string(),
            ),
            //山口
            filter_and_to_string_numbers(
                vec![3301..=3316, 33002..=33006],
                &[3305, 3309, 3314, 33004],
                |x| x.to_string(),
            ),
            //鳥取
            filter_and_to_string_numbers(vec![3401..=3404, 34001..=34006], &[34002], |x| {
                x.to_string()
            }),
            //広島
            filter_and_to_string_numbers(
                vec![
                    3502..=3516,
                    35001..=35001,
                    35007..=35008,
                    35010..=35010,
                    35016..=35016,
                    350101..=350108,
                ],
                &[3506, 3507],
                |x| x.to_string(),
            ),
            //香川
            filter_and_to_string_numbers(vec![3601..=3608, 36001..=36006], &[36002], |x| {
                x.to_string()
            }),
            //徳島
            filter_and_to_string_numbers(vec![3701..=3708, 37002..=37010], &[37003], |x| {
                x.to_string()
            }),
            //愛媛
            filter_and_to_string_numbers(
                vec![3801..=3815, 38001..=38012],
                &[3808, 3809, 3811, 3812, 38002, 38004, 38008, 38009, 38011],
                |x| x.to_string(),
            ),
            //高知
            filter_and_to_string_numbers(vec![3901..=3912, 39001..=39007], &[3906, 39003], |x| {
                x.to_string()
            }),
            //福岡
            filter_and_to_string_numbers(
                vec![4007..=4037, 40001..=40018, 400101..=400107, 402101..=402107],
                &[
                    4013, 4014, 4021, 4028, 40002, 40003, 40008, 40010, 40013, 40016, 40017,
                ],
                |x| x.to_string(),
            ),
            //佐賀
            filter_and_to_string_numbers(vec![4101..=4110, 41002..=41008], &[41004], |x| {
                x.to_string()
            }),
            //長崎
            filter_and_to_string_numbers(
                vec![4201..=4214, 42004..=42009],
                &[4206, 42005, 42008],
                |x| x.to_string(),
            ),
            //熊本
            filter_and_to_string_numbers(
                vec![4302..=4316, 43001..=43012, 430101..=430105],
                &[4307, 4309, 43004, 43006, 43011],
                |x| x.to_string(),
            ),
            //大分
            filter_and_to_string_numbers(
                vec![4401..=4415, 44005..=44005, 44009..=44010],
                &[4409],
                |x| x.to_string(),
            ),
            //宮崎
            filter_and_to_string_numbers(vec![4501..=4509, 45001..=45006], &[], |x| x.to_string()),
            //鹿児島
            filter_and_to_string_numbers(
                vec![4601..=4626, 46001..=46011],
                &[
                    4602, 4605, 4608, 4609, 4611, 4612, 4613, 46002, 46004, 46007, 46012,
                ],
                |x| x.to_string(),
            ),
            //沖縄
            filter_and_to_string_numbers(
                vec![4701..=4715, 47001..=47005],
                &[4702, 4703, 4705, 4707],
                |x| x.to_string(),
            ),
        ]
        .concat();
        Self { acag_nr }
    }
}

impl Default for ACAGNumberGen {
    fn default() -> Self {
        Self::new()
    }
}

fn filter_and_to_string_numbers(
    nrs: Vec<RangeInclusive<i32>>,
    filter_nrs: &[i32],
    to_string: impl Fn(i32) -> String,
) -> Vec<String> {
    nrs.into_iter()
        .flat_map(|nrs| nrs.filter(|x| !filter_nrs.contains(x)).map(&to_string))
        .collect()
}

impl Iterator for ACAGNumberGen {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.acag_nr.len());
        let s = self.acag_nr[idx].clone();

        Some(match rand::random::<u8>() {
            0..=99 => s + "H",
            100..=199 => s + "M",
            200..=224 => s + "L",
            225..=255 => s + "P",
        })
    }
}
