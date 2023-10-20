@echo off
setlocal EnableDelayedExpansion

REM Définition de variables globales
set "object_files="
set "external_library_files="

REM Lecture des paramètres du fichier compile.txt
if exist "settings.txt" (
    for /f "usebackq tokens=1,* delims=: " %%a in ("settings.txt") do (
        if "%%a"=="SOURCE_DIR" ( set "source_dir=%%b" )
        else if "%%a"=="COMPILE_DIR" ( set "compile_dir=%%b" )
        else if "%%a"=="IGNORE" ( set "ignore_list=%%b" )
        else if "%%a"=="LIB_DIR" ( set "ext_library=%%b" )
        else if "%%a"=="INCLUDE_LIB_NAME" ( set "include_library=%%b" )
        else if "%%a"=="INCLUDE_HEADER " ( set "included_header=%%b" )
        else if "%%a"=="SEARCH_ALSO_IN " ( set "external_directory=%%b" )
    )
)

REM Si les répertoires source_dir et compile_dir ne sont pas définis, les définir
if not defined source_dir ( set "source_dir=src" )
if not defined compile_dir ( set "compile_dir=compile" )

if defined LIB_DIR (
    for %%f in ("%ext_library%\*.dll") do (
        set "library_files=!library_files! -l%%~nf "
        echo External library file : "%%f" >> "%log_file%"
    )
)

if defined LIB_NAME (
    for %%n in (%LIB_NAME%) do (
        if exist "%ext_library%\%%n.dll" (
            set "library_files=!library_files! -l%%n "
            echo External library file : "%%n.dll" >> "%log_file%"
        )
    )
)

REM Gestion du timestamp pour les logs ; Obtenir la date au format YYYY-MM-DD
for /f "tokens=1-3 delims=:. " %%a in ("%time%") do ( set "time_stamp=%%ah%%bm!time:~0,-3!s%%c" )
for /f "tokens=1-3 delims=/-" %%d in ("%date%") do ( set "date_stamp=%%d-%%e-%%f" )

call :CREATE_DIRECTORY

if defined ignore_list (
    for %%i in (%ignore_list%) do ( set "ignore_files=%%i;%ignore_files%" )
)

REM Ajout du temps de compilation au fichier log
echo time: %time% >> "%log_file%"

echo Compilation des fichiers source...
call :COMPILE_SOURCE_FILES
call :COMPILE_HEADER_FILES

set "compilation_success=1"

echo Searching for object files...
call :COMPILE_OBJECT_FILES

echo Searching for external library files...
call :COMPILE_EXTERNAL_LIBRARY_FILES

echo Linkage des fichiers : >> "%log_file%"
call :LINKAGE_ALL_FILES

%gcc_command% >> "%log_file%" 2>&1

REM Si erreur, ajouter le message au fichier log
if errorlevel 1 (
    echo Linkage : FAILED >> "%log_file%"
    echo Erreur Linkage Failed
    exit /b
)

echo Linkage : DONE >> "%log_file%"
REM Exécution de main.exe
if %compilation_success% equ 1 (
    echo Execution de main.exe...
    echo Exécution de main.exe : >> "%log_file%"
    "%compile_dir%\exe\main.exe"

    if not errorlevel 1 (echo Compilation terminee. >> "%log_file%")
    else (echo Echec de l'execution. >> "%log_file%")

) else (
    echo Echec de la compilation. >> "%log_file%"
    echo Recherche de fichiers dans les répertoire externe... >> "%log_file%"

    for %%D in (!external_directories:,= !) do (
        set "current_directory=%%D"
        REM Appel de la sous-routine
        REM Recherche de fichiers .c dans le répertoire courant
        for %%C in ("%current_directory%\*.c") do (
            set "compilation_success=1"
            echo Fichier .c trouvé : "%%C"
            REM HERE

        )

        REM Recherche de fichiers .h dans le répertoire courant
        for %%H in ("%current_directory%\*.h") do (
            set "compilation_success=1"
            echo Fichier .h trouvé : "%%H"
            REM traitement
        )

        REM Recherche de fichiers .dll dans le répertoire courant
        for %%F in ("%current_directory%\*.dll") do (
            set "compilation_success=1"
            echo Fichier .dll trouvé : "%%F"
            REM traitement
        )
    )
    %gcc_command% >> "%log_file%" 2>&1
)

:CREATE_DIRECTORY
    set "log_file=%compile_dir%\logs\%date_stamp%_%time_stamp%.txt"
    if not exist "%compile_dir%\logs" mkdir "%compile_dir%\logs"
    if not exist "%compile_dir%\build" mkdir "%compile_dir%\build"
    if not exist "%compile_dir%\exe" mkdir "%compile_dir%\exe"
exit /b

:COMPILE_SOURCE_FILES
    for /r "%source_dir%" %%F in (*.c) do (
        set "skip_file="
        for %%i in (%ignore_list%) do (
            if "%%~dpnxF" equ "%source_dir%\%%~i" (
                set "skip_file=1"
            )
        )
        if not defined skip_file (
            set "source_file=%%~nF"
            set "relative_path=%%~dpF"
            set "relative_path=!relative_path:%source_dir%=!"
            set "relative_path=!relative_path:\=!"
            set "build_path=%compile_dir%\build\!relative_path!"
            if not "!relative_path!" == "" (
                mkdir "!build_path!" 2>nul
            )
            REM Compilation avec redirigeant les erreurs vers le fichier log
            echo Compilation de "%%~nF.c" : >> "%log_file%"
            gcc -c "%%F" -o "!build_path!\%%~nF.o" -Wall >> "%log_file%" 2>&1
            REM Si erreur, ajouter le message au fichier log
            if errorlevel 1 (
                echo %%~nF.c : FAILED >> "%log_file%"
                echo     %%~nF.c ... failed
            ) else (
                echo %%~nF.c : DONE >> "%log_file%"
                echo     %%~nF.c ... done
            )
        )
    )
exit /b

:COMPILE_HEADER_FILES
    for /r "%source_dir%" %%H in (*.h) do (
        set "skip_file="
        for %%i in (%ignore_list%) do (
            set "compare_path=%source_dir%\%%~i"
            if "%%~dpH%%~nxH" equ "%source_dir%\%%~i" (
                set "skip_file=1"
            )
        )
        if not defined skip_file (
            set "relative_path=%%~dpH"
            set "relative_path=!relative_path:%source_dir%=!"
            set "relative_path=!relative_path:\=!"
            set "build_path=%compile_dir%\build\!relative_path!"
            if not "!relative_path!" == "" (
                mkdir "!build_path!" 2>nul
            )
            echo Copie de %%H...
            copy "%%H" "!build_path!\" 2>nul
        )
    )
exit /b

:COMPILE_OBJECT_FILES
    for /r "%compile_dir%\build\" %%O in (*.o) do (
        set "object_files=!object_files! "%%~fO""
        echo Object file : "%%~fO" >> "%log_file%"
    )
exit /b

:COMPILE_EXTERNAL_LIBRARY_FILES
    for %%f in ("%ext_library%\*.a" "%ext_library%\*.dll") do (
        for %%i in (%include_library%) do (
            set "filename=%%~nxf"
            if "!filename:install.res.=!"=="%%~nxf" (
                REM Remove 'lib' prefix and extension
                set "filename=!filename:~3,-4!"
                if "%%i"=="!filename!" (
                    set "external_library_files=!external_library_files! -lname!filename! "
                )
            )
        )
    )
exit /b

:LINKAGE_ALL_FILES
    if not defined external_library_files (
        set "gcc_command=gcc !object_files! -o "%compile_dir%\exe\main.exe" -Wall"
        echo No library file found >> "%log_file%"
    ) else ( 
        set "gcc_command=gcc !object_files! -o "%compile_dir%\exe\main.exe" -Wall -I%ext_library% -L%ext_library% %external_library_files%"
        echo Library file found >> "%log_file%"
    )
exit /b

endlocal
echo.
pause
