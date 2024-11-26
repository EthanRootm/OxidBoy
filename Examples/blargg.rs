fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new("./Roms/gb-test-roms").exists() {
        println!("$ git clone --depth=1 https://github.com/retrio/gb-test-roms ./res/gb-test-roms");
        std::process::Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("https://github.com/retrio/gb-test-roms")
            .arg("./Roms/gb-test-roms")
            .spawn()?
            .wait()?;
    }
    println!("$ cargo run -- ./Roms/gb-test-roms/instr_timing/instr_timing.gb");
    std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("./Roms/gb-test-roms/instr_timing/instr_timing.gb")
        .spawn()?
        .wait()?;

    println!("$ cargo run -- ./Roms/gb-test-roms/cpu_instrs/cpu_instrs.gb");
    std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("./Roms/gb-test-roms/cpu_instrs/cpu_instrs.gb")
        .spawn()?
        .wait()?;

    Ok(())
}
