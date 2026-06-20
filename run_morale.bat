@echo off
REM MORALE - AI Skills Auditor
REM Sample batch script to run the auditor

echo MORALE - AI Skills Auditor
echo ===========================

if "%1"=="" (
    echo Usage: %0 ^<path_to_ai_skill^>
    echo Example: %0 C:\path\to\my\ai-skill
    exit /b 1
)

echo Auditing AI skill at: %1
echo Running MORALE...

REM Build the project first
cargo build --release

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

echo.
echo Starting audit...
target\release\morale.exe %1

echo.
echo Audit completed!
pause