@echo off
setlocal EnableDelayedExpansion

REM Lecture des paramètres du fichier compile.txt
set "compile_txt=compile.txt"

if exist "%compile_txt%" (
    for /f "usebackq tokens=1,* delims=: " %%a in ("%compile_txt%") do (
        if "%%a"=="SOURCE_DIR" (
            set "source_dir=%%b"
        ) else if "%%a"=="COMPILE_DIR" (
            set "compile_dir=%%b"
        ) else if "%%a"=="IGNORE" (
            set "ignore_list=%%b"
        )
    )
)

REM Si les répertoires source_dir et compile_dir ne sont pas définis, les définir
if not defined source_dir (
    set "source_dir=src"
)

if not defined compile_dir (
    set "compile_dir=compile"
)

REM Gestion du timestamp pour les logs
for /f "tokens=1-4 delims=:." %%a in ("%time%") do (
    set "time_stamp=%date:~6,4%-%date:~3,2%-%date:~0,2%_%%a-%%b-%%c"
)

for /f "tokens=1 delims=," %%i in ("%time_stamp%") do set "log_subdir=%%i"

set "log_subdir=%compile_dir%\logs\%log_subdir%"

REM Création des répertoires
if not exist "%log_subdir%" mkdir "%log_subdir%"

if not exist "%compile_dir%\build" mkdir "%compile_dir%\build"
if not exist "%compile_dir%\exe" mkdir "%compile_dir%\exe"

REM Traitement de la liste des fichiers à ignorer
if defined ignore_list (
    for %%i in (%ignore_list%) do (
        set "ignore_files=%%i;%ignore_files%"
    )
)

REM Variable pour stocker le chemin du fichier log
set "log_file=%log_subdir%\log.txt"

REM Ajout du temps de compilation au fichier log
echo time: %time% >> "%log_file%"

REM Compilation des fichiers .c
for /r "%source_dir%" %%F in (*.c) do (
    set "skip_file="
    for %%i in (%ignore_list%) do (
        if "%%~dpnxF" equ "%source_dir%\%%~i" (
            set "skip_file=1"
        )
    )
    if not defined skip_file (
        set "source_file=%%~nF"
        echo Compilation de %%~nF.c ...
        REM Compilation avec redirigeant les erreurs vers le fichier log
        echo Compilation de "%%~nF.c" : >> "%log_file%"
        gcc -c "%%F" -o "%compile_dir%\build\%%~nF.o" -Wall >> "%log_file%" 2>&1
        REM Si erreur, ajouter le message au fichier log
        if errorlevel 1 (
            echo %%~nF.c : FAILED >> "%log_file%"
        ) else (
            echo %%~nF.c : DONE >> "%log_file%"
        )
    )
)

REM Copie des fichiers .h
for /r "%source_dir%" %%H in (*.h) do (
    set "skip_file="
    for %%i in (%ignore_list%) do (
        set "compare_path=%source_dir%\%%~i"
        if "%%~dpH%%~nxH" equ "%source_dir%\%%~i" (
            set "skip_file=1"
        )
    )
    if not defined skip_file (
        echo Copie de %%H...
        copy "%%H" "%compile_dir%\build\"
    )
)

REM Variable pour suivre le succès de la compilation
set "compilation_success=1"

REM Collect all .o files in the compile directory
set "object_files="
for %%O in ("%compile_dir%\build\*.o") do (
    set "object_files=!object_files! "%%~fO""
)

REM Generate the gcc command to link the object files
echo Linkage des fichiers .o : >> "%log_file%"
set "gcc_command=gcc !object_files! -o "%compile_dir%\exe\main.exe" -Wall"

REM Execute the gcc command and redirect both stdout and stderr to log file
%gcc_command% >> "%log_file%" 2>&1

REM Si erreur, ajouter le message au fichier log
if errorlevel 1 (
    echo Linkage : FAILED >> "%log_file%"
) else (
    echo Linkage : DONE >> "%log_file%"
    REM Exécution de main.exe
    if %compilation_success% equ 1 (
        echo Execution de main.exe...
        echo Exécution de main.exe : >> "%log_file%"
        "%compile_dir%\exe\main.exe"
        if not errorlevel 1 (
            echo Compilation terminee. >> "%log_file%"
        ) else (
            echo Echec de l'execution. >> "%log_file%"
        )
    ) else (
        echo Echec de la compilation. >> "%log_file%"
    )
)

endlocal
pause
