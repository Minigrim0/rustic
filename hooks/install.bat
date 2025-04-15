@echo off
setlocal enabledelayedexpansion

REM Windows installation script for git hooks

REM Get the git hooks directory
FOR /F "tokens=*" %%a IN ('git rev-parse --git-dir') DO SET GIT_DIR=%%a
SET HOOKS_DIR=%GIT_DIR%\hooks

REM Get the script directory
SET SCRIPT_DIR=%~dp0

echo Installing git hooks...

REM Copy the pre-commit hooks
echo Copying Bash version of pre-commit hook...
copy "%SCRIPT_DIR%\pre-commit" "%HOOKS_DIR%\pre-commit"

echo Copying PowerShell version of pre-commit hook...
copy "%SCRIPT_DIR%\pre-commit.ps1" "%HOOKS_DIR%\pre-commit.ps1"

REM Create a pre-commit.cmd wrapper to run the PowerShell script
echo Creating pre-commit.cmd wrapper script...
(
  echo @echo off
  echo powershell.exe -ExecutionPolicy Bypass -File "%%~dp0\pre-commit.ps1"
  echo exit /b %%ERRORLEVEL%%
) > "%HOOKS_DIR%\pre-commit.cmd"

echo.
echo Hooks installed successfully!
echo Pre-commit hook will now run before each commit to check formatting and run tests.
echo.
echo Both Bash and PowerShell versions of the pre-commit hook have been installed.
echo PowerShell version will be used by default on Windows systems.
echo.

pause