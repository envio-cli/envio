use regex::Regex;
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PassphraseCodegen {
    versions: Vec<VersionInfo>,
}

impl PassphraseCodegen {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            versions: discover_versions()?,
        })
    }

    pub fn write_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.versions.is_empty() {
            return Ok(());
        }

        let out_dir = std::env::var("OUT_DIR")?;

        fs::write(
            Path::new(&out_dir).join("passphrase_metadata_generated.rs"),
            self.generate_metadata_code(),
        )?;

        fs::write(
            Path::new(&out_dir).join("passphrase_decrypt_match_generated.rs"),
            self.generate_decrypt_match_macro(),
        )?;

        fs::write(
            Path::new(&out_dir).join("passphrase_encrypt_generated.rs"),
            self.generate_encrypt_function(),
        )?;

        println!("cargo:rerun-if-changed=src/cipher/passphrase");
        for v in &self.versions {
            println!(
                "cargo:rerun-if-changed=src/cipher/passphrase/{}.rs",
                v.module_name
            );
        }

        Ok(())
    }

    pub fn generate_metadata_code(&self) -> String {
        let imports = self
            .versions
            .iter()
            .map(|v| format!("use super::{}::{};", v.module_name, v.metadata_struct_type))
            .collect::<Vec<_>>()
            .join("\n");

        let variants = self
            .versions
            .iter()
            .map(|v| {
                format!(
                    "    #[serde(rename = \"{}\")] {}({}),",
                    v.version_string, v.version_ident, v.metadata_struct_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let default_variant = &self.versions.first().unwrap().version_ident;

        let from_impls = self
            .versions
            .iter()
            .map(|v| {
                format!(
                    "impl From<{}> for VersionedMetadata {{
    fn from(meta: {}) -> Self {{
        VersionedMetadata::{}(meta)
    }}
}}",
                    v.metadata_struct_type, v.metadata_struct_type, v.version_ident
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        format!(
            "{imports}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = \"version\")]
pub enum VersionedMetadata {{
{variants}
}}

impl Default for VersionedMetadata {{
    fn default() -> Self {{
        VersionedMetadata::{default_variant}(Default::default())
    }}
}}

{from_impls}
"
        )
    }

    pub fn generate_decrypt_match_macro(&self) -> String {
        let arms = self
            .versions
            .iter()
            .map(|v| {
                format!(
                    "            VersionedMetadata::{}(metadata) => \
crate::cipher::passphrase::{}::decrypt(&$self.key, metadata, $encrypted_data),",
                    v.version_ident, v.module_name
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "macro_rules! decrypt_match {{
    ($self:ident, $encrypted_data:expr) => {{
        match &$self.metadata {{
{arms}
        }}
    }};
}}"
        )
    }

    pub fn generate_encrypt_function(&self) -> String {
        if self.versions.is_empty() {
            return String::new();
        }

        let latest = self.versions.last().unwrap();

        format!(
"pub fn encrypt_latest(key: &str, data: &[u8]) -> crate::error::Result<(Vec<u8>, VersionedMetadata)> {{
    use {}::encrypt;
    let (encrypted, metadata) = encrypt(key, data)?;
    Ok((encrypted, metadata.into()))
}}",
            latest.module_name
        )
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub module_name: String,
    pub version_ident: String,
    pub metadata_struct_type: String,
    pub version_string: String,
}

pub fn discover_versions() -> Result<Vec<VersionInfo>, Box<dyn std::error::Error>> {
    let dir = Path::new("src/cipher/passphrase");

    if !dir.exists() {
        return Err("can not find passphrase directory".into());
    }

    let mut versions = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter_map(|path| parse_path_into_version(path).ok())
        .collect::<Vec<_>>();

    versions.sort_by(|a, b| {
        let va = parse_version(&a.version_string);
        let vb = parse_version(&b.version_string);
        va.cmp(&vb)
    });

    Ok(versions)
}

fn parse_path_into_version(path: PathBuf) -> Result<VersionInfo, Box<dyn std::error::Error>> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("invalid filename")?;

    if !name.starts_with('v') || !name.ends_with(".rs") || name == "mod.rs" {
        return Err("not a version file".into());
    }

    let version_num = &name[1..name.len() - 3];
    parse_version_file(&path, version_num)
}

fn parse_version_file(
    path: &Path,
    version_num: &str,
) -> Result<VersionInfo, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    let version_ident = format!("V{}", version_num.to_uppercase());
    let module_name = format!("v{}", version_num);
    let version_string = version_num.replace('_', ".");
    let metadata_struct_type = extract_metadata_type(&content, &version_ident)?;

    Ok(VersionInfo {
        module_name,
        version_ident,
        metadata_struct_type,
        version_string,
    })
}

fn extract_metadata_type(
    content: &str,
    version_ident: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let re = Regex::new(&format!(r"metadata_struct!\(\s*{}\s*,", version_ident))?;

    if re.is_match(content) {
        Ok(format!("Metadata{}", version_ident))
    } else {
        Err(format!("Could not find metadata_struct! for {}", version_ident).into())
    }
}

fn parse_version(version_str: &str) -> Version {
    Version::parse(version_str).unwrap_or_else(|_| {
        version_str
            .parse::<u64>()
            .map(|n| Version::new(n, 0, 0))
            .unwrap_or(Version::new(0, 0, 0))
    })
}
