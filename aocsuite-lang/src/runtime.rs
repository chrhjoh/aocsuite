use std::path::{Path, PathBuf};

use aocsuite_utils::atomic_write;
use serde::{Deserialize, Serialize};

use crate::{AocLanguageError, AocLanguageResult};

const RUNTIME_VERSION: u32 = 1;
const MANIFEST_NAME: &str = ".aocsuite-runtime.json";

#[derive(Deserialize, Serialize)]
struct RuntimeManifest {
    infrastructure_version: u32,
}

pub fn migrate_runtime(root: &Path, files: Vec<(PathBuf, String)>) -> AocLanguageResult<()> {
    std::fs::create_dir_all(root)?;
    let manifest_path = root.join(MANIFEST_NAME);
    let version = match std::fs::read(&manifest_path) {
        Ok(contents) => {
            serde_json::from_slice::<RuntimeManifest>(&contents)?.infrastructure_version
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error.into()),
    };
    if version > RUNTIME_VERSION {
        return Err(AocLanguageError::RuntimeMigration(format!(
            "runtime version {version} is newer than supported version {RUNTIME_VERSION}"
        )));
    }

    for (path, contents) in files {
        if version < RUNTIME_VERSION || !path.exists() {
            let parent = path.parent().expect("runtime file path has a parent");
            std::fs::create_dir_all(parent)?;
            atomic_write(&path, contents.as_bytes())?;
        }
    }

    if version < RUNTIME_VERSION {
        let manifest = serde_json::to_vec(&RuntimeManifest {
            infrastructure_version: RUNTIME_VERSION,
        })?;
        atomic_write(&manifest_path, &manifest)?;
    }
    Ok(())
}
