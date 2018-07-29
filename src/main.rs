use std::env::args;
use std::fs::File;
use std::io::*;
use std::path::Path;

fn read_data<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = try!(File::open(&path));

    let mut data = Vec::new();

    try!(file.read_to_end(&mut data));

    Ok(data)
}

fn main() {
    let path = args().nth(1).unwrap();

    let data = read_data(path).unwrap();

    let header: &BSSectionHeader = unsafe { std::mem::transmute_copy(&data) };

    println!("lenght =  {:?}", data.len());
    println!("data =  {:x?}", &data as *const Vec<u8>);
    println!(
        "ID =  {:x?} (IsClump = {:?})",
        header.id as u64,
        header.id == RW::SidClump as u32
    );
    println!("Size = {:?}", header.size as u64);
    println!("Version ID = {:x?}", header.versionid as u64);

    if header.id == RW::SidClump as u32 {
        let a = &data[12..];
        let header: &BSClump = unsafe { std::mem::transmute_copy(&a) };

        println!(" Clump Data");
        println!("  Atomics = {:?}", header.numatomics as u64);
        println!("  Lights = {:?}", header.numlights as u64);
        println!("  Cameras = {:?}", header.numcameras as u64);
    }
}

enum RW {
    SidStruct = 0x0001,
    SidString = 0x0002,
    SidExtension = 0x0003,

    SidClump = 0x0010,
}

#[derive(Copy, Clone, Debug)]
struct BSSectionHeader {
    id: u32,
    size: u32,
    versionid: u32,
}

#[derive(Copy, Clone, Debug)]
struct BSClump {
    numatomics: u32,
    numlights: u32,
    numcameras: u32,
}
