#[cfg(feature = "application")]
mod application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "application")]
    {
        application::gen_man_and_comp()?;
        application::export_build_env_vars();
    }

    Ok(())
}
