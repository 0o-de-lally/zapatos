use std::process::Command;
use std::path::Path;

fn main() {

    if cfg!(target_os = "windows") {
        let rustup_output = Command::new("rustup")
            .arg("which")
            .arg("rustc")
            .output()
            .expect("Couldn't get rustup output.");
        print!("{:?}", rustup_output);
        let rustc_path = String::from_utf8(rustup_output.stdout).expect("Couldn't get toolchain path");
        let toolchain_path = Path::new(&rustc_path)
            .parent().unwrap()
            .parent().unwrap();
        print!("{:?}", toolchain_path);

        let output = Command::new("rustc")
            .arg("-vV")
            .output()
            .expect("Failed to run rustc to get the host target");
        let output = String::from_utf8(output.stdout).expect("`rustc -vV` didn't return utf8 output");

        let field = "host: ";
        let toolchain_triple = output
            .lines()
            .find(|l| l.starts_with(field))
            .map(|l| &l[field.len()..]).unwrap().to_string();

        print!("{:?}", toolchain_triple);
        let architecture = if let Some(_) = toolchain_triple.find("x86_64") {
            "x86_64"
        } else {
            "x86"
        };

        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(".github").join("redist").join(architecture);
        let dll_path = source_path.join("gmp.dll");
        let lib_path = source_path.join("gmp.lib");
        let target_path = toolchain_path
            .join("lib")
            .join("rustlib")
            .join(toolchain_triple)
            .join("lib");
        print!("{:?}", dll_path);
        print!("{:?}", target_path);
        std::fs::copy(dll_path, target_path.join("gmp.dll")).expect("Couldn't copy dll");
        std::fs::copy(lib_path, target_path.join("gmp.lib")).expect("Couldn't copy lib");
    };

}
