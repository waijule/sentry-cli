use clap::{App, Arg, ArgMatches, AppSettings};

use std::path::PathBuf;

use prelude::*;
use config::Config;
use gradle::get_android_version_from_manifest;
use utils::ArgExt;


pub fn make_app<'a, 'b: 'a>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("uploads react-native projects from within a gradle build step")
        .setting(AppSettings::Hidden)
        .org_project_args()
        .arg(Arg::with_name("build_path")
             .index(1)
             .required(true)
             .value_name("PATH")
             .help("The path to the build folder that is the basis of the \
                    Android build process.  This is the folder that contains \
                    the 'intermediates' and other folders."))
        .arg(Arg::with_name("build_type")
             .long("build-type")
             .value_name("TYPE")
             .help("The build type that should be used. This defaults to \
                    'release' but can be overridden."))
        .arg(Arg::with_name("product_flavor")
             .long("product-flavor")
             .help("The product flavor that is built.  This defaults to \
                    'full'."))
}

pub fn execute<'a>(matches: &ArgMatches<'a>, _config: &Config) -> Result<()> {
    let base = PathBuf::from(matches.value_of("build_path").unwrap());
    let build_type = matches.value_of("build_type").unwrap_or("release");
    let product_flavor = matches.value_of("product_flavor").unwrap_or("full");

    let manifest_path = base
        .join("intermediates")
        .join("manifests")
        .join(product_flavor)
        .join(build_type)
        .join("AndroidManifest.xml");

    let (_version_code, _version_name) = get_android_version_from_manifest(
        &manifest_path)?;

    panic!("This command is not fully implemented yet.")
}
