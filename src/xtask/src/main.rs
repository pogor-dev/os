use std::path::PathBuf;

use anyhow::{Result, bail};
use bootloader::{BiosBoot, UefiBoot};
use duct::cmd;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args.next(); // skip binary name
    let command = args.next().unwrap_or_else(|| "help".into());

    match command.as_str() {
        "build" => build(),
        "iso" => iso(),
        "run" => run(),
        "test-integration" => test_integration(),
        "help" | _ => return help(),
    }
}

fn help() -> Result<()> {
    println!("Available commands:");
    println!("  build             Build the kernel");
    println!("  iso               Build the kernel and create an ISO image");
    println!("  run               Build and run the kernel in QEMU");
    println!("  test-integration  Build and run kernel tests in QEMU");
    Ok(())
}

fn build() -> Result<()> {
    cmd!(
        "cargo",
        "+nightly",
        "build",
        "--release",
        "-Zbuild-std=core,compiler_builtins,alloc",
        "-Zbuild-std-features=compiler-builtins-mem",
        "--manifest-path",
        "crates/kernel/Cargo.toml",
        "--target",
        "target.json",
        "--target-dir",
        "target/kernel",
    )
    .run()?;
    Ok(())
}

fn iso() -> Result<()> {
    // Determine the path to the kernel ELF
    let kernel_path = PathBuf::from("./target/kernel/target/release/kernel");

    // Create publish directory if it doesn't exist
    let publish_dir = PathBuf::from("../publish");
    if !publish_dir.exists() {
        std::fs::create_dir_all(&publish_dir)?;
    }

    // Create BIOS bootable image
    let bios_path = PathBuf::from("../publish/bios.img");
    BiosBoot::new(&kernel_path)
        .create_disk_image(&bios_path)
        .expect("Failed to create BIOS disk image");

    // Create UEFI bootable image
    let uefi_path = PathBuf::from("../publish/uefi.img");
    UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI disk image");

    println!("Bootable images created in publish folder: bios.img and uefi.img");
    Ok(())
}

fn run() -> Result<()> {
    build()?;
    iso()?;

    let current_dir = std::env::current_dir()?;
    let image_path = current_dir.join("..").join("publish").join("uefi.img");

    cmd!(
        "qemu-system-x86_64",
        "-serial",
        "stdio",
        "-display",
        "none",
        "-no-reboot",
        // "-m 128M",
        // "-smp 2",
        // "-accel tcg",
        "-cdrom",
        image_path.to_str().unwrap(),
        "-boot",
        "d"
    )
    .run()?;
    Ok(())
}

fn test_integration() -> Result<()> {
    build()?;
    let output = cmd!(
        "qemu-system-x86_64",
        "-serial",
        "stdio",
        "-display",
        "none",
        "-no-reboot",
        "-cdrom",
        "target/x86_64-unknown-none/debug/bootimage-kernel.bin"
    )
    .stderr_to_stdout()
    .stdout_capture()
    .read()?;

    if output.contains("Integration test passed") {
        println!("Integration Test Passed!");
        Ok(())
    } else {
        bail!("Integration Test Failed!\n\nCaptured Output:\n{}", output);
    }
}
