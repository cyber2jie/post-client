use std::env;
use std::process;
fn pack_win() {
    let path = env::current_dir().unwrap();
    process::Command::new(path.join("src/bin/win/packfolder.exe"))
        .args(["src/ui", "app.bin", "-binary"])
        .output()
        .unwrap();
}
fn pack_linux() {
    let path = env::current_dir().unwrap();
    process::Command::new(path.join("src/bin/linux/packfolder"))
        .args(["src/ui", "app.bin", "-binary"])
        .output()
        .unwrap();
}
fn pack_mac() {
    let path = env::current_dir().unwrap();
    process::Command::new(path.join("src/bin/osx/packfolder"))
        .args(["src/ui", "app.bin", "-binary"])
        .output()
        .unwrap();
}
fn main() {
    #[cfg(windows)]
    pack_win();
    #[cfg(linux)]
    pack_linux();
    #[cfg(macos)]
    pack_mac();
}
