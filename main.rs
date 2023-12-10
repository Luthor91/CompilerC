use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Spécifie le chemin du répertoire racine à partir duquel le parcours commence
    let root_path = ".";

    // Mesure le temps d'exécution
    let start_time = SystemTime::now();

    // Collecte les fichiers avec les extensions spécifiées
    let c_files = match explore_directory(root_path, "c") {
        Ok(files) => files,
        Err(err) => {
            eprintln!("Erreur lors de la collecte des fichiers C : {}", err);
            return;
        }
    };

    let h_files = match explore_directory(root_path, "h") {
        Ok(files) => files,
        Err(err) => {
            eprintln!("Erreur lors de la collecte des fichiers H : {}", err);
            return;
        }
    };

    let dll_files = match explore_directory(root_path, "dll") {
        Ok(files) => files,
        Err(err) => {
            eprintln!("Erreur lors de la collecte des fichiers DLL : {}", err);
            return;
        }
    };

    let a_files = match explore_directory(root_path, "a") {
        Ok(files) => files,
        Err(err) => {
            eprintln!("Erreur lors de la collecte des fichiers A : {}", err);
            return;
        }
    };

    // Affiche les listes
    println!("Fichiers C : {:?}", c_files);
    println!("Fichiers H : {:?}", h_files);
    println!("Fichiers DLL : {:?}", dll_files);
    println!("Fichiers A : {:?}", a_files);

    // Liste pour stocker les lignes contenant "#include"
    let unique_lines: HashSet<String> = {
        let unique_lines_mutex = Arc::new(Mutex::new(HashSet::new()));

        // Parallélisation de la lecture des fichiers ".c"
        let handles: Vec<_> = c_files.clone().into_iter().map(|file_path| {
            let unique_lines_mutex = Arc::clone(&unique_lines_mutex);

            thread::spawn(move || {
                if let Ok(file) = File::open(&file_path) {
                    let reader = io::BufReader::new(file);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if line.contains("#include ") {
                                // Trouve les positions des guillemets
                                let start_quote = line.find('"');
                                let end_quote = line.rfind('"');

                                // Si les guillemets sont présents, stocke le texte entre eux
                                if let (Some(start), Some(end)) = (start_quote, end_quote) {
                                    let include_text = line[start + 1..end].to_string();
                                    let mut unique_lines = unique_lines_mutex.lock().unwrap();
                                    unique_lines.insert(include_text);
                                } else {
                                    // Sinon, recherche les symboles '<' et '>'
                                    let start_bracket = line.find('<');
                                    let end_bracket = line.rfind('>');

                                    // Si les symboles sont présents, stocke le texte entre eux
                                    if let (Some(start), Some(end)) = (start_bracket, end_bracket) {
                                        let include_text = line[start + 1..end].to_string();
                                        let mut unique_lines = unique_lines_mutex.lock().unwrap();
                                        unique_lines.insert(include_text);
                                    }
                                }
                            }
                        }
                    }
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        Arc::try_unwrap(unique_lines_mutex).unwrap().into_inner().unwrap()
    };

    // Convertit l'ensemble en un vecteur pour obtenir la liste sans doublons
    let unique_lines: Vec<String> = unique_lines.into_iter().collect();

    // Divise unique_lines en quatre listes en fonction de l'extension
    let (expected_files_for_c, expected_files_for_h, expected_files_for_dll, expected_files_for_a) =
        split_files_by_extension(&unique_lines.clone());

    // Vérifie si les fichiers inclus sont présents dans les listes c_files, h_files, dll_files et a_files
    let _ = log_warning(
        root_path,
        "C",
        &c_files,
        &expected_files_for_c,
    );
    let _ = log_warning(
        root_path,
        "H",
        &h_files,
        &expected_files_for_h,
    );
    let _ = log_warning(
        root_path,
        "DLL",
        &dll_files,
        &expected_files_for_dll,
    );
    let _ = log_warning(
        root_path, 
        "A", 
        &a_files, 
        &expected_files_for_a
    );
    

    // Affiche le temps d'exécution et le nombre total de fichiers traités
    if let Ok(elapsed_time) = start_time.elapsed() {
        let elapsed_secs = elapsed_time.as_secs();
        let elapsed_millis = elapsed_time.subsec_millis();

        let total_files = c_files.len() + h_files.len() + dll_files.len() + a_files.len() + unique_lines.len();
        println!("Temps d'exécution : {}.{:03} secondes", elapsed_secs, elapsed_millis);
        println!("Nombre de fichiers traités : {}", total_files);
    }
}

fn log_warning(
    root_path: &str,
    file_type: &str,
    file_list: &Vec<PathBuf>,
    expected_files: &Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convertit la liste de fichiers en HashSet pour une recherche plus rapide
    let file_set: HashSet<_> = file_list.iter().collect();

    // Liste des fichiers manquants pour cette extension
    let mut missing_files: Vec<&PathBuf> = Vec::new();

    // Vérifie si chaque fichier inclus est présent dans la liste
    for include_file in expected_files {
        if !file_set.contains(include_file) {
            missing_files.push(include_file);
        }
    }

    // Construit le message d'avertissement
    if !missing_files.is_empty() {
        let missing_files_str: Vec<_> = missing_files
            .iter()
            .map(|path_buf| path_buf.to_string_lossy().to_string())
            .collect();

        let warning_message = format!(
            "Avertissement: Fichier(s) {} non trouvé(s): {}",
            file_type,
            missing_files_str.join(", ")
        );

        // Récupère la date actuelle
        let current_time = SystemTime::now();
        let formatted_time = time_to_string(current_time);

        // Construit le message de log complet
        let log_message = format!(
            "Projet {}  -  {}\n{}\nFichiers attendus : {:?}\nFichiers dans unique_lines : {:?}\n\n",
            root_path,
            formatted_time,
            file_type,
            warning_message,
            expected_files
        );

        // Crée le dossier "logs" s'il n'existe pas
        if let Err(err) = fs::create_dir_all("logs") {
            eprintln!("Erreur lors de la création du dossier 'logs': {}", err);
            return Err(err.into());
        }

        // Crée le chemin du fichier de log avec la date du jour
       // let log_path = format!("logs/{}.log", formatted_time);
       let log_path = format!("logs/{}.log", formatted_time);
        let mut file = match OpenOptions::new().create(true).append(true).open(&log_path) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Erreur lors de l'ouverture ou de la création du fichier de log : {}", err);
                return Err(err.into());
            }
        };

        // Écrit le message de log dans le fichier
        if let Err(err) = writeln!(file, "{}", log_message) {
            eprintln!("Erreur lors de l'écriture dans le fichier de log : {}", err);
            return Err(err.into());
        }        
    }

    Ok(())
}

fn explore_directory(root_path: &str, extension: &str) -> Result<Vec<PathBuf>, io::Error> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_file() && entry_path.extension().is_some() {
                    let file_extension = entry_path.extension().unwrap().to_string_lossy().to_lowercase();
                    if file_extension == extension {
                        result.push(entry_path.clone());
                    }
                } else if entry_path.is_dir() {
                    result.extend(explore_directory(&entry_path.to_string_lossy(), extension)?);
                }
            }
        }
    }
    Ok(result)
}

fn time_to_string(time: SystemTime) -> String {
    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        // Calcul des jours, heures, minutes et secondes
        let days = duration.as_secs() / (24 * 3600);

        // Formatage de la date
        let formatted_time = format!(
            "{:02}_{:02}_{:04}",
            1 + (duration.as_secs() as i64) / 86400,
            1 + (duration.as_secs() as i64) / 2592000,
            1970 + days
        );

        // Remplace les caractères interdits par des tirets
        let sanitized_time = formatted_time
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();

        sanitized_time
    } else {
        "unknown_time".to_string()
    }
}





fn split_files_by_extension(unique_lines: &Vec<String>) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut c_files = Vec::new();
    let mut h_files = Vec::new();
    let mut dll_files = Vec::new();
    let mut a_files = Vec::new();

    for file in unique_lines {
        let file_extension = Path::new(&file)
            .extension()
            .map(|ext| ext.to_string_lossy())
            .unwrap_or_default();

        match file_extension.to_lowercase().as_str() {
            "c" => c_files.push(PathBuf::from(file)),
            "h" => h_files.push(PathBuf::from(file)),
            "dll" => dll_files.push(PathBuf::from(file)),
            "a" => a_files.push(PathBuf::from(file)),
            _ => (),
        }
    }

    (c_files, h_files, dll_files, a_files)
}
