//! # Projet de Vérification des Fichiers d'un Projet Rust
//!
//! Ce programme en Rust parcourt un répertoire et vérifie la cohérence des fichiers du projet.

extern crate chrono;

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use chrono::Local;
use std::sync::{Arc, Mutex};
use std::thread;

/// Représente les types de fichiers que nous recherchons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FileType {
    C,
    H,
    DLL,
    A,
}

// Déclarer les variables globales
static mut FORMATTED_TIME: Option<String> = None;
static mut LOG_PATH: Option<String> = None;

/// Fonction principale du programme.
fn main() {
    // Spécifie le chemin du répertoire racine à partir duquel le parcours commence
    let root_path = ".";

    // Mesure le temps d'exécution
    let start_time = SystemTime::now();

    // Collecte les fichiers avec les extensions spécifiées
    let c_files = collect_files(root_path, FileType::C);
    let h_files = collect_files(root_path, FileType::H);
    let dll_files = collect_files(root_path, FileType::DLL);
    let a_files = collect_files(root_path, FileType::A);

    // Liste pour stocker les lignes contenant "#include"
    let unique_lines = process_c_files(&c_files);

    unsafe {
        // Obtient la date formatée
        FORMATTED_TIME = Some(get_date());
        // Obtient le chemin du fichier de log
        LOG_PATH = Some(format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()));
    }

    // Divise unique_lines en quatre listes en fonction de l'extension
    let (expected_files_for_c, expected_files_for_h, expected_files_for_dll, expected_files_for_a) =
        split_files_by_extension(&unique_lines);

    // Vérifie si les fichiers inclus sont présents dans les listes c_files, h_files, dll_files et a_files
    check_and_log_warnings("C", &c_files, &expected_files_for_c);
    check_and_log_warnings("H", &h_files, &expected_files_for_h);
    check_and_log_warnings("DLL", &dll_files, &expected_files_for_dll);
    check_and_log_warnings("A", &a_files, &expected_files_for_a);

    // Affiche le temps d'exécution et le nombre total de fichiers traités
    if let Ok(elapsed_time) = start_time.elapsed() {
        let elapsed_secs = elapsed_time.as_secs();
        let elapsed_millis = elapsed_time.subsec_millis();
    
        let total_files = c_files.len() + h_files.len() + dll_files.len() + a_files.len() + unique_lines.len();
        println!("Temps d'exécution : {}.{:03} secondes pour {} fichiers", elapsed_secs, elapsed_millis, total_files);

        let l_log_path: String;
        let mut log_message: String = format!("Temps d'exécution : {}.{:03} secondes", elapsed_secs, elapsed_millis);

        unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }
        write_in_logs(l_log_path.clone(), log_message);

        log_message = format!("Nombre de fichiers traités : {}\n", total_files);

        write_in_logs(l_log_path, log_message);
    }

    let include_paths = extract_unique_paths(&h_files);
    let library_paths = extract_unique_paths(&dll_files);

    let libraries = extract_unique_file_names(&dll_files);

    // Générer la commande
    let mut command = Command::new("gcc");

    // Ajouter les fichiers .c
    command.args(&["-o", "main"]).args(&c_files);

    // Ajouter les chemins d'inclusion (-I)
    for include_path in &include_paths {
        command.args(&["-I", include_path]);
    }

    // Ajouter les chemins des bibliothèques (-L)
    for library_path in &library_paths {
        command.args(&["-L", library_path]);
    }

    // Ajouter les bibliothèques à lier (-l)
    for library in &libraries {
        // Retirer le préfixe "lib" si présent pour les fichiers .dll
        let library_name = if library.ends_with(".dll") {
            &library[3..library.len() - 4]
        } else {
            library
        };
        command.args(&["-l", library_name]);
    }

    // Ajouter les autres options
    command.args(&["-lm", "-Wall"]);

    // Afficher la commande
    display_command(&command);

    // Exécuter la commande
    let output = command.output().expect("Impossible d'exécuter la commande");

    let l_log_path: String;
    let log_message : String = format!("Sortie de la commande : \n\t{}", String::from_utf8_lossy(&output.stdout));

    unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }
    write_in_logs(l_log_path, log_message);

    if !output.status.success() {

        let l_log_path: String;
        let log_message : String = format!("Erreur, la commande à échouée : \n\t{}", String::from_utf8_lossy(&output.stderr));

        unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }

        write_in_logs(l_log_path, log_message);
    }
}

fn display_command(command: &Command) {
    let command_str = format!("Commande réalisée : \n\t{:?}\n", command);
    let l_log_path:String;

    unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }

    write_in_logs(l_log_path, command_str);

}

/// Collecte les fichiers avec une extension spécifiée.
fn collect_files(root_path: &str, file_type: FileType) -> Vec<PathBuf> {
    match explore_directory(root_path, file_type) {
        Ok(files) => files,
        Err(err) => {
            eprintln!("Erreur lors de la collecte des fichiers {:?} : {}", file_type, err);
            Vec::new()
        }
    }
}

/// Parcours le contenu des fichiers ".c" en parallèle pour extraire les lignes contenant "#include ".
fn process_c_files(c_files: &[PathBuf]) -> HashSet<String> {
    let unique_lines_mutex = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];

    for file_path in c_files {
        let unique_lines_mutex = Arc::clone(&unique_lines_mutex);

        // Clonage du chemin de fichier pour que chaque thread possède sa propre copie
        let file_path = file_path.clone();

        let handle = thread::spawn(move || {
            if let Ok(file) = File::open(&file_path) {
                let reader = io::BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.contains("#include ") {
                            process_include_line(&line, &unique_lines_mutex);
                        }
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Attend que tous les threads se terminent
    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(unique_lines_mutex).unwrap().into_inner().unwrap()
}

/// Traite une ligne contenant "#include " en extrayant le texte inclus.
fn process_include_line(line: &str, unique_lines_mutex: &Arc<Mutex<HashSet<String>>>) {
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

/// Vérifie si les fichiers inclus sont présents dans la liste de fichiers et log les avertissements si nécessaire.
fn check_and_log_warnings(file_type: &str, file_list: &[PathBuf], expected_files: &[PathBuf]) {
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

        unsafe {
            // Obtient la date formatée
            FORMATTED_TIME = Some(get_date());
            // Obtient le chemin du fichier de log
            LOG_PATH = Some(format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()));
        }

        let formatted_time = unsafe { FORMATTED_TIME.as_ref().unwrap().clone() };
        let log_path = unsafe { LOG_PATH.as_ref().unwrap().clone() };
        let current_path: PathBuf = std::env::current_dir().expect("Impossible d'obtenir le répertoire actuel");

        let parent_name: Option<&str> = current_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
        let parent_name_str: String = parent_name.unwrap_or_default().to_string();
        

        // Construit le message de log complet
        let log_message = format!(
            "Project Name : {}\nDate actuelle : {}\nType de fichiers analysés : {}\n\nFichiers attendus :\n\t{:?}\nFichiers trouvés :\n\t{:?}\n",
            parent_name_str,
            formatted_time,
            file_type,
            missing_files_str.join(", "),
            expected_files
        );

        // Crée le dossier "logs" s'il n'existe pas
        if let Err(err) = fs::create_dir_all("logs") {
            eprintln!("Erreur lors de la création du dossier 'logs': {}", err);
            return;
        }

        write_in_logs(log_path, log_message);
    }
}

/// Parcours un répertoire et collecte les fichiers avec une extension spécifiée.
fn explore_directory(root_path: &str, file_type: FileType) -> Result<Vec<PathBuf>, io::Error> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_file() && entry_path.extension().is_some() {
                    let file_extension = entry_path.extension().unwrap().to_string_lossy().to_lowercase();
                    if file_extension == file_type_to_extension(file_type) {
                        result.push(entry_path.clone());
                    }
                } else if entry_path.is_dir() {
                    result.extend(explore_directory(&entry_path.to_string_lossy(), file_type)?);
                }
            }
        }
    }
    Ok(result)
}

/// Formate la date actuelle.
fn get_date() -> String {
    let local_time = Local::now();
    local_time.format("%Y-%m-%d").to_string()
}

fn write_in_logs(log_path: String, log_message: String) {

    let mut file = match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Erreur lors de l'ouverture ou de la création du fichier de log : {}", err);
            return;
        }
    };

    if let Err(err) = writeln!(file, "{}", log_message) {
        eprintln!("Erreur lors de l'écriture dans le fichier de log : {}", err);
    }

}

/// Divise les lignes uniques en quatre listes en fonction de l'extension.
fn split_files_by_extension(unique_lines: &HashSet<String>) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
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

/// Mappe les types de fichiers aux extensions correspondantes.
fn file_type_to_extension(file_type: FileType) -> &'static str {
    match file_type {
        FileType::C => "c",
        FileType::H => "h",
        FileType::DLL => "dll",
        FileType::A => "a",
    }
}

/// Extrait les chemins uniques des fichiers.
fn extract_unique_paths(paths: &[PathBuf]) -> Vec<String> {
    let unique_paths: HashSet<_> = paths.iter().flat_map(|path| path.parent().map(|p| p.to_str().unwrap().to_string())).collect();
    unique_paths.into_iter().collect()
}

fn extract_unique_file_names(paths: &[PathBuf]) -> Vec<String> {
    let unique_names: HashSet<_> = paths
        .iter()
        .filter_map(|path| path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
        .collect();
    unique_names.into_iter().collect()
}
