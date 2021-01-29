use serde_derive::Deserialize;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct Pkg {
    name: String,
    version: String,
    dependencies: Option<Vec<String>>,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    package: Vec<Pkg>,
}

fn main() {
    let opt = Opt::from_args();

    let path = opt
        .input
        .unwrap_or_else(|| env::current_dir().expect("Unable to locate current work dir"));

    if opt.debug {
        eprintln!("DEBUG -> working dir {:?}", path);
    }

    // Do we have the Cargo.lock?
    let lockfile = path.join("Cargo.lock");
    if !lockfile.exists() {
        eprintln!("lockfile {:?} not found", lockfile);
        std::process::exit(1);
    } else if opt.debug {
        eprintln!("DEBUG -> found {:?}", lockfile);
    }

    // Can we parse it?
    let buffer = std::fs::read(lockfile).expect("Unable to open lockfile for reading!");

    let config: Config =
        toml::from_slice(&buffer).expect("Unable to parse lockfile, invalid toml!");

    // Now output the values.
    if opt.debug {
        for pkg in &config.package {
            eprintln!("DEBUG -> pkg -> {:?}", pkg);
        }
    }

    for pkg in &config.package {
        println!("Provides: bundled(crate({})) = {}", pkg.name, pkg.version);
    }

    if opt.debug {
        eprintln!("DEBUG -> Success! 🎉");
    }
}
