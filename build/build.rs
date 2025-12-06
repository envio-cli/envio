#[cfg(feature = "application")]
mod application;
mod passphrase_codegen;

use passphrase_codegen::PassphraseCodegen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    PassphraseCodegen::new()?.write_all()?;

    #[cfg(feature = "application")]
    {
        application::gen_man_and_comp()?;
        application::export_build_env_vars();
    }

    Ok(())
}
