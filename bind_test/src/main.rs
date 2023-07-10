use bytes::Bytes;
use rasn::{ber::de::Error, types::Utf8String, types::VisibleString};
use rs_space_core::pus_types::HexBytes;
use rs_space_sle::{
    asn1::*,
    sle::config::HashToUse,
    types::{
        aul::{HashInput, ISP1Credentials},
        sle::{
            new_service_instance_attribute, null_ccsds_time, service_instance_identifier_to_string,
            string_to_service_instance_id, ConditionalTime, Credentials, Time, RAF,
            RSL_FG, SAGR, SPACK,
        },
    },
};

fn bind_enc_test() {
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

    let res: Result<SlePdu, Error> = rasn::der::decode(&bind_enc);
    println!("Bind Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&bind_ret);
    println!("Bind Return Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&bind_neg_ret);
    println!("Bind Negative Return Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&unbind);
    println!("Unbind Result: {:?}", res);

    let res: Result<SlePdu, Error> = rasn::der::decode(&unbind_ret);
    println!("Unbind Return Result: {:?}", res);

    let sii_attr = vec![
        new_service_instance_attribute(&SAGR, "3"),
        new_service_instance_attribute(&SPACK, "facility-PASS1"),
        new_service_instance_attribute(&RSL_FG, "1"),
        new_service_instance_attribute(&RAF, "onlc1"),
    ];

    let formatted_sii = service_instance_identifier_to_string(&sii_attr);

    println!("SII: {:?}\nFormatted: {:?}", sii_attr, formatted_sii);

    //let sii = "sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1";
    let sii1 = "sagr=3";
    let parsed_sii1 = string_to_service_instance_id(sii1);
    println!("\nSII Parsing: {:?}", parsed_sii1);

    //let sii = "sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1";
    let sii2 = "sagr=3.spack=facility-PASS1";
    let parsed_sii2 = string_to_service_instance_id(sii2);
    println!("\nSII Parsing: {:?}", parsed_sii2);

    let sii3 = "sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1";
    let parsed_sii3 = string_to_service_instance_id(sii3);
    println!("\nSII Parsing: {:?}", parsed_sii3);
}

fn isp1_test() {
    let hi_comp = vec![
        0x30, 0x19, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x0a,
        0x1a, 0x04, 0x54, 0x65, 0x73, 0x74, 0x04, 0x04, 0x00, 0x01, 0x02, 0x03,
    ];

    let hi = HashInput::new(
        &null_ccsds_time(),
        10,
        &VisibleString::new(Utf8String::from("Test")),
        Bytes::copy_from_slice(&[0x00, 0x01, 0x02, 0x03]),
    );

    let enc_hi = rasn::der::encode(&hi);

    println!(
        "Encoded HashInput: {:?}\nShall be:             {:?}",
        enc_hi, hi_comp
    );

    let test_prot_sha1 = HexBytes::from_str("6c06ff67a1092d5074a3399c16c0293633a31bf8");
    let prot = hi.the_protected(HashToUse::SHA1);

    let test_prot_sha256 =
        HexBytes::from_str("3e3b6a42a56b9c9d4c23252392ba3985880940e7de963413015063713c3335a8");
    let prot2 = hi.the_protected(HashToUse::SHA256);

    println!(
        "test: {:?}\nres:  {:?}",
        test_prot_sha1,
        HexBytes(Vec::from(prot))
    );
    println!(
        "test2: {:?}\nres2:  {:?}",
        test_prot_sha256,
        HexBytes(Vec::from(prot2.clone()))
    );

    let isp1_test = HexBytes::from_str("302f0408000000000000000002010a04203e3b6a42a56b9c9d4c23252392ba3985880940e7de963413015063713c3335a8");

    let isp1 = ISP1Credentials {
        time: null_ccsds_time(),
        random: 10,
        the_protected: prot2,
    };

    let enc_isp = rasn::der::encode(&isp1).unwrap();

    println!(
        "ISP1 test: {:?}\nISP1 res:  {:?}",
        isp1_test,
        HexBytes(enc_isp.clone())
    );

    let test_creds = HexBytes::from_str("8131302f0408000000000000000002010a04203e3b6a42a56b9c9d4c23252392ba3985880940e7de963413015063713c3335a8");

    let empty_creds = Credentials::Unused;
    let enc_empty_creds = rasn::der::encode(&empty_creds);

    println!("empty_creds: {:?}", HexBytes(enc_empty_creds.unwrap()));

    let creds = Credentials::Used(isp1);
    let enc_creds = rasn::der::encode(&creds);

    println!(
        "Credentials test: {:?}\nCredentials res:  {:?}",
        test_creds,
        HexBytes(enc_creds.unwrap())
    );
}

fn time_test() {
    let t1 = ConditionalTime::HasTime(Time::CcsdsFormat(null_ccsds_time()));
    let enc_t1 = rasn::der::encode(&t1).unwrap();

    let t2: ConditionalTime = ConditionalTime::NoTime;
    let enc_t2 = rasn::der::encode(&t2).unwrap();

    println!("conditional time: {:?}", HexBytes(enc_t1));
    println!("empty conditional time: {:?}", HexBytes(enc_t2));
}

fn main() {
    bind_enc_test();
    isp1_test();

    time_test();
}
