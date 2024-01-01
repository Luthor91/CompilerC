//! # Projet de Vérification des Fichiers d'un Projet Rust
//!
//! Ce programme en Rust parcourt un répertoire et vérifie la cohérence des fichiers du projet.

extern crate chrono;

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions, metadata};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
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
    O,
}

// Déclarer les variables globales
static mut FORMATTED_TIME: Option<String> = None;
static mut LOG_PATH: Option<String> = None;
static mut _TARGET_FILE: Option<String> = None;
static mut CURRENT_DIRECTORY: Option<String> = None;

fn _display_pathbufs(paths: &Vec<PathBuf>) {
    for path in paths {
        println!("{}", path.display());
    }
}

/// Fonction principale du programme.
fn main() {
    // Mesure le temps d'exécution
    let start_time: SystemTime = SystemTime::now();

    // Obtient le nom du répertoire actuel
    let current_directory: String = get_current_directory();
    let root_directory: &str = "./files/";
    unsafe { CURRENT_DIRECTORY = Some(current_directory.clone()); }

    // Crée un dossier portant le nom du répertoire actuel
    create_directory("./", &current_directory);
    create_directory(&current_directory, "executable");
    create_directory(&current_directory, "source");
    create_directory(&current_directory, "output");
    create_directory(&current_directory, "dll");
    create_directory(&current_directory, "a");

    // Collecte les fichiers avec les extensions spécifiées
    let c_files: Vec<PathBuf> = collect_files(&root_directory, FileType::C);
    let h_files: Vec<PathBuf> = collect_files(&root_directory, FileType::H);
    let dll_files: Vec<PathBuf> = collect_files(&root_directory, FileType::DLL);
    let a_files: Vec<PathBuf> = collect_files(&root_directory, FileType::A);
    let o_files: Vec<PathBuf> = collect_files(&root_directory, FileType::O);

    let unique_lines: HashSet<String> = update_library_list(&c_files);

    let total_files: usize = c_files.len() + h_files.len() + dll_files.len() + a_files.len() + o_files.len() + unique_lines.len();

    let current_directory = get_current_directory();
    unsafe { CURRENT_DIRECTORY = Some(current_directory.clone()); }

    // Copie les fichiers .h dans un dossier "header", les fichiers .c ensemble, les .o ensemble, les .dll ensemble
    copy_files_to_directory(&h_files, &current_directory, "source");
    copy_files_to_directory(&c_files, &current_directory, "source");
    copy_files_to_directory(&o_files, &current_directory, "output");
    copy_files_to_directory(&dll_files, &current_directory, "dll");
    copy_files_to_directory(&a_files, &current_directory, "a");

    let path_output = format!("{}{}", &current_directory, "\\output");
    let path_source= format!("{}{}", &current_directory, "\\source");

    let o_files: Vec<PathBuf> = collect_files(&path_output, FileType::O);
    let h_files: Vec<PathBuf> = collect_files(&path_source, FileType::H);

    // Optionnel: Fouiller dans les fichiers .c pour extraire les fichiers inclus
    let _exclude_list = get_exclude_list(&c_files);

    let _output_files = compile_c_files_to_output(&c_files);

    // Refait la série de collect_files pour avoir les chemins des fichiers
    let c_files: Vec<PathBuf> = collect_files(&current_directory, FileType::C);
    
    unsafe {
        // Obtient la date formatée
        FORMATTED_TIME = Some(get_date());
        // Obtient le chemin du fichier de log
        LOG_PATH = Some(format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()));
    }

    // Divise unique_lines en quatre listes en fonction de l'extension
    let (expected_files_for_c, expected_files_for_h, expected_files_for_dll, expected_files_for_a, expected_files_for_o) =
        split_files_by_extension(&unique_lines);

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

        let l_log_path: String;
        let mut log_message: String = format!("Temps d'exécution : {}.{:03} secondes", elapsed_secs, elapsed_millis);

        unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }

        write_in_logs(l_log_path.clone(), log_message);

        log_message = format!("Nombre de fichiers traités : {}\n", total_files);

        write_in_logs(l_log_path, log_message);
    }

    let include_paths: Vec<String>  = extract_unique_paths(&h_files);
    let library_paths: Vec<String>  = extract_unique_paths(&dll_files);
    let libraries: Vec<String>      = extract_unique_file_names(&dll_files);

    compile_files_to_output(&c_files, &h_files);

    let command: Command = create_gcc_command(o_files.clone(), include_paths, library_paths, libraries);

    let command_str = format!("Commande réalisée pour l'exécution du projet : \n\t{:?}\n", command);
    let l_log_path:String;

    unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }

    write_in_logs(l_log_path, command_str);

    let _ = execute_gcc_command(command);

    // Affiche le temps d'exécution et le nombre total de fichiers traités
    if let Ok(elapsed_time) = start_time.elapsed() {
        let elapsed_secs = elapsed_time.as_secs();
        let elapsed_millis = elapsed_time.subsec_millis();

        let l_log_path: String;

        unsafe {  l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }

        let log_message: String = format!("Temps d'exécution Total : {}.{:03} secondes", elapsed_secs, elapsed_millis);

        println!("Time : {}", log_message);

        write_in_logs(l_log_path, log_message);
    }

    execute_main();

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

/// Compile les fichiers .c en fichiers .o
fn compile_files_to_output(c_files: &[PathBuf], h_files: &[PathBuf]) {
    let current_directory: String = get_current_directory();

    let output_files: Vec<PathBuf> = c_files.iter()
    .map(|c_file| {
        let mut output_file = PathBuf::from(format!("{}{}", current_directory, "\\output"));
        output_file.push(c_file.file_name().unwrap()); // Utilisez file_name() ici
        output_file.set_extension("o");
        output_file
    })
    .collect();

    for h in h_files {
        println!("header : {}", h.display());
    }
    

    let include_paths = extract_unique_paths(h_files);
    
    for include_path in &include_paths {
        println!("include : {}", include_path);
    }

    for c_file in c_files {
        println!("source : {}", c_file.display());
    }
    
    let command: Command = create_compile_command(c_files, &output_files, include_paths);

    let command_str = format!("Commande réalisée pour la compilation en fichiers .o : \n\t{:?}\n", command);
    let l_log_path: String;

    unsafe { l_log_path = format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()); }

    write_in_logs(l_log_path, command_str);

    let _ = execute_gcc_command(command);
}

/// Crée la commande de compilation en fichiers .o
fn create_compile_command(c_files: &[PathBuf], output_files: &[PathBuf], include_paths: Vec<String>) -> Command {
    // Générer la commande
    let mut command: Command = Command::new("gcc");

    // Ajouter les fichiers .c
    command.args(&*c_files);

    // Ajouter les chemins d'inclusion (-I)
    for include_path in &include_paths {
        command.args(&["-I", include_path]);
    }

    // Vérifier si le vecteur output_files n'est pas vide avant d'ajouter les fichiers .o en tant que sortie
    if !output_files.is_empty() {
        command.arg("-o");
        
        for file in output_files {
            println!("output : {}", file.display());
            command.arg(file.to_str().unwrap());
        }

    } else {
        eprintln!("Erreur: Aucun fichier objet trouvé pour la sortie.");
    }


    command
}


fn create_gcc_command(o_files: Vec<PathBuf>, include_paths: Vec<String>, library_paths: Vec<String>, libraries: Vec<String>) -> Command {

    // Générer la commande
    let mut command: Command = Command::new("gcc");

    let current_directory = unsafe { CURRENT_DIRECTORY.as_ref().unwrap() }; // Utilisez la variable globale CURRENT_DIRECTORY
    
    let path_exe = format!("{}{}", current_directory, "\\executable\\main.exe");

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

    return command;

}

fn execute_gcc_command(mut command: Command) -> Result<Output, io::Error> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = command.output()?;
    
    // Affiche la sortie standard
    println!("\nSortie de la commande :{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        // Affiche l'erreur standard en cas d'échec
        eprintln!("Erreur lors de l'exécution du main, Erreur, la commande a échoué :\n{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output)
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

        unsafe {
            // Obtient la date formatée
            FORMATTED_TIME = Some(get_date());
            // Obtient le chemin du fichier de log
            LOG_PATH = Some(format!("logs/{}.log", FORMATTED_TIME.as_ref().unwrap()));
        }

        let formatted_time = unsafe { FORMATTED_TIME.as_ref().unwrap().clone() };
        let log_path = unsafe { LOG_PATH.as_ref().unwrap().clone() };
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

    // Crée le dossier "logs" s'il n'existe pas
    if let Err(err) = fs::create_dir_all("logs") {
        eprintln!("Erreur lors de la création du dossier 'logs': {}", err);
        return;
    }

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
fn split_files_by_extension(unique_lines: &HashSet<String>) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut c_files = Vec::new();
    let mut h_files = Vec::new();
    let mut dll_files = Vec::new();
    let mut a_files = Vec::new();
    let mut o_files = Vec::new();

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

fn create_directory(base_directory: &str, destination_folder: &str) {
    let directory_path = format!("{}/{}", base_directory, destination_folder);

    // Vérifie si le dossier existe déjà
    if !Path::new(&directory_path).exists() {
        // Crée le dossier s'il n'existe pas déjà
        if let Err(err) = fs::create_dir(&directory_path) {
            eprintln!("Erreur lors de la création du dossier '{}': {}", directory_path, err);
        }
    }
}

fn copy_files_to_directory(files: &[PathBuf], base_directory: &str, destination_folder: &str) {
    let destination_path = format!("{}/{}", base_directory, destination_folder);

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

fn get_current_directory() -> String {
    // Obtenir le répertoire actuel
    match std::env::current_dir() {
        Ok(path) => path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string(),
        Err(err) => {
            eprintln!("Erreur lors de l'obtention du répertoire actuel : {}", err);
            String::new()
        }
    }
}

fn compile_c_to_output(c_file: &PathBuf) -> Result<PathBuf, io::Error> {
    
    let current_directory = unsafe { CURRENT_DIRECTORY.as_ref().unwrap() };
    
    let path = format!("{}{}{}", "./", current_directory, "\\output");
    let mut output_file = PathBuf::from(path);
    output_file.push(c_file.file_name().unwrap());
    output_file.set_extension("o");

    let c_file_str: String = c_file.to_str().unwrap().replace("\\", "/");
    let output_file_str: String = output_file.to_str().unwrap().replace("\\", "/");

    let mut command: Command = Command::new("gcc");
    command.args(&[&c_file_str, "-c", "-o", &output_file_str]);

    let output = command.output()?;
    if output.status.success() {
        Ok(output_file)
    } else {
        let error_message = format!(
            "La compilation a échoué. Erreur : {}\nCommande exécutée : {:?}\nSortie de la commande : {}",
            String::from_utf8_lossy(&output.stderr),
            command,
            String::from_utf8_lossy(&output.stdout),
        );
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}

fn compile_c_files_to_output(c_files: &[PathBuf]) -> Vec<PathBuf> {
    let mut output_files = Vec::new();
    for c_file in c_files {
        match compile_c_to_output(c_file) {
            Ok(output_file) => output_files.push(output_file),
            Err(err) => eprintln!("\nErreur lors de la compilation du fichier {:?}: {}", c_file, err),
        }
    }

    output_files
}

fn execute_main() {

    let current_directory = unsafe { CURRENT_DIRECTORY.as_ref().unwrap() }; // Utilisez la variable globale CURRENT_DIRECTORY
    
    let path = format!("{}{}{}", "./", current_directory, "\\executable\\main");

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
