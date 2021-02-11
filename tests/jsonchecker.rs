use simd_json;
use std::fs::File;
use std::io::Read;

macro_rules! pass {
    ($file:ident) => {
        #[test]
        fn $file() {
            let mut v1 = Vec::new();
            let f = String::from(concat!("data/pass/", stringify!($file), ".json"));
            File::open(f).unwrap().read_to_end(&mut v1).unwrap();
            let mut v2 = v1.clone();
            let v1 = simd_json::to_borrowed_value(&mut v1);
            dbg!(&v1);
            assert!(v1.is_ok());
            let v1 = v1.unwrap();
            let v2 = simd_json::to_owned_value(&mut v2);
            dbg!(&v2);
            assert!(v2.is_ok());
            let v2 = v2.unwrap();
            let v1o: simd_json::OwnedValue = v1.clone().into();
            assert_eq!(v2, v1o)
        }
    };
}

macro_rules! fail {
    ($file:ident) => {
        #[test]
        fn $file() {
            let mut v1 = Vec::new();
            let f = String::from(concat!("data/fail/", stringify!($file), ".json"));
            File::open(f).unwrap().read_to_end(&mut v1).unwrap();
            let mut v2 = v1.clone();
            let v1 = simd_json::to_borrowed_value(&mut v1);
            dbg!(&v1);
            assert!(v1.is_err());
            let v2 = simd_json::to_owned_value(&mut v2);
            dbg!(&v2);
            assert!(v2.is_err());
        }
    };
}

macro_rules! crash {
    ($file:ident) => {
        #[test]
        fn $file() {
            let mut v1 = Vec::new();
            let f = String::from(concat!("data/crash/", stringify!($file), ".json"));
            File::open(f).unwrap().read_to_end(&mut v1).unwrap();
            let mut v2 = v1.clone();
            let _ = simd_json::to_borrowed_value(&mut v1);
            let _ = simd_json::to_owned_value(&mut v2);
        }
    };
}

pass!(pass01);
pass!(pass02);
pass!(pass03);
pass!(pass04);
pass!(pass05);
pass!(pass06);
pass!(pass07);
pass!(pass08);
pass!(pass09);
pass!(pass10);
pass!(pass11);
pass!(pass12);
pass!(pass13);
pass!(pass14);
pass!(pass15);
pass!(pass16);

// fail!(fail01_EXCLUDED);
fail!(fail02);
fail!(fail03);
fail!(fail04);
fail!(fail05);
fail!(fail06);
fail!(fail07);
fail!(fail08);
fail!(fail09);

fail!(fail10);
fail!(fail11);
fail!(fail12);
fail!(fail13);
fail!(fail14);
fail!(fail15);
fail!(fail16);
fail!(fail17);
// fail!(fail18_EXCLUDED);
fail!(fail19);

fail!(fail20);
fail!(fail21);
fail!(fail22);
fail!(fail23);
fail!(fail24);
fail!(fail25);
fail!(fail26);
fail!(fail27);
fail!(fail28);
fail!(fail29);

fail!(fail30);
fail!(fail31);
fail!(fail32);
fail!(fail33);
fail!(fail34);
fail!(fail35);
fail!(fail36);
fail!(fail37);
fail!(fail38);
//fail!(fail39_EXCLUDED);

//fail!(fail40_s64boverflow); No longer a failure!
fail!(fail41_toolarge);
fail!(fail42);
fail!(fail43);
fail!(fail44);
fail!(fail45);
fail!(fail46);
fail!(fail47);
fail!(fail48);
fail!(fail49);

fail!(fail50);
fail!(fail51);
fail!(fail52);
fail!(fail53);
fail!(fail54);
fail!(fail55);
fail!(fail56);
fail!(fail57);
fail!(fail58);
fail!(fail59);

fail!(fail60);
fail!(fail61);
fail!(fail62);
fail!(fail63);
fail!(fail64);
fail!(fail65);
fail!(fail66);
fail!(fail67);
fail!(fail68);
// This is not a failure on 128bit parsing
#[cfg(not(feature = "128bit"))]
fail!(fail69);

fail!(fail70);
fail!(fail71);
fail!(fail72);
fail!(fail73);
fail!(fail74);
fail!(fail75);
fail!(fail76);
fail!(fail77);
fail!(fail78);
//fail!(fail79);

crash!(crash000000);
crash!(crash000001);
crash!(crash000002);
crash!(crash000003);
crash!(crash000004);
crash!(crash000005);
crash!(crash000006);
crash!(crash000007);
crash!(crash000008);
crash!(crash000009);

crash!(crash000010);
crash!(crash000011);
crash!(crash000012);
crash!(crash000013);
crash!(crash000014);
crash!(crash000015);
crash!(crash000016);
crash!(crash000017);
crash!(crash000018);
crash!(crash000019);

crash!(crash000020);
crash!(crash000021);
crash!(crash000022);
crash!(crash000023);
crash!(crash000024);
crash!(crash000025);
crash!(crash000026);
crash!(crash000027);
crash!(crash000028);
crash!(crash000029);

crash!(crash000030);
crash!(crash000031);
crash!(crash000032);
crash!(crash000033);
crash!(crash000034);
crash!(crash000035);
crash!(crash000036);
crash!(crash000037);
// crash!(crash000038);
// crash!(crash000039);
