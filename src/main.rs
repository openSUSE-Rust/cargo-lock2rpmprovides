use serde_derive::Deserialize;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(parse(from_os_str))]
    _dummy: PathBuf,
    #[structopt(parse(from_os_str))]
    /// The directory containing the Cargo.toml.
    workdir: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    /// The path to the associated vendor directory.
    vendordir: Option<PathBuf>,
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

#[derive(Debug, Deserialize)]
struct CargoPkg {
    license: Option<String>,
    #[serde(rename = "license-file")]
    license_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Cargotoml {
    package: CargoPkg,
}

fn do_license_check(vendordir: &Path, debug: bool) -> Option<String> {
    let name = vendordir.join("Cargo.toml");
    // https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields
    if debug {
        eprintln!("checking license in ... {:?}", name);
    }
    if !name.exists() {
        eprintln!(
            "Unable to check license from {:?}. You may need to check this manually",
            name
        );
        return None;
    }

    let buffer = std::fs::read(&name).expect("Unable to open cargo.toml for reading!");

    let config: Cargotoml =
        toml::from_slice(&buffer).expect("Unable to parse cargo.toml, invalid!");

    if debug {
        eprintln!("Parsed config - {:?}", config);
    }

    match (config.package.license, config.package.license_file) {
        (Some(lic), _) => {
            // We have to do a bit of normalisation here.
            // If it contains an operator, we need braces.
            let mut lic = lic.replace(" / ", " OR ")
                .replace("/", " OR ");

            if lic.contains("OR") || lic.contains("AND") {
                lic.insert_str(0, "( ");
                lic.push_str(" )");
            }

            // Some common replacements to avoid duplication.
            match lic.as_str() {
                "( MIT OR Apache-2.0 )" => Some("( Apache-2.0 OR MIT )".to_string()),
                _ => Some(lic),
            }


        }
        (None, Some(fname)) => {
            let license_file = vendordir.join(fname);
            eprintln!(
                "Unable to find license in {:?}. You may need to check {:?} for details.",
                name, license_file
            );
            None
        }
        (None, None) => {
            eprintln!(
                "Unable to determine license for {:?}. You must manually investigate!",
                name
            );
            None
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    let path = opt
        .workdir
        .unwrap_or_else(|| env::current_dir().expect("Unable to locate current work dir"));

    let vendordir = opt
        .vendordir
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| path.join("vendor"));

    if opt.debug {
        eprintln!("DEBUG -> working dir {:?}", path);
        eprintln!("DEBUG -> vendor dir {:?}", vendordir);
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

    // Now check the licenses if possible.
    let debug = opt.debug;
    let mut licenses: Vec<String> = if vendordir.exists() {
        if debug {
            eprintln!("DEBUG -> found {:?}", vendordir);
        }
        config
            .package
            .iter()
            .filter_map(|pkg| {
                let pkg_vendored_path = vendordir.join(&pkg.name);
                do_license_check(&pkg_vendored_path, debug)
            })
            .collect()
    } else {
        eprintln!("ERROR could not find vendordir - {:?}", vendordir);
        Vec::new()
    };

    licenses.sort();
    licenses.dedup();

    // Now output the values.
    if opt.debug {
        for pkg in &config.package {
            eprintln!("DEBUG -> pkg -> {:?}", pkg);
        }
    }

    for pkg in &config.package {
        // There are some versions that can be problematic due to multiple hyphens.
        // This tries to account for that ...
        // 

        let mut version = pkg.version.clone();

        let mut hyphens: Vec<_> = pkg.version
            .char_indices()
            .rev()
            .filter_map(|(i, c)| if c == '-' { Some(i) } else { None })
            .collect();

        if opt.debug {
            eprintln!("hypens -> {:?}", hyphens);
        }

        // Remove the last one (should be the first hypen, if present)
        hyphens.pop();

        for i in hyphens.iter() {
            version.replace_range(*i..(i+1), "_");
        }

        println!("Provides: bundled(crate({})) = {}", pkg.name, version);
    }

    let mut license = String::new();
    for lic in licenses.iter() {
        license.push_str(&lic);
        license.push_str(" AND ");
    }
    println!("License: {}", license);

    if opt.debug {
        eprintln!("DEBUG -> Success! 🎉");
    }
}
