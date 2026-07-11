use std::path::Path;

use terminalos_shared::WorkspaceId;
use uuid::Uuid;

/// Derives a stable workspace id from a project path.
#[must_use]
pub fn id_from_path(path: impl AsRef<Path>) -> WorkspaceId {
    let canonical = path
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf());
    let name = format!("terminalos:workspace:{}", canonical.display());
    WorkspaceId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_for_same_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = id_from_path(dir.path());
        let b = id_from_path(dir.path());
        assert_eq!(a, b);
    }
}
