//! # Projet de Vérification des Fichiers d'un Projet Rust
//!
//! Ce programme en Rust parcourt un répertoire et vérifie la cohérence des fichiers du projet.

extern crate chrono;
extern crate tokio;

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions, metadata};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::{thread, env};

use tokio::process::Command as AsyncCommand;

use chrono::Local;

/// Représente les types de fichiers que nous recherchons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FileType {
    C,
    H,
    DLL,
    A,
    O,
}

// Déclarer les variables globales
static mut FORMATTED_TIME: Option<String> = None;
static mut LOG_PATH: Option<String> = None;
static mut PROJECT_PATH: Option<String> = None;
static mut PROJECT_NAME: Option<String> = None;
static mut TARGET_PATH: Option<String> = None;
static mut LOG_FILE: Option<String> = None;


fn _display_pathbufs(paths: &Vec<PathBuf>) {
    for path in paths {
        println!("{}", path.display());
    }
}

/// Fonction principale du programme.
#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    
    let project_path = args[1].clone(); // Le projet à compiler
    let target_path = args[2].clone();  // Destinations des fichiers compilés
    let mut project_name = String::new(); // Initialisation à une chaîne vide par défaut
    let path = Path::new(&project_path);

    // Utilise la méthode file_name pour obtenir la dernière partie du chemin
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            project_name = file_name_str.to_string();
        }
    }

    let start_time: SystemTime = SystemTime::now();
    
    unsafe { 

        PROJECT_NAME            = Some(project_name.to_string()); 
        PROJECT_PATH            = Some(project_path.clone()); 
        TARGET_PATH             = Some(format!("{}/{}", target_path.clone(), project_name.to_string() )); 
        FORMATTED_TIME          = Some(get_date());
        LOG_PATH                = Some(format!("./{}/{}/logs/", TARGET_PATH.as_ref().unwrap(), PROJECT_NAME.as_ref().unwrap()));
        LOG_FILE                = Some(format!("{}_{}.log", PROJECT_NAME.as_ref().unwrap(), FORMATTED_TIME.as_ref().unwrap()));

    }

    let project_path: String    = get_project_path();
    let target_path: String     = get_target_path();

    // Crée un dossier portant le nom du répertoire actuel
    create_directory();
    // Collecte les fichiers avec les extensions spécifiées
    let c_files: Vec<PathBuf> = collect_files(&project_path, FileType::C);
    let h_files: Vec<PathBuf> = collect_files(&project_path, FileType::H);
    let dll_files: Vec<PathBuf> = collect_files(&project_path, FileType::DLL);
    let a_files: Vec<PathBuf> = collect_files(&project_path, FileType::A);
    let o_files: Vec<PathBuf> = collect_files(&project_path, FileType::O);
    let unique_library_files: HashSet<String> = update_library_list(&c_files);

    let total_files: usize = c_files.len() + h_files.len() + dll_files.len() + a_files.len() + o_files.len() + unique_library_files.len();

    // Copie les fichiers .h dans un dossier "header", les fichiers .c ensemble, les .o ensemble, les .dll ensemble
    copy_files_to_directory(&h_files, "source");
    copy_files_to_directory(&c_files,  "source");
    copy_files_to_directory(&o_files,  "output");
    copy_files_to_directory(&dll_files,  "dll");
    copy_files_to_directory(&a_files,  "a");

    let _path_output: String = format!("{}{}", &target_path, "\\output");
    let _path_source: String= format!("{}{}", &target_path, "\\source");

    let o_files: Vec<PathBuf> = collect_files(&target_path, FileType::O);
    let h_files: Vec<PathBuf> = collect_files(&target_path, FileType::H);

    // Optionnel: Fouiller dans les fichiers .c pour extraire les fichiers inclus
    let _exclude_list = get_exclude_list(&c_files);

    if let Ok(_output) = compile_source_to_output(&c_files).await  {
        
        // Refait la série de collect_files pour avoir les chemins des fichiers
        let c_files: Vec<PathBuf> = collect_files(&project_path, FileType::C);
        
        // Divise unique_library_files en quatre listes en fonction de l'extension
        let (expected_files_for_c, expected_files_for_h, expected_files_for_dll, expected_files_for_a, expected_files_for_o) =
            split_files_by_extension(&unique_library_files);

        // Vérifie si les fichiers inclus sont présents dans les listes c_files, h_files, dll_files et a_files
        check_files_then_log("C", &c_files, &expected_files_for_c);
        check_files_then_log("H", &h_files, &expected_files_for_h);
        check_files_then_log("DLL", &dll_files, &expected_files_for_dll);
        check_files_then_log("A", &a_files, &expected_files_for_a);
        check_files_then_log("O", &o_files, &expected_files_for_o);

        // Affiche le temps d'exécution et le nombre total de fichiers traités
        if let Ok(elapsed_time) = start_time.elapsed() {
            let elapsed_secs = elapsed_time.as_secs();
            let elapsed_millis = elapsed_time.subsec_millis();
            
            let log_message: String = format!("Temps d'exécution : {}.{:03} secondes\nNombre de fichiers traités : {}\n", elapsed_secs, elapsed_millis, total_files);

            write_in_logs(log_message);

        }

        let include_paths: Vec<String>  = extract_unique_paths(&h_files);
        let library_paths: Vec<String>  = extract_unique_paths(&dll_files);
        let libraries: Vec<String>      = extract_unique_file_names(&dll_files);

        if let Ok(_output) = compile_output_to_executable(o_files.clone(), include_paths, library_paths, libraries).await  {

            // Affiche le temps d'exécution et le nombre total de fichiers traités
            if let Ok(elapsed_time) = start_time.elapsed() {
                let elapsed_secs = elapsed_time.as_secs();
                let elapsed_millis = elapsed_time.subsec_millis();

                let log_message: String = format!("Temps d'exécution Total : {}.{:03} secondes", elapsed_secs, elapsed_millis);

                println!("Time : {}", log_message);

                write_in_logs(log_message);
            }

            execute_main();
        }

    }






}

fn get_log_path() -> String {
    unsafe {  
    match &LOG_PATH {
        Some(value) => return value.to_string(),
        None => {
            return "".to_string();
        }
    }

    } 
}

fn get_log_file() -> String {
    unsafe {  
    match &LOG_FILE {
        Some(value) => return value.to_string(),
        None => {
            return "".to_string();
        }
    }

    } 
}

/// Obtient la liste des fichiers à exclure.
fn get_exclude_list(c_files: &[PathBuf]) -> Vec<String> {
    let mut exclude_list = Vec::new();

    for c_file in c_files {
        if let Ok(file) = File::open(c_file) {
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.starts_with("// EXCLUDE:") {
                        let excluded_file = line.trim_start_matches("// EXCLUDE:").trim().to_string();
                        exclude_list.push(excluded_file);
                    }
                }
            }
        }
    }

    exclude_list
}

async fn compile_output_to_executable(o_files: Vec<PathBuf>, include_paths: Vec<String>, library_paths: Vec<String>, libraries: Vec<String>) -> Result<Vec<u8>, std::io::Error> {

    // Générer la commande
    let mut command: Command = Command::new("gcc");

    let target_path = get_target_path();
    
    let path_exe = format!("{}{}", target_path, "\\executable\\main.exe");

    // Ajouter les fichiers .o
    command.args(&["-o", &path_exe]).args(&o_files);

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

    let command_str = format!("Commande réalisée pour l'exécution du projet : \n\t{:?}\n", command);
    
    write_in_logs(command_str);

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let child = command.spawn()?;
    let output = child.wait_with_output()?;

    // Affiche la sortie standard
    println!("\nSortie de la commande :{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        // Affiche l'erreur standard en cas d'échec
        eprintln!("Erreur lors de l'exécution du main, Erreur, la commande a échoué :\n{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output.stdout)
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
fn update_library_list(c_files: &[PathBuf]) -> HashSet<String> {
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
                            put_library(&line, &unique_lines_mutex);
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
fn put_library(line: &str, unique_lines_mutex: &Arc<Mutex<HashSet<String>>>) {
    // Trouve les positions des guillemets
    let start_quote = line.find('"');
    let end_quote = line.rfind('"');

    // Si les guillemets sont présents, stocke le texte entre eux
    if let (Some(start), Some(end)) = (start_quote, end_quote) {
        let include_text = line[start + 1..end].to_string();
        let mut unique_library_files = unique_lines_mutex.lock().unwrap();
        unique_library_files.insert(include_text);
    } else {
        // Sinon, recherche les symboles '<' et '>'
        let start_bracket = line.find('<');
        let end_bracket = line.rfind('>');

        // Si les symboles sont présents, stocke le texte entre eux
        if let (Some(start), Some(end)) = (start_bracket, end_bracket) {
            let include_text = line[start + 1..end].to_string();
            let mut unique_library_files = unique_lines_mutex.lock().unwrap();
            unique_library_files.insert(include_text);
        }
    }
}


/// Vérifie si les fichiers inclus sont présents dans la liste de fichiers et log les avertissements si nécessaire.
fn check_files_then_log(file_type: &str, file_list: &[PathBuf], expected_files: &[PathBuf]) {
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

        let formatted_time = get_formatted_time();
        let current_path: PathBuf = std::env::current_dir().expect("Impossible d'obtenir le répertoire actuel");

        let current_folder_name: Option<&str> = current_path.file_name().and_then(|n| n.to_str());
        let current_folder_name_str: String = current_folder_name.unwrap_or_default().to_string();

        // Construit le message de log complet
        let log_message = format!(
            "Project Name : {}\nDate actuelle : {}\nType de fichiers analysés : {}\n\nFichiers attendus :\n\t{:?}\nFichiers trouvés :\n\t{:?}\n",
            current_folder_name_str,
            formatted_time,
            file_type,
            missing_files_str.join(", "),
            expected_files
        );

        write_in_logs(log_message);
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

fn write_in_logs(log_message: String) {

    let log_path = format!("{}/{}", get_log_path(), get_log_file());

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
fn split_files_by_extension(unique_library_files: &HashSet<String>) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut c_files = Vec::new();
    let mut h_files = Vec::new();
    let mut dll_files = Vec::new();
    let mut a_files = Vec::new();
    let mut o_files = Vec::new();

    for file in unique_library_files {
        let file_extension = Path::new(&file)
            .extension()
            .map(|ext| ext.to_string_lossy())
            .unwrap_or_default();

        match file_extension.to_lowercase().as_str() {
            "c" => c_files.push(PathBuf::from(file)),
            "h" => h_files.push(PathBuf::from(file)),
            "dll" => dll_files.push(PathBuf::from(file)),
            "a" => a_files.push(PathBuf::from(file)),
            "o" => o_files.push(PathBuf::from(file)),
            _ => (),
        }
    }

    (c_files, h_files, dll_files, a_files, o_files)
}

/// Mappe les types de fichiers aux extensions correspondantes.
fn file_type_to_extension(file_type: FileType) -> &'static str {
    match file_type {
        FileType::C => "c",
        FileType::H => "h",
        FileType::DLL => "dll",
        FileType::A => "a",
        FileType::O => "o",
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

fn create_directory() {

    let path: String = format!("{}", get_target_path());

    let directory_paths: Vec<String> = [
        format!("./{}", path),
        format!("{}/executable", path),
        format!("{}/source", path),
        format!("{}/output", path),
        format!("{}/dll", path),
        format!("{}/a", path),
        format!("{}", get_log_path()),

    ].to_vec();

    for directory_path in directory_paths {

        if !Path::new(&directory_path).exists() {
            if let Err(err) = fs::create_dir_all(&directory_path) {
                eprintln!("Erreur lors de la création du dossier '{}': {}", directory_path, err);
            }
        }

    }

}

fn copy_files_to_directory(files: &[PathBuf], destination_folder: &str) {
    let destination_path = format!("{}/{}", get_target_path(), destination_folder);

    for file in files {
        let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let destination_file_path = format!("{}/{}", destination_path, file_name);
        // Vérifie si le fichier existe déjà dans le dossier de destination
        if !metadata(&destination_file_path).is_ok() {
            // Copie le fichier vers le dossier de destination
            if let Err(err) = fs::copy(&file, &destination_file_path) {
                eprintln!("Erreur lors de la copie du fichier '{}' : {}", file_name, err);
            }
        }
    }
}

fn get_project_path() -> String {
    unsafe {  
    match &PROJECT_PATH {
        Some(value) => return value.to_string(),
        None => {
            return "".to_string();
        }
    }

    } 
}

fn get_formatted_time() -> String {
    unsafe {  
    match &FORMATTED_TIME {
        Some(value) => return value.to_string(),
        None => {
            return "".to_string();
        }
    }

    } 
}

fn get_target_path() -> String {
    unsafe {  
    match &TARGET_PATH {
        Some(value) => return value.to_string(),
        None => {
            return "".to_string();
        }
    }

    } 
}

async fn compile_source_to_output(c_files: &[PathBuf]) -> Result<Vec<PathBuf>, io::Error> {
    let target_path = get_target_path();
    let mut output_files: Vec<PathBuf> = Vec::new();

    for c_file in c_files {
        
        let path: String = format!("{}{}", target_path, "\\output");
        let mut output_file: PathBuf = PathBuf::from(path);
        output_file.push(c_file.file_name().unwrap());
        output_file.set_extension("o");

        let c_file_str: String = c_file.to_str().unwrap().replace("\\", "/");
        let output_file_str: String = output_file.to_str().unwrap().replace("\\", "/");

        let mut command: Command = Command::new("gcc");
        command.args(&[&c_file_str, "-c", "-o", &output_file_str]);
        // Créez une nouvelle instance de Command avec les mêmes arguments
        let mut cloned_command = AsyncCommand::new("gcc");
        cloned_command.args(&[&c_file_str, "-c", "-o", &output_file_str]);
           
        // Passez la nouvelle instance de tokio::process::Command à AsyncCommand::from
        let mut async_command = AsyncCommand::from(cloned_command);

        // Attendre la sortie de la commande de manière asynchrone
        let output: Output = async_command.output().await?;
        if output.status.success() {

            output_files.push(output_file);
        } else {
            let error_message = format!(
                "La compilation a échoué. Erreur : {}\nCommande exécutée : {:?}\nSortie de la commande : {}",
                String::from_utf8_lossy(&output.stderr),
                command,
                String::from_utf8_lossy(&output.stdout),
            );
            eprintln!("Erreur lors de la compilation du fichier {:?}: {}", c_file, error_message);
        }
    }

    Ok(output_files)

}

fn execute_main() {

    let target_path = get_target_path();
    
    let path = format!("{}{}{}", "./", target_path, "\\executable\\main");

    let mut command = Command::new(path.clone());

    match command.status() {
        Ok(status) => {
            if status.success() {
                println!("\nExécutable '{}' a été exécuté avec succès.", path);
            } else {
                eprintln!("Erreur lors de l'exécution de l'exécutable '{}'. Le processus a renvoyé un code d'erreur.", path);
            }
        }
        Err(err) => {
            eprintln!("Erreur lors de l'exécution de l'exécutable '{}': {}", path, err);
        }
    }
}
