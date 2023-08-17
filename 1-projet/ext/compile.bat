@echo off
setlocal

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

if not defined source_dir (
    set "source_dir=src"
)

if not defined compile_dir (
    set "compile_dir=compile"
)

for /f "tokens=1-4 delims=:." %%a in ("%time%") do (
    set "time_stamp=%date:~6,4%-%date:~3,2%-%date:~0,2%_%%a-%%b-%%c"
)

for /f "tokens=1 delims=," %%i in ("%time_stamp%") do set "log_subdir=%%i"

set "log_subdir=%compile_dir%\logs\%log_subdir%"

if not exist "%log_subdir%" mkdir "%log_subdir%"

if not exist "%compile_dir%\build" mkdir "%compile_dir%\build"
if not exist "%compile_dir%\exe" mkdir "%compile_dir%\exe"

if defined ignore_list (
    for %%i in (%ignore_list%) do (
        set "ignore_files=%%i;%ignore_files%"
    )
)

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
        gcc -c "%%F" -o "%compile_dir%\build\%%~nF.o" -Wall >> "%log_subdir%\log_%%~nF.txt" 2>&1
    )
)

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

set "compilation_success=1"

for %%O in ("%compile_dir%\build\*.o") do (
    set "object_file=%%~nO"
    echo edition des liens pour %%~nO.o ...
    gcc "%%O" -o "%compile_dir%\exe\main.exe" -Wall >> "%log_subdir%\log_link.txt" 2>&1
    if errorlevel 1 (
        set "compilation_success=0"
    )
)

if %compilation_success% equ 1 (
    echo Execution de %source_file%.exe...
    "%compile_dir%\exe\main.exe"
    if not errorlevel 1 (
        del "%log_subdir%\log_link.txt"
        echo %time% >> "%log_subdir%\log_time.txt"
        echo Compilation terminee.
    ) else (
        echo Echec de l'execution.
    )
) else (
    echo Echec de la compilation.
)

endlocal
pause
