# PowerShell pre-commit hook script for Windows users

Write-Host "Running pre-commit hook..."

# Check if cargo is available
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cargo is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Verify formatting
Write-Host "Checking code formatting..."
$formatOutput = (cargo fmt --all -- --check) 2>&1
$formatExitCode = $LASTEXITCODE

if ($formatExitCode -ne 0) {
    Write-Host "❌ Code formatting check failed!" -ForegroundColor Red
    Write-Host $formatOutput
    Write-Host "Run 'cargo fmt' to fix formatting issues." -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Code formatting looks good!" -ForegroundColor Green

# Run tests
Write-Host "Running tests..."
$testOutput = (cargo test) 2>&1
$testExitCode = $LASTEXITCODE

if ($testExitCode -ne 0) {
    Write-Host "❌ Tests failed!" -ForegroundColor Red
    Write-Host $testOutput
    exit 1
}

Write-Host "✅ All tests passed!" -ForegroundColor Green

Write-Host "Pre-commit hook completed successfully!" -ForegroundColor Green
exit 0