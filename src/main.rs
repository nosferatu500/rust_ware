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

fn mutate<T>(data: &Vec<u8>, mut offset: &usize) -> T {
    let new_data = &data[*offset..];
    unsafe { std::mem::transmute_copy(&new_data) }
}

fn dump_model_file(data: Vec<u8>, mut offset: usize) {
    let header: &BSSectionHeader = mutate::<&BSSectionHeader>(&data, &offset);
    offset += std::mem::size_of::<BSSectionHeader>();

    println!(
        "ID =  {:x?} (IsClump = {:?})",
        header.id as u64,
        header.id == RW::SidClump as u32
    );
    println!("Size = {:?}", header.size as u64);
    println!("Version ID = {:x?}", header.versionid as u64);

    offset += std::mem::size_of::<BSSectionHeader>();

    let clump: &BSClump = mutate::<&BSClump>(&data, &offset);
    offset += std::mem::size_of::<BSClump>();

    println!(" Clump Data");
    println!("  Atomics = {:?}", clump.numatomics as u64);

    let frame_header: &BSSectionHeader = mutate::<&BSSectionHeader>(&data, &offset);
    offset += std::mem::size_of::<BSSectionHeader>();

    println!(
        "ID =  {:x?} (IsFrameList = {:?})",
        frame_header.id as u64,
        frame_header.id == RW::SidFrameList as u32
    );

    offset += std::mem::size_of::<BSSectionHeader>();

    let frames: &BSFrameList = mutate::<&BSFrameList>(&data, &offset);
    offset += std::mem::size_of::<BSFrameList>();

    println!(" Frame List Data");
    println!("  Frames = {:?}", frames.numframes);

    for _ in 0..frames.numframes {
        let frame: &BSFrameListFrame = mutate::<&BSFrameListFrame>(&data, &offset);
        offset += std::mem::size_of::<BSFrameListFrame>();
        println!(" Frame Data");
        println!("  Index = {:?}", frame.index);
        println!(
            "  Position = {:?} {:?} {:?}",
            frame.postiion.x, frame.postiion.y, frame.postiion.z
        );
        println!("  Rotation");
        println!(
            "   {:?} {:?} {:?}",
            frame.rotation.a.x, frame.rotation.a.y, frame.rotation.a.z
        );
        println!(
            "   {:?} {:?} {:?}",
            frame.rotation.b.x, frame.rotation.b.y, frame.rotation.b.z
        );
        println!(
            "   {:?} {:?} {:?}",
            frame.rotation.c.x, frame.rotation.c.y, frame.rotation.c.z
        );
    }

    let mut next_header: &BSSectionHeader = mutate::<&BSSectionHeader>(&data, &offset);
    offset += std::mem::size_of::<BSSectionHeader>();

    while next_header.id == RW::SidExtension as u32 {
        for _ in 0..2 {
            let mut first_header: &BSSectionHeader = mutate::<&BSSectionHeader>(&data, &offset);
            offset += std::mem::size_of::<BSSectionHeader>();

            if first_header.id == RW::SidNodeName as u32 {
                println!(" Name = {:?}", first_header.size); // TODO: implement name from chars.
            } else if first_header.id == RW::SidHAnimPlg as u32 {
                println!(" Bone Information Present");
            }

            offset += first_header.size as usize;
        }
        next_header = mutate::<&BSSectionHeader>(&data, &offset);
        offset += std::mem::size_of::<BSSectionHeader>();
    }

    offset += std::mem::size_of::<BSSectionHeader>(); // Structure Header..

    let geom_list: &BSGeometryList = mutate::<&BSGeometryList>(&data, &offset);
    offset += std::mem::size_of::<BSGeometryList>();

    println!("  Geometry List Data");
    println!("   Geometries = {:?}", geom_list.numgeometry);

    for _ in 0..geom_list.numgeometry {
        let geom_header: &BSSectionHeader = mutate::<&BSSectionHeader>(&data, &offset);
        offset += std::mem::size_of::<BSSectionHeader>();

        let base_data: usize = offset;
        offset += std::mem::size_of::<BSSectionHeader>();

        let geom: &BSGeometry = mutate::<&BSGeometry>(&data, &offset);

        println!("  Geometry Data");
        println!("  Flags = {:x?}", geom.flags);
        println!("  UV Sets = {:?}", geom.numuvs);
        println!("  Flags = {:x?}", geom.geomflags);
        println!("  Triangles = {:?}", geom.numtris);
        println!("  Verticies = {:?}", geom.numverts);
        println!("  Frames = {:?}", geom.numframes);

        if geom_header.versionid < 0x1003FFFF {
            println!("  Some extra colour info");
            let colors: &BSGeometryColor = mutate::<&BSGeometryColor>(&data, &offset);
            offset += std::mem::size_of::<&BSGeometryColor>();
        }

        if (geom.flags & En::VertexColors as u16) != 0 {
            println!("  Vertex Colours Present");

            for v in 0..geom.numverts {
                println!("  {:?}: {:?}", v, mutate::<&BSColor>(&data, &offset));
                offset += std::mem::size_of::<&BSColor>();
            }
        }

        if ((geom.flags & En::TexCoords1 as u16) != 0)
            || ((geom.flags & En::TexCoords2 as u16) != 0)
        {
            println!("  UV Coords Present");

            for v in 0..geom.numverts {
                let coords: &BSGeometryUV = mutate::<&BSGeometryUV>(&data, &offset);
                offset += std::mem::size_of::<&BSGeometryUV>();

                println!("  {:?}: U{:?} V{:?}", v, coords.u, coords.v);
            }
        }

        for _ in 0..geom.numtris {
            let tri: &BSGeometryTriangle = mutate::<&BSGeometryTriangle>(&data, &offset);
            offset += std::mem::size_of::<&BSGeometryTriangle>();

            println!(
                "  Triangle {:?} {:?} {:?} A: {:?}",
                tri.first as u64, tri.second as u64, tri.third as u64, tri.attrib as u64
            );
        }

        let bounds: &BSGeometryBounds = mutate::<&BSGeometryBounds>(&data, &offset);
        offset += std::mem::size_of::<&BSGeometryBounds>();

        println!("  Bounding Radius = {:?}", bounds.radius);

        for _ in 0..geom.numverts {
            let p: &BSTVector3 = mutate::<&BSTVector3>(&data, &offset);
            offset += std::mem::size_of::<&BSTVector3>();

            println!("  v {:?} {:?} {:?}", p.x, p.y, p.z);
        }

        if (geom.flags & En::StoreNormals as u16) != 0 {
            println!("  Vertex Normals present");

            for _ in 0..geom.numverts {
                let p: &BSTVector3 = mutate::<&BSTVector3>(&data, &offset);
                offset += std::mem::size_of::<&BSTVector3>();

                println!("  n {:?} {:?} {:?}", p.x, p.y, p.z);
            }
        }

        // Jump to the start of the next geometry
        offset = base_data + geom_header.size as usize;
    }
}

fn main() {
    let path = args().nth(1).unwrap();

    let data = read_data(path).unwrap();

    let offset = 0;

    dump_model_file(data, offset);
}

enum RW {
    SidStruct = 0x0001,
    SidString = 0x0002,
    SidExtension = 0x0003,

    SidTexture = 0x0006,
    SidMaterial = 0x0007,
    SidMaterialList = 0x0008,

    SidFrameList = 0x000E,
    SidGeometry = 0x000F,
    SidClump = 0x0010,

    SidGeometryList = 0x001A,

    SidHAnimPlg = 0x011E,

    SidNodeName = 0x0253F2FE,
}

/**
 * Vector data
*/
struct BSTVector3 {
    x: f32,
    y: f32,
    z: f32,
}

/**
 * Rotation Matrix
*/
struct BSTMatrix {
    a: BSTVector3,
    b: BSTVector3,
    c: BSTVector3,
}

#[derive(Copy, Clone, Debug)]
struct BSSectionHeader {
    id: u32,
    size: u32,
    versionid: u32,
}

struct BSExtension {}

struct BSFrameList {
    numframes: u32,
}

struct BSFrameListFrame {
    rotation: BSTMatrix,
    postiion: BSTVector3,
    index: u32,
    matrixflags: u32, // UNUSED BY ANYTHING.
}

#[derive(Copy, Clone, Debug)]
struct BSClump {
    numatomics: u32,
}

struct BSStruct {
    id: u32, // = 0x0001
}

struct BSGeometryList {
    numgeometry: u32,
}

enum En {
    IsTriangleStrip = 0x1,
    VertexTranslation = 0x2,
    TexCoords1 = 0x4,
    VertexColors = 0x8,
    StoreNormals = 0x16,
    DynamicVertexLighting = 0x32,
    ModuleMaterialColor = 0x64,
    TexCoords2 = 0x128,
}

struct BSGeometry {
    flags: u16,
    numuvs: u8,
    geomflags: u8,
    numtris: u32,
    numverts: u32,
    numframes: u32,
}

type BSColor = u32;

struct BSGeometryColor {
    ambient: BSColor,
    diffuse: BSColor,
    specular: BSColor,
}

struct BSGeometryUV {
    u: f32,
    v: f32,
}

struct BSGeometryTriangle {
    first: u16,
    second: u16,
    attrib: u16, // Who designed this nonsense.
    third: u16,
}

struct BSGeometryBounds {
    center: BSTVector3,
    radius: f32,
    positions: u32,
    normals: u32,
}
