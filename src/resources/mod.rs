//! Read-only loading of validated BSNbn, skfs and snf binary formats.
//! Planned lookup: ROZM_DATA_DIR, otherwise data/ beside the executable.

use std::path::PathBuf;

#[derive(Debug)]
pub struct ResourceLocation {
    pub root: PathBuf,
}
