use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

#[path = "src/cli/mod.rs"]
mod cli;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/cli/mod.rs");
    if let Some(outdir) = std::env::var_os("OUT_DIR") {
        let outdir = std::path::PathBuf::from(outdir);
        let man_dir = outdir.join("man");
        std::fs::create_dir_all(&man_dir)?;

        let mut command = cli::ArgConfig::command();

        let man = clap_mangen::Man::new(command.clone());
        let mut buffer: Vec<u8> = Default::default();
        man.render(&mut buffer)?;

        std::fs::write(man_dir.join("mirro-rs.1"), buffer)?;

        let completions_dir = outdir.join("completions");
        std::fs::create_dir_all(&completions_dir)?;

        let crate_name = env!("CARGO_PKG_NAME");

        generate_to(Shell::Zsh, &mut command, crate_name, &completions_dir)?;
        generate_to(Shell::Bash, &mut command, crate_name, &completions_dir)?;
        generate_to(Shell::Fish, &mut command, crate_name, &completions_dir)?;
        generate_to(Shell::Elvish, &mut command, crate_name, &completions_dir)?;
    }

    Ok(())
}
