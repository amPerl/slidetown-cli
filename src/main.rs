use std::time::Instant;

use clap::Clap;

mod agt;
mod lbf;
mod levelmodifier;
mod lf;
mod lof;
mod loi;
mod world;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "amPerl")]
struct Opts {
    #[clap(subcommand, about = "archive type")]
    archive: Archive,
}

#[derive(Clap)]
enum Archive {
    #[clap(about = "AGT archives")]
    Agt(agt::AgtOpts),
    #[clap(about = "LF terrain blocks")]
    Lf(lf::LfOpts),
    #[clap(about = "LF terrain block objects")]
    Lbf(lbf::LbfOpts),
    #[clap(about = "LOF model table")]
    Lof(lof::LofOpts),
    #[clap(about = "LOI object list")]
    Loi(loi::LoiOpts),
    #[clap(about = "World/city")]
    World(world::WorldOpts),
    #[clap(name = "levelmodifier", about = "LevelModifier variables")]
    LevelModifier(levelmodifier::LevelModifierOpts),
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let before_process = Instant::now();

    let result = match opts.archive {
        Archive::Agt(agt_opts) => agt::process_agt(agt_opts),
        Archive::Lf(lf_opts) => lf::process_lf(lf_opts),
        Archive::Lbf(lbf_opts) => lbf::process_lbf(lbf_opts),
        Archive::Lof(lof_opts) => lof::process_lof(lof_opts),
        Archive::Loi(loi_opts) => loi::process_loi(loi_opts),
        Archive::World(world_opts) => world::process_world(world_opts),
        Archive::LevelModifier(levelmodifier_opts) => {
            levelmodifier::process_levelmodifier(levelmodifier_opts)
        }
    };

    println!("Done in {}ms", before_process.elapsed().as_millis());
    result
}
