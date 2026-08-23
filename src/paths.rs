use std::path::{Path, PathBuf};

// resuelve una ruta relativa probando, en orden: CWD actual, directorio del
// ejecutable, y hasta 3 niveles hacia arriba desde ahi (cubre target/release/).
// si ninguna existe, devuelve la ruta original para que el warning de carga siga siendo claro.
pub fn resolve(path: &str) -> String {
    if Path::new(path).exists() {
        return path.to_string();
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(mut dir) = exe.parent().map(Path::to_path_buf) {
            for _ in 0..4 {
                let candidate = dir.join(path);
                if candidate.exists() {
                    return candidate.to_string_lossy().into_owned();
                }
                match dir.parent().map(PathBuf::from) {
                    Some(parent) => dir = parent,
                    None => break,
                }
            }
        }
    }

    path.to_string()
}
