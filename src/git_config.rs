use anyhow::Result;
use ini::Ini;
use std::path::PathBuf;

#[derive(Debug)]
pub struct GitConfig {
    pub repository_format_version: i32,
    pub filemode: bool,
    pub bare: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            repository_format_version: 0,
            filemode: false,
            bare: false,
        }
    }
}

impl GitConfig {
    pub fn read(path: &PathBuf) -> Result<Self> {
        let conf = Ini::load_from_file(path)?;
        let core_repository_format_version = conf
            .section(Some("core"))
            .ok_or(anyhow::anyhow!("core section not found"))?
            .get("repositoryformatversion")
            .ok_or(anyhow::anyhow!("repositoryformatversion not found"))?
            .parse::<i32>()?;
        Ok(Self {
            repository_format_version: core_repository_format_version,
            filemode: false,
            bare: false,
        })
    }

    pub fn write(&self, path: &PathBuf) -> Result<()> {
        let mut conf = Ini::new();
        conf.with_section(Some("core")).set(
            "repositoryformatversion",
            self.repository_format_version.to_string(),
        );
        conf.with_section(Some("core"))
            .set("filemode", self.filemode.to_string());
        conf.with_section(Some("core"))
            .set("bare", self.bare.to_string());
        conf.write_to_file(path)?;
        Ok(())
    }
}
