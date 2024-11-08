extern crate vergen;
extern crate vergen_git2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = vergen::BuildBuilder::default()
        .build_timestamp(true)
        .build()?;
    let cargo = vergen::CargoBuilder::default()
        .opt_level(true)
        .target_triple(true)
        .build()?;
    let rustc = vergen::RustcBuilder::default()
        .semver(true)
        .host_triple(true)
        .build()?;
    let si = vergen::SysinfoBuilder::default()
        .cpu_core_count(true)
        .build()?;
    let git = vergen_git2::Git2Builder::default()
        .sha(true)
        .commit_timestamp(true)
        .build()?;
    vergen::Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .add_instructions(&git)?
        .emit()?;

    Ok(())
}
