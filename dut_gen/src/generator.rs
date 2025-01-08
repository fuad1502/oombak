use std::path::{Path, PathBuf};

use crate::parser;

pub fn generate(_sv_path: &Path, _probe: &parser::Probe) -> Result<PathBuf, String> {
    Ok(PathBuf::new())
}
