# Build and QEMU-boot FSOT Reality OS kernel (Rust no_std).
# Usage (from repo root or kernel/):
#   pwsh kernel/scripts/build_and_run.ps1
#   pwsh kernel/scripts/build_and_run.ps1 -BuildOnly

param(
    [switch]$BuildOnly
)

$ErrorActionPreference = "Stop"
$KernelRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
if (-not (Test-Path (Join-Path $KernelRoot "crates\reality_os_kernel\Cargo.toml"))) {
    $KernelRoot = Split-Path $PSScriptRoot -Parent
}
Set-Location $KernelRoot
Write-Host "=== FSOT Reality OS kernel build ===" -ForegroundColor Cyan
Write-Host "Kernel root: $KernelRoot"

# Ensure nightly components for bootimage
rustup show | Out-Null
$env:RUSTFLAGS = ""

Write-Host ">>> cargo build -p reality_os_kernel --release" -ForegroundColor Yellow
cargo build -p reality_os_kernel --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

Write-Host ">>> cargo bootimage -p reality_os_kernel --release" -ForegroundColor Yellow
cargo bootimage -p reality_os_kernel --release
if ($LASTEXITCODE -ne 0) { throw "cargo bootimage failed" }

$img = Get-ChildItem -Path "$KernelRoot\target" -Recurse -Filter "bootimage-reality_os_kernel.bin" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $img) {
    throw "bootimage-reality_os_kernel.bin not found under target/"
}
Write-Host "Boot image: $($img.FullName)" -ForegroundColor Green

$outDir = Join-Path (Split-Path $KernelRoot -Parent) "data"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$copy = Join-Path $outDir "reality_os_kernel.img"
Copy-Item $img.FullName $copy -Force
Write-Host "Copied: $copy"

if ($BuildOnly) {
    Write-Host "BuildOnly — skip QEMU"
    exit 0
}

$qemu = $null
foreach ($c in @(
        "qemu-system-x86_64",
        "C:\Program Files\qemu\qemu-system-x86_64.exe"
    )) {
    if (Get-Command $c -ErrorAction SilentlyContinue) { $qemu = (Get-Command $c).Source; break }
    if (Test-Path $c) { $qemu = $c; break }
}
if (-not $qemu) { throw "qemu-system-x86_64 not found" }

$log = Join-Path $outDir "reality_os_qemu_serial.log"
Write-Host ">>> QEMU boot (serial capture) $qemu" -ForegroundColor Yellow
# Capture serial; use debug-exit so QEMU exits after kernel marker
& $qemu `
    -drive "format=raw,file=$($img.FullName)" `
    -display none `
    -serial file:$log `
    -device isa-debugcon,chardev=serial0,iobase=0xe9 `
    -chardev "file,path=$log,id=serial0" `
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 `
    -no-reboot `
    -no-shutdown 2>$null
# qemu may return non-zero via debug-exit — still check log

if (-not (Test-Path $log)) {
    # alternate single-serial invocation
    & $qemu `
        -drive "format=raw,file=$($img.FullName)" `
        -display none `
        -serial stdio `
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 `
        -no-reboot 2>&1 | Tee-Object -FilePath $log
}

Write-Host "=== Serial log (tail) ===" -ForegroundColor Cyan
if (Test-Path $log) {
    Get-Content $log -Tail 40
    $text = Get-Content $log -Raw
    if ($text -match "FSOT_ROS_OVERALL=ok" -and $text -match "FSOT_QEMU_DISK_BOOT=ok") {
        Write-Host "REALITY_OS_QEMU_BOOT: PASS" -ForegroundColor Green
        exit 0
    }
    Write-Host "REALITY_OS_QEMU_BOOT: FAIL (markers missing)" -ForegroundColor Red
    exit 1
}
Write-Host "REALITY_OS_QEMU_BOOT: FAIL (no log)" -ForegroundColor Red
exit 1
