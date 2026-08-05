# Hardware / boot path

**Kernel lives in `../kernel/`** — Rust `no_std` + bootloader + QEMU.

```powershell
cd ../kernel
cargo +nightly bootimage -p reality_os_kernel --release
```

Boot image copy: `../data/reality_os_kernel.img`  
Serial log: `../data/reality_os_qemu_serial.log`
