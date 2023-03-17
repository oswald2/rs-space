
use rs_space_sle::common::*;

use rasn::types::*;
use rasn::ber::de::Error;

use rs_space_sle::asn1::*;

fn main() {
    let bind_enc: Vec<u8> = vec![
        191, 100, 106, 128, 0, 26, 8, 83, 76, 69, 95, 85, 83, 69, 82, 26, 5, 53, 53, 53, 50, 57, 2,
        1, 0, 2, 1, 2, 48, 79, 49, 14, 48, 12, 6, 7, 43, 112, 4, 3, 1, 2, 52, 26, 1, 49, 49, 25,
        48, 23, 6, 7, 43, 112, 4, 3, 1, 2, 53, 26, 12, 86, 83, 84, 45, 80, 65, 83, 83, 48, 48, 48,
        49, 49, 14, 48, 12, 6, 7, 43, 112, 4, 3, 1, 2, 38, 26, 1, 49, 49, 18, 48, 16, 6, 7, 43,
        112, 4, 3, 1, 2, 22, 26, 5, 111, 110, 108, 116, 49,
    ];

    let res: Result<SleBindInvocation, Error> = rasn::der::decode(&bind_enc[..]);

    println!("Result: {:?}", res);
}
