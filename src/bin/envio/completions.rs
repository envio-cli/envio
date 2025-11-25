use std::env;

pub const BASH_COMPLETION: &str = include_str!(env!("ENVIO_GENERATED_COMPLETION_BASH"));
pub const FISH_COMPLETION: &str = include_str!(env!("ENVIO_GENERATED_COMPLETION_FISH"));
pub const ZSH_COMPLETION: &str = include_str!(env!("ENVIO_GENERATED_COMPLETION_ZSH"));
pub const PS1_COMPLETION: &str = include_str!(env!("ENVIO_GENERATED_COMPLETION_PS1"));
