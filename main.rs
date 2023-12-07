use std::fs;
use std::path::Path;

fn main() {
    // Spécifie le chemin du répertoire racine à partir duquel le parcours commence
    let root_path = ".";

    // Appelle la fonction récursive pour parcourir le répertoire racine
    if let Err(e) = explore_directory(Path::new(root_path)) {
        eprintln!("Erreur: {}", e);
    }
}

fn explore_directory(path: &Path) -> Result<(), std::io::Error> {
    // Récupère tous les éléments dans le répertoire actuel
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;

        // Obtient le chemin complet de l'entrée
        let entry_path = entry.path();

        if entry_path.is_file() {
            // Si l'entrée est un fichier, affiche les détails
            if let Some(extension) = entry_path.extension() {
                if let Some(file_name) = entry_path.file_name() {
                    let file_size = entry.metadata()?.len();
                    println!(
                        "Nom du fichier: {:?}, Extension: {:?}, Taille: {} octets",
                        file_name, extension, file_size
                    );
                }
            }
        } else if entry_path.is_dir() {
            // Si l'entrée est un répertoire, appelle la fonction récursive
            explore_directory(&entry_path)?;
        }
    }

    Ok(())
}
