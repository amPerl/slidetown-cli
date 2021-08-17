use std::{
    convert::TryInto,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

use clap::Clap;
use encoding_rs::EUC_KR;
use slidetown::parsers::lof;

#[derive(Clap)]
pub struct LofOpts {
    #[clap(subcommand, about = "subcommand to run")]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    #[clap(about = "display info about archive contents")]
    Info(InfoOpts),

    #[clap(about = "unpack model table nifs and create manifest")]
    Unpack(UnpackOpts),

    #[clap(about = "pack model table nifs using manifest")]
    Pack(PackOpts),

    #[clap(about = "export preview gltf with model table nifs")]
    Gltf(GltfOpts),
}

#[derive(Clap)]
struct InfoOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(&info_opts.input_path)?;
    let header: lof::Header = lof::Header::parse(&mut file)?;

    println!("Model count: {}", header.model_count);

    Ok(())
}

#[derive(Clap)]
struct UnpackOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output directory")]
    output_path: String,
}

fn process_unpack(unpack_opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = File::open(&unpack_opts.input_path)?;

    let lof_archive: lof::Lof = lof::Lof::parse(&mut file).expect("Could not parse LOF");

    let out_dir_path = Path::new(&unpack_opts.output_path);
    std::fs::create_dir_all(out_dir_path).expect("Could not create output directory");

    {
        let manifest_file = File::create(out_dir_path.join("manifest.json"))?;
        serde_json::to_writer_pretty(manifest_file, &lof_archive)?;
    }

    for lof_model in lof_archive.models {
        println!("Writing model {}", lof_model.file_name);

        let nif_position: u64 = lof_model.file_offset.into();
        let nif_length: usize = lof_model
            .file_length
            .try_into()
            .expect("Model file size too high");

        let mut nif_buffer = vec![0u8; nif_length];

        file.seek(SeekFrom::Start(nif_position))?;
        file.read_exact(&mut nif_buffer)?;

        let nif_path = out_dir_path.join(lof_model.file_name);
        let nif_dir = nif_path.with_file_name("");
        std::fs::create_dir_all(nif_dir).expect("Could not create directory for model");

        let mut nif_file = File::create(nif_path).expect("Failed to open model file for writing");
        nif_file
            .write_all(&nif_buffer)
            .expect("Failed to write to model file");
    }

    Ok(())
}

#[derive(Clap)]
struct PackOpts {
    #[clap(short, long, about = "input manifest")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_pack(pack_opts: PackOpts) -> anyhow::Result<()> {
    let input_path = Path::new(&pack_opts.input_path);

    let lof_archive: lof::Lof = {
        let manifest_file = File::open(input_path).expect("Failed to open manifest for reading");
        serde_json::from_reader(manifest_file).expect("Failed to parse manifest")
    };

    let mut out_file =
        File::create(pack_opts.output_path).expect("Failed to create lof for writing");
    out_file.write_all(b"LOF\0kjc\0")?;
    out_file.write_all(&lof_archive.header.unknown1.to_le_bytes())?;
    out_file.write_all(&lof_archive.header.version_date.to_le_bytes())?;
    out_file.write_all(&lof_archive.header.model_count.to_le_bytes())?;
    out_file.write_all(&lof_archive.header.unknown2.to_le_bytes())?;

    let mut offsets_offsets: Vec<u64> = Vec::new();

    for model in lof_archive.models.iter() {
        out_file.write_all(&model.index.to_le_bytes())?;
        out_file.write_all(&model.unknown1.to_le_bytes())?;
        out_file.write_all(&model.unknown2.to_le_bytes())?;
        out_file.write_all(&model.unknown3.to_le_bytes())?;
        out_file.write_all(&model.unknown4.to_le_bytes())?;
        out_file.write_all(&model.unknown5.to_le_bytes())?;

        let (encoded_name, _used_encoding, _had_errors) = EUC_KR.encode(&model.name);
        out_file.write_all(&encoded_name)?;
        out_file.write_all(&[0u8; 1])?;

        let (encoded_file_name, _used_encoding, _had_errors) = EUC_KR.encode(&model.file_name);
        out_file.write_all(&encoded_file_name)?;
        out_file.write_all(&[0u8; 1])?;

        out_file.write_all(&model.unknown6.to_le_bytes())?;
        out_file.write_all(&model.unknown7.to_le_bytes())?;
        out_file.write_all(&model.unknown8.to_le_bytes())?;

        // Save position to fill in offsets later
        offsets_offsets.push(out_file.seek(SeekFrom::Current(0))?);
        out_file.write_all(&0_u32.to_le_bytes())?;
        out_file.write_all(&0_u32.to_le_bytes())?;
    }

    for (model, &header_offset) in lof_archive.models.iter().zip(offsets_offsets.iter()) {
        let model_file_path = input_path.with_file_name("").join(&model.file_name);

        let file_offset = out_file.seek(SeekFrom::Current(0))? as u32;

        let mut model_file =
            File::open(model_file_path).expect("Failed to open model for writing into lof");
        let file_length = std::io::copy(&mut model_file, &mut out_file)
            .expect("Failed to write model to lof") as u32;

        // Go back and fill in header offsets
        out_file.seek(SeekFrom::Start(header_offset))?;
        out_file.write_all(&file_offset.to_le_bytes())?;
        out_file.write_all(&file_length.to_le_bytes())?;
        out_file.seek(SeekFrom::Start((file_offset + file_length).into()))?;
    }

    Ok(())
}

#[derive(Clap)]
struct GltfOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

pub fn process_gltf_inner(
    input_path: &str,
    scene_name: Option<&str>,
) -> anyhow::Result<(
    nif::gltf::Gltf,
    std::collections::HashMap<u32, nif::gltf::json::Index<nif::gltf::json::Node>>,
)> {
    let mut file = File::open(&input_path)?;
    let lof: lof::Lof = lof::Lof::parse(&mut file)?;

    let mut gltf = nif::gltf::Gltf::new();
    let mut model_indices = std::collections::HashMap::new();

    for model in lof.models {
        file.seek(SeekFrom::Start(model.file_offset as u64))?;

        let mut nif_buf = vec![0u8; model.file_length as usize];
        file.read_exact(&mut nif_buf)?;

        let mut nif_cursor = Cursor::new(nif_buf);

        let nif = match nif::Nif::parse(&mut nif_cursor) {
            Ok(nif) => nif,
            Err(e) => {
                println!(
                    "Failed to parse NIF for model index {}: {:?}",
                    model.index, e
                );
                continue;
            }
        };

        model_indices.insert(
            model.index,
            gltf.visit_nif(&nif, scene_name, &format!("Model{}", model.index)),
        );
    }

    Ok((gltf, model_indices))
}

fn process_gltf(gltf_opts: GltfOpts) -> anyhow::Result<()> {
    let (gltf, _model_indices) = process_gltf_inner(&gltf_opts.input_path, Some("Models"))?;

    let gltf_path = std::path::PathBuf::from(gltf_opts.output_path);
    gltf.write_to_files(gltf_path)?;

    Ok(())
}

pub fn process_lof(lof_opts: LofOpts) -> anyhow::Result<()> {
    match lof_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Unpack(unpack_opts) => process_unpack(unpack_opts),
        Command::Pack(pack_opts) => process_pack(pack_opts),
        Command::Gltf(gltf_opts) => process_gltf(gltf_opts),
    }
}
