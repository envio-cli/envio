pub use git_version::git_version;

use crate::error::AppResult;
use crate::utils::{get_configdir, get_profile_dir};

struct Code {
    language: Option<String>,
    code: String,
}

enum DiagnosticEntry {
    Text(String),
    Code(Code),
    List(Vec<DiagnosticEntry>),
}

struct DiagnosticSection<'a> {
    title: &'a str,
    entry: DiagnosticEntry,
}

pub struct DiagnosticReport<'a> {
    sections: Vec<DiagnosticSection<'a>>,
}

impl<'a> DiagnosticReport<'a> {
    pub fn generate() -> AppResult<DiagnosticReport<'a>> {
        let mut sections = vec![];

        sections.push(DiagnosticSection {
            title: "Software version",
            entry: DiagnosticEntry::List(vec![
                DiagnosticEntry::Text(format!(
                    "{} {} ({})",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                    git_version!(fallback = "")
                )),
                DiagnosticEntry::Text(format!("Build timestamp: {}", env!("BUILD_TIMESTAMP"))),
            ]),
        });

        sections.push(DiagnosticSection {
            title: "Operating system",
            entry: DiagnosticEntry::List(vec![
                DiagnosticEntry::Text(format!(
                    "OS: {}",
                    sysinfo::System::long_os_version().unwrap_or_else(|| "Unknown".to_owned()),
                )),
                DiagnosticEntry::Text(format!(
                    "Kernel: {}",
                    sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".to_owned()),
                )),
            ]),
        });

        #[cfg(target_family = "unix")]
        {
            if let Ok(shell) = std::env::var("SHELL") {
                sections.push(DiagnosticSection {
                    title: "Shell",
                    entry: DiagnosticEntry::Text(shell),
                });
            }
        }

        sections.push(DiagnosticSection {
            title: "Configuration",
            entry: DiagnosticEntry::List(get_config_info()),
        });

        sections.push(DiagnosticSection {
            title: "Command line",
            entry: DiagnosticEntry::Code(Code {
                language: Some("bash".into()),
                code: std::env::args_os()
                    .map(|arg| shell_escape::escape(arg.to_string_lossy()).to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            }),
        });

        sections.push(DiagnosticSection {
            title: "GnuPG",
            entry: DiagnosticEntry::Text(get_gnupg_version()?),
        });

        #[cfg(target_family = "unix")]
        {
            sections.push(DiagnosticSection {
                title: "GPGME",
                entry: DiagnosticEntry::Text(get_gpgme_version()?),
            });
        }

        sections.push(DiagnosticSection {
            title: "Compile time information",
            entry: DiagnosticEntry::List(vec![
                DiagnosticEntry::Text(format!("Profile: {}", env!("PROFILE"))),
                DiagnosticEntry::Text(format!("Target triple: {}", env!("TARGET"))),
                DiagnosticEntry::Text(format!("Family: {}", env!("CARGO_CFG_TARGET_FAMILY"))),
                DiagnosticEntry::Text(format!("OS: {}", env!("CARGO_CFG_TARGET_OS"))),
                DiagnosticEntry::Text(format!("Architecture: {}", env!("CARGO_CFG_TARGET_ARCH"))),
                DiagnosticEntry::Text(format!(
                    "Pointer width: {}",
                    env!("CARGO_CFG_TARGET_POINTER_WIDTH")
                )),
                DiagnosticEntry::Text(format!("Endian: {}", env!("CARGO_CFG_TARGET_ENDIAN"))),
                DiagnosticEntry::Text(format!(
                    "CPU features: {}",
                    env!("CARGO_CFG_TARGET_FEATURE")
                )),
                DiagnosticEntry::Text(format!("Host: {}", env!("HOST"))),
            ]),
        });

        Ok(DiagnosticReport { sections })
    }

    pub fn print(&self) -> AppResult<()> {
        let mut output = String::new();

        for section in &self.sections {
            output += &format_section(section.title);
            output += &format_entry(&section.entry);
            output += "\n";
        }

        println!("{}", output.trim_end());
        Ok(())
    }
}

fn format_section(title: &str) -> String {
    format!("#### {}\n\n", title)
}

fn format_entry(entry: &DiagnosticEntry) -> String {
    match entry {
        DiagnosticEntry::Text(content) => format!("{}\n", content),
        DiagnosticEntry::Code(c) => format!(
            "```{}\n{}\n```\n",
            c.language.as_deref().unwrap_or(""),
            c.code
        ),
        DiagnosticEntry::List(entries) => {
            entries
                .iter()
                .map(|e| format!("- {}", format_entry(e).trim_end()))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
    }
}

fn get_config_info() -> Vec<DiagnosticEntry> {
    let mut info = vec![];

    let configdir = get_configdir();
    info.push(DiagnosticEntry::Text(format!(
        "Config directory: {} ({})",
        configdir.display(),
        if configdir.exists() {
            "exists"
        } else {
            "does not exist"
        }
    )));

    let profile_dir = get_profile_dir();
    if profile_dir.exists() {
        match std::fs::read_dir(&profile_dir) {
            Ok(entries) => {
                let profile_count = entries.count();
                info.push(DiagnosticEntry::Text(format!(
                    "Profile directory: {} ({} profiles)",
                    profile_dir.display(),
                    profile_count
                )));
            }
            Err(_) => {
                info.push(DiagnosticEntry::Text(format!(
                    "Profile directory: {} (cannot read)",
                    profile_dir.display()
                )));
            }
        }
    } else {
        info.push(DiagnosticEntry::Text(format!(
            "Profile directory: {} (does not exist)",
            profile_dir.display()
        )));
    }

    #[cfg(target_family = "unix")]
    {
        let shellscript_path = configdir.join("setenv.sh");
        info.push(DiagnosticEntry::Text(format!(
            "Shell script: {} ({})",
            shellscript_path.display(),
            if shellscript_path.exists() {
                "exists"
            } else {
                "does not exist"
            }
        )));
    }

    info
}

#[cfg(target_family = "unix")]
fn get_gpgme_version() -> AppResult<String> {
    let lib_version = gpgme::init().version();
    Ok(format!("gpgme {}", lib_version))
}

fn get_gnupg_version() -> AppResult<String> {
    let output = std::process::Command::new("gpg")
        .arg("--version")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout.to_string())
}
