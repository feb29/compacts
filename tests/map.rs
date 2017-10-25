extern crate compacts;
extern crate snap;
extern crate zstd;

use std::{fs, io};
use compacts::{bits, ReadFrom, WriteTo};
use self::bits::PopCount;

// https://github.com/RoaringBitmap/RoaringFormatSpec

#[test]
fn read_write_from_file() {
    let m1 = bitmapwithruns();
    let m2 = bitmapwithoutruns();

    for i in 0..100000 {
        if i % 1000 == 0 {
            assert!(m1[i] && m2[i]);
        } else {
            assert!(!m1[i] && !m2[i]);
        }
    }
    for i in 100000..200000 {
        assert!(m1[i * 3] && m2[i * 3]);
    }
    for i in 700000..800000 {
        assert!(m1[i] && m2[i]);
    }

    assert_eq!(m1.count1(), m2.count1());
    assert_eq!(m1.count0(), m2.count0());
    for (b1, b2) in m1.bits().zip(m2.bits()) {
        assert_eq!(b1, b2);
    }

    let body = {
        let mut file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        let mut body = Vec::with_capacity(8192);
        io::copy(&mut file, &mut body).unwrap();
        body
    };
    let buff = {
        let mut buff = Vec::with_capacity(8192);
        {
            m1.write_to(&mut buff).unwrap();
        }
        println!("no-compress {}", buff.len());
        buff
    };
    assert_eq!(body, buff);
}

#[test]
fn read_write_snappy() {
    use std::io::Write;

    let m1 = bitmapwithruns();
    let mut w = Vec::with_capacity(8192);
    {
        let mut buf = snap::Writer::new(&mut w);
        m1.write_to(&mut buf).unwrap();
        buf.flush().unwrap();
    }
    println!("snappy {}", w.len());

    let mut r = snap::Reader::new(io::Cursor::new(w));
    let mut m2 = bits::Map::new();
    m2.read_from(&mut r).unwrap();

    assert_eq!(m1.count1(), m2.count1());
    assert_eq!(m1.count0(), m2.count0());
    for (b1, b2) in m1.bits().zip(m2.bits()) {
        assert_eq!(b1, b2);
    }
}

#[test]
fn read_write_zstd() {
    let m1 = bitmapwithruns();
    let mut w = Vec::with_capacity(8192);
    {
        let mut enc = zstd::Encoder::new(&mut w, 0).unwrap();
        m1.write_to(&mut enc).unwrap();
        enc.finish().unwrap();
    }
    println!("zstd {}", w.len());

    let mut r = zstd::Decoder::new(io::Cursor::new(w)).unwrap();
    let mut m2 = bits::Map::new();
    m2.read_from(&mut r).unwrap();

    assert_eq!(m1.count1(), m2.count1());
    assert_eq!(m1.count0(), m2.count0());
    for (b1, b2) in m1.bits().zip(m2.bits()) {
        assert_eq!(b1, b2);
    }
}

fn must_open_bitmap(p: &str) -> bits::Map {
    let file = fs::File::open(p).unwrap();
    let mut map = bits::Map::new();
    map.read_from(&mut io::BufReader::new(file)).unwrap();
    map.optimize();
    map
}

fn bitmapwithruns() -> bits::Map {
    must_open_bitmap("./tests/bitmapwithruns.bin")
}
fn bitmapwithoutruns() -> bits::Map {
    must_open_bitmap("./tests/bitmapwithoutruns.bin")
}

// #[test]
// fn similarity() {
//     let _ = env_logger::init();
//     let mut rng = rand::thread_rng();
//
//     let size = (1 << 15) * 7;
//     let maxn = (1 << 16) * 2;
//
//     let p = &gen_bitmap!(size, maxn, rng);
//     let q = &gen_bitmap!(size, maxn, rng);
//
//     let jaccard = {
//         let r = p.intersection(q);
//         r.count_ones() as f64 / (p.count_ones() + q.count_ones() - r.count_ones()) as f64
//     };
//
//     let dice = {
//         let r = p.intersection(q);
//         (2.0 * (r.count_ones() as f64)) / (p.count_ones() + q.count_ones()) as f64
//     };
//
//     let simpson = {
//         let r = p.intersection(q);
//         (r.count_ones() as f64) / (p.count_ones() as f64).min(q.count_ones() as f64)
//     };
//
//     info!("Jaccard  = {:.5?}", jaccard);
//     info!("Dice     = {:.5?}", dice);
//     info!("Simpson  = {:.5?}", simpson);
//     info!("Distance = {:.5?}", 1f64 - jaccard);
// }
