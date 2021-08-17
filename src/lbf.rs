use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};

use clap::Clap;
use slidetown::parsers::lbf;

#[derive(Clap)]
pub struct LbfOpts {
    #[clap(subcommand, about = "subcommand to run")]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    #[clap(about = "display info about archive contents")]
    Info(InfoOpts),

    #[clap(about = "export preview obj with terrain blocks")]
    Obj(ObjOpts),

    #[clap(about = "export preview gltf with terrain blocks")]
    Gltf(GltfOpts),
}

#[derive(Clap)]
struct InfoOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(&info_opts.input_path)?;
    let header: lbf::Header = lbf::Header::parse(&mut file)?;

    println!("Block count: {}", header.block_count);

    Ok(())
}

#[derive(Clap)]
struct ObjOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_obj(obj_opts: ObjOpts) -> anyhow::Result<()> {
    let mut file = File::open(&obj_opts.input_path)?;
    let lbf: lbf::Lbf = lbf::Lbf::parse(&mut file)?;

    let mut obj = nif::obj::Obj::default();

    for block in lbf.blocks {
        for block_object in block.objects {
            file.seek(SeekFrom::Start(block_object.file_offset as u64))?;

            let mut nif_buf = vec![0u8; block_object.file_length as usize];
            file.read_exact(&mut nif_buf)?;

            let mut nif_cursor = Cursor::new(nif_buf);

            let nif = match nif::Nif::parse(&mut nif_cursor) {
                Ok(nif) => nif,
                Err(e) => {
                    println!(
                        "Failed to parse NIF for block index {} unk {}: {:?}",
                        block_object.index, block_object.unk, e
                    );
                    continue;
                }
            };

            obj.visit_nif(
                &nif,
                Some(format!(
                    "Block{}Object{}",
                    block_object.index, block_object.unk
                )),
            );
        }
    }

    let obj_path = std::path::PathBuf::from(obj_opts.output_path);
    let mtl_path = obj_path.with_extension("mtl");

    obj.write_to_files(obj_path, mtl_path)?;

    Ok(())
}

#[derive(Clap)]
struct GltfOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_gltf(gltf_opts: GltfOpts) -> anyhow::Result<()> {
    let mut file = File::open(&gltf_opts.input_path)?;
    let lbf: lbf::Lbf = lbf::Lbf::parse(&mut file)?;

    let mut gltf = nif::gltf::Gltf::new();

    for block in lbf.blocks {
        for block_object in block.objects {
            file.seek(SeekFrom::Start(block_object.file_offset as u64))?;

            let mut nif_buf = vec![0u8; block_object.file_length as usize];
            file.read_exact(&mut nif_buf)?;

            let mut nif_cursor = Cursor::new(nif_buf);

            let nif = match nif::Nif::parse(&mut nif_cursor) {
                Ok(nif) => nif,
                Err(e) => {
                    println!(
                        "Failed to parse NIF for block index {} unk {}: {:?}",
                        block_object.index, block_object.unk, e
                    );
                    continue;
                }
            };

            gltf.visit_nif(
                &nif,
                Some("Block Objects"),
                &format!("Block{}Object{}", block_object.index, block_object.unk),
            );
        }
    }

    let gltf_path = std::path::PathBuf::from(gltf_opts.output_path);
    gltf.write_to_files(gltf_path)?;

    Ok(())
}

pub fn process_lbf(lbf_opts: LbfOpts) -> anyhow::Result<()> {
    match lbf_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Obj(obj_opts) => process_obj(obj_opts),
        Command::Gltf(gltf_opts) => process_gltf(gltf_opts),
    }
}
