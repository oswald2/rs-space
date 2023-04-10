use rasn::ber::de::Error;

use rs_space_sle::asn1_raf::*;

fn main() {
    let bind_enc: Vec<u8> = vec![
        191, 100, 106, 128, 0, 26, 8, 83, 76, 69, 95, 85, 83, 69, 82, 26, 5, 53, 53, 53, 50, 57, 2,
        1, 0, 2, 1, 2, 48, 79, 49, 14, 48, 12, 6, 7, 43, 112, 4, 3, 1, 2, 52, 26, 1, 49, 49, 25,
        48, 23, 6, 7, 43, 112, 4, 3, 1, 2, 53, 26, 12, 86, 83, 84, 45, 80, 65, 83, 83, 48, 48, 48,
        49, 49, 14, 48, 12, 6, 7, 43, 112, 4, 3, 1, 2, 38, 26, 1, 49, 49, 18, 48, 16, 6, 7, 43,
        112, 4, 3, 1, 2, 22, 26, 5, 111, 110, 108, 116, 49,
    ];

    let bind_ret: Vec<u8> = vec![
        191, 101, 19, 128, 0, 26, 12, 83, 76, 69, 95, 80, 82, 79, 86, 73, 68, 69, 82, 128, 1, 2,
    ];

    let bind_neg_ret: Vec<u8> = vec![
        191, 101, 19, 128, 0, 26, 12, 83, 76, 69, 95, 80, 82, 79, 86, 73, 68, 69, 82, 129, 1, 2,
    ];

    let unbind: Vec<u8> = vec![191, 102, 5, 128, 0, 2, 1, 0];
    let unbind_ret: Vec<u8> = vec![191, 103, 4, 128, 0, 128, 0];

    let res: Result<SlePdu, Error> = rasn::der::decode(&bind_enc[..]);
    println!("Bind Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&bind_ret[..]);
    println!("Bind Return Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&bind_neg_ret[..]);
    println!("Bind Negative Return Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&unbind[..]);
    println!("Unbind Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&unbind_ret[..]);
    println!("Unbind Return Result: {:?}", res);
}
