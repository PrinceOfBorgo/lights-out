use std::{
    env::{self, VarError},
    path::{Path, PathBuf},
};
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=settings.toml");
    println!("cargo:warning=CWD is {:?}", env::current_dir()?);
    println!("cargo:warning=OUT_DIR is {:?}", env::var("OUT_DIR")?);
    println!(
        "cargo:warning=CARGO_MANIFEST_DIR is {:?}",
        env::var("CARGO_MANIFEST_DIR")?
    );
    println!("cargo:warning=PROFILE is {:?}", env::var("PROFILE")?);

    let output_path = get_output_path()?;
    println!(
        "cargo:warning=Calculated build path: {}",
        output_path.to_str().unwrap()
    );

    let input_path = Path::new(&env::var("CARGO_MANIFEST_DIR")?).join("settings.toml");
    let output_path = Path::new(&output_path).join("settings.toml");
    let res = std::fs::copy(input_path, output_path);
    println!("cargo:warning={:?}", res);

    #[cfg(windows)]
    {
        println!(
            "cargo:warning=Setting icon: {:?}",
            WindowsResource::new()
                // This path can be absolute, or relative to your crate root.
                .set_icon("lights_out.ico")
                .compile()
        );
    }

    Ok(())
}

fn get_output_path() -> Result<PathBuf, VarError> {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR")?;
    let build_type = env::var("PROFILE")?;
    let path = Path::new(&manifest_dir_string)
        .join("target")
        .join(build_type);
    Ok(path)
}
