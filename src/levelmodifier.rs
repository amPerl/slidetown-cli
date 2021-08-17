use std::{fs::File, io::Write, path::Path};

use clap::Clap;
use slidetown::parsers::levelmodifier;

#[derive(Clap)]
pub struct LevelModifierOpts {
    #[clap(subcommand, about = "subcommand to run")]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    // #[clap(about = "display info about archive contents")]
    // Info(InfoOpts),
    #[clap(about = "unpack levelmodifier and create manifest")]
    Unpack(UnpackOpts),
    #[clap(about = "pack levelmodifier using manifest")]
    Pack(PackOpts),
}

// #[derive(Clap)]
// struct InfoOpts {
//     #[clap(short, long, about = "input file")]
//     input_path: String,
// }

// fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
//     let mut file = File::open(&info_opts.input_path)?;
//     let header: lf::Header = lf::Header::parse(&mut file)?;

//     println!("Dimensions: {}x{}", header.size_x, header.size_y);
//     println!("Block count: {}", header.block_count);

//     Ok(())
// }

#[derive(Clap)]
struct UnpackOpts {
    #[clap(short, long, about = "input file")]
    input_path: String,
    #[clap(short, long, about = "output file")]
    output_path: String,
}

fn process_unpack(unpack_opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = File::open(&unpack_opts.input_path).expect("Failed to open source file");

    let levelmodifier: levelmodifier::LevelModifier =
        levelmodifier::LevelModifier::parse(&mut file).expect("Failed to parse source file");

    let out_path = Path::new(&unpack_opts.output_path);

    {
        let json_file = File::create(out_path).expect("Failed to open target file");
        serde_json::to_writer_pretty(json_file, &levelmodifier)
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

    let levelmodifier: levelmodifier::LevelModifier = {
        let levelmodifier_json = File::open(input_path)?;
        serde_json::from_reader(levelmodifier_json)?
    };

    let mut out_file = File::create(pack_opts.output_path)?;
    out_file.write_all(b"DPDB")?;
    out_file.write_all(&levelmodifier.header.version_date.to_le_bytes())?;
    out_file.write_all(&levelmodifier.header.speed_length.to_le_bytes())?;
    out_file.write_all(&levelmodifier.header.accel_length.to_le_bytes())?;
    out_file.write_all(&levelmodifier.header.dura_length.to_le_bytes())?;
    out_file.write_all(&levelmodifier.header.boost_length.to_le_bytes())?;

    for id in levelmodifier.header.speed_ids {
        out_file.write_all(&id.to_le_bytes())?;
    }

    for id in levelmodifier.header.accel_ids {
        out_file.write_all(&id.to_le_bytes())?;
    }

    for id in levelmodifier.header.dura_ids {
        out_file.write_all(&id.to_le_bytes())?;
    }

    for id in levelmodifier.header.boost_ids {
        out_file.write_all(&id.to_le_bytes())?;
    }

    for option in levelmodifier.speed {
        for value in option.values {
            out_file.write_all(&value.to_le_bytes())?;
        }
    }

    for option in levelmodifier.accel {
        for value in option.values {
            out_file.write_all(&value.to_le_bytes())?;
        }
    }

    for option in levelmodifier.dura {
        for value in option.values {
            out_file.write_all(&value.to_le_bytes())?;
        }
    }

    for option in levelmodifier.boost {
        for value in option.values {
            out_file.write_all(&value.to_le_bytes())?;
        }
    }

    Ok(())
}

pub fn process_levelmodifier(levelmodifier_opts: LevelModifierOpts) -> anyhow::Result<()> {
    match levelmodifier_opts.cmd {
        // Command::Info(info_opts) => process_info(info_opts),
        Command::Unpack(unpack_opts) => process_unpack(unpack_opts),
        Command::Pack(pack_opts) => process_pack(pack_opts),
    }
}
