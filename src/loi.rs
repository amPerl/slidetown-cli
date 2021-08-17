use std::{fs::File, io::Write, path::Path};

use clap::Clap;
use slidetown::parsers::loi;

#[derive(Clap)]
pub struct LoiOpts {
    #[clap(subcommand, about = "subcommand to run")]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    #[clap(about = "display info about object list")]
    Info(InfoOpts),
    #[clap(about = "unpack object list and create manifest")]
    Unpack(UnpackOpts),
    #[clap(about = "pack object list using manifest")]
    Pack(PackOpts),
    #[clap(about = "export preview gltf with instanced objects")]
    Gltf(GltfOpts),
}

#[derive(Clap)]
struct InfoOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(&info_opts.input_path)?;
    let loi: loi::Loi = loi::Loi::parse(&mut file)?;

    println!("Block count: {}", loi.header.block_count);
    println!(
        "Blocks with any objects in them {}",
        loi.blocks
            .iter()
            .filter(|block| block.object_count > 0)
            .count()
    );
    println!(
        "Object count sum over blocks {}",
        loi.blocks
            .iter()
            .map(|block| block.object_count)
            .sum::<u32>()
    );

    let object_indices = loi
        .blocks
        .iter()
        .flat_map(|block| block.objects.iter().map(|object| object.object_index))
        .collect::<Vec<u32>>();

    let object_indices_max = *object_indices.iter().max().unwrap_or(&0);

    println!("Highest object_index {}", object_indices_max);

    // println!("Blocks with objects:");
    // for block in loi.blocks.iter() {
    //     if block.object_count <= 0 {
    //         continue;
    //     }
    //     println!(
    //         "Block {} with object ids: {:?}",
    //         block.block_index,
    //         block
    //             .objects
    //             .iter()
    //             .map(|object| object.object_index)
    //             .collect::<Vec<u32>>()
    //     )
    // }

    // for potential_object_index in 0..object_indices_max {
    //     if object_indices.contains(&potential_object_index) {
    //         continue;
    //     }
    //     println!(
    //         "Missing object index in sequence: {}",
    //         potential_object_index
    //     );
    // }

    Ok(())
}

#[derive(Clap)]
struct UnpackOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_unpack(unpack_opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = File::open(&unpack_opts.input_path).expect("Failed to open source file");

    let loi_archive: loi::Loi = loi::Loi::parse(&mut file).expect("Failed to parse source file");

    let out_path = Path::new(&unpack_opts.output_path);

    {
        let json_file = File::create(out_path).expect("Failed to open target file");
        serde_json::to_writer_pretty(json_file, &loi_archive)
            .expect("Failed to write to target file");
    }

    Ok(())
}

#[derive(Clap)]
struct PackOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_pack(pack_opts: PackOpts) -> anyhow::Result<()> {
    let input_path = Path::new(&pack_opts.input_path);

    let loi: loi::Loi = {
        let manifest_file = File::open(input_path)?;
        serde_json::from_reader(manifest_file)?
    };

    let mut out_file = File::create(pack_opts.output_path)?;
    out_file.write_all(b"LOI\0kjc\0")?;
    out_file.write_all(&loi.header.unknown1.to_le_bytes())?;
    out_file.write_all(&loi.header.version_date.to_le_bytes())?;
    out_file.write_all(&loi.header.block_count.to_le_bytes())?;

    for block in loi.blocks.iter() {
        out_file.write_all(&block.block_index.to_le_bytes())?;
        out_file.write_all(&block.object_count.to_le_bytes())?;

        for block_object in block.objects.iter() {
            out_file.write_all(&block_object.unknown1.to_le_bytes())?;
            out_file.write_all(&block_object.unknown2.to_le_bytes())?;
            out_file.write_all(&block_object.unknown3.to_le_bytes())?;
            out_file.write_all(&block_object.unknown4.to_le_bytes())?;
            out_file.write_all(&block_object.object_index.to_le_bytes())?;
            out_file.write_all(&block_object.block_index.to_le_bytes())?;
            out_file.write_all(&block_object.model_table_index.to_le_bytes())?;
            out_file.write_all(&block_object.position.0.to_le_bytes())?;
            out_file.write_all(&block_object.position.1.to_le_bytes())?;
            out_file.write_all(&block_object.position.2.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.0 .0.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.0 .1.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.0 .2.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.1 .0.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.1 .1.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.1 .2.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.2 .0.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.2 .1.to_le_bytes())?;
            out_file.write_all(&block_object.rotation.2 .2.to_le_bytes())?;
            out_file.write_all(&block_object.scale.to_le_bytes())?;
            out_file.write_all(&block_object.unknown8.to_le_bytes())?;
            out_file.write_all(&block_object.unknown9.to_le_bytes())?;
            out_file.write_all(&block_object.object_extra_index.to_le_bytes())?;
            out_file.write_all(&block_object.unknown11.to_le_bytes())?;
        }
    }

    out_file.write_all(&loi.object_extra_count.to_le_bytes())?;

    for object_extra in loi.object_extras.iter() {
        out_file.write_all(&object_extra.object_index.to_le_bytes())?;
        out_file.write_all(&object_extra.object_extra_index.to_le_bytes())?;
        out_file.write_all(&object_extra.unknown3.to_le_bytes())?;
        out_file.write_all(&object_extra.position.0.to_le_bytes())?;
        out_file.write_all(&object_extra.position.1.to_le_bytes())?;
        out_file.write_all(&object_extra.position.2.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.0 .0.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.0 .1.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.0 .2.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.1 .0.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.1 .1.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.1 .2.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.2 .0.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.2 .1.to_le_bytes())?;
        out_file.write_all(&object_extra.rotation.2 .2.to_le_bytes())?;
        out_file.write_all(&object_extra.unknown4.0.to_le_bytes())?;
        out_file.write_all(&object_extra.unknown4.1.to_le_bytes())?;
        out_file.write_all(&object_extra.unknown4.2.to_le_bytes())?;
        out_file.write_all(&object_extra.unknown5.to_le_bytes())?;
    }

    for unknown_object_2 in loi.unknown_objects_2.iter() {
        out_file.write_all(&unknown_object_2.unknown_count.to_le_bytes())?;

        for unknown_object_2_item in unknown_object_2.items.iter() {
            out_file.write_all(&unknown_object_2_item.to_le_bytes())?;
        }
    }

    out_file.write_all(&loi.unknown_object_3_count.to_le_bytes())?;

    for unknown_object_3 in loi.unknown_objects_3.iter() {
        out_file.write_all(&unknown_object_3.unknown1.to_le_bytes())?;
        out_file.write_all(&unknown_object_3.unknown_count.to_le_bytes())?;

        for unknown_object_3_item in unknown_object_3.items.iter() {
            out_file.write_all(&unknown_object_3_item.to_le_bytes())?;
        }
    }

    for unknown_object_4 in loi.unknown_objects_4.iter() {
        out_file.write_all(&unknown_object_4.unknown_count.to_le_bytes())?;
        out_file.write_all(&unknown_object_4.unknown1.to_le_bytes())?;

        for unknown_object_4_item in unknown_object_4.items.iter() {
            out_file.write_all(&unknown_object_4_item.to_le_bytes())?;
        }
    }

    for unknown_object_5 in loi.unknown_objects_5.iter() {
        out_file.write_all(&unknown_object_5.object_count.to_le_bytes())?;

        for object_index in unknown_object_5.object_indices.iter() {
            out_file.write_all(&object_index.to_le_bytes())?;
        }
    }

    Ok(())
}

#[derive(Clap)]
struct GltfOpts {
    #[clap(short, long, about = "path to object0.loI")]
    loi_path: String,
    #[clap(short, long, about = "path to modeltable0.LOF")]
    lof_path: String,

    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_gltf(gltf_opts: GltfOpts) -> anyhow::Result<()> {
    let mut file = File::open(&gltf_opts.loi_path)?;
    let loi: loi::Loi = loi::Loi::parse(&mut file)?;

    let (mut gltf, model_indices) =
        crate::lof::process_gltf_inner(&gltf_opts.lof_path, None).expect("failed to process lof");

    let mut instance_indices = Vec::new();

    for block in loi.blocks {
        for block_object in block.objects {
            let &model_node_index = model_indices
                .get(&block_object.model_table_index)
                .expect("couldn't find model");
            instance_indices.push(gltf.clone_node(
                model_node_index,
                Some([
                    block_object.position.0,
                    block_object.position.1,
                    block_object.position.2,
                ]),
                Some([
                    block_object.rotation.0 .0,
                    block_object.rotation.0 .1,
                    block_object.rotation.0 .2,
                    block_object.rotation.1 .0,
                    block_object.rotation.1 .1,
                    block_object.rotation.1 .2,
                    block_object.rotation.2 .0,
                    block_object.rotation.2 .1,
                    block_object.rotation.2 .2,
                ]),
                Some(block_object.scale),
            ));
        }
    }

    gltf.get_or_create_scene("Instanced Objects", Some(instance_indices));

    let gltf_path = std::path::PathBuf::from(gltf_opts.output_path);
    gltf.write_to_files(gltf_path)?;

    Ok(())
}

pub fn process_loi(loi_opts: LoiOpts) -> anyhow::Result<()> {
    match loi_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Unpack(unpack_opts) => process_unpack(unpack_opts),
        Command::Pack(pack_opts) => process_pack(pack_opts),
        Command::Gltf(gltf_opts) => process_gltf(gltf_opts),
    }
}
