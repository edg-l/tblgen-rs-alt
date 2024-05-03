use std::{
    env,
    error::Error,
    path::Path,
    process::{exit, Command},
    str,
};

const LLVM_MAJOR_VERSION: usize = if cfg!(feature = "llvm16-0") {
    16
} else if cfg!(feature = "llvm17-0") {
    17
} else {
    18
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let version = llvm_config("--version")?;

    if !version.starts_with(&format!("{}.", LLVM_MAJOR_VERSION)) {
        return Err(format!(
            "failed to find correct version ({}.x.x) of llvm-config (found {})",
            LLVM_MAJOR_VERSION, version
        )
        .into());
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=cc");
    println!("cargo:rustc-link-search={}", llvm_config("--libdir")?);
    println!("cargo:rustc-link-lib=static=LLVMCore");
    println!("cargo:rustc-link-lib=static=LLVMSupport");
    println!("cargo:rustc-link-lib=static=LLVMTableGen");

    for name in llvm_config("--libnames")?.trim().split(' ') {
        if let Some(name) = trim_library_name(name) {
            println!("cargo:rustc-link-lib={}", name);
        }
    }

    for flag in llvm_config("--system-libs")?.trim().split(' ') {
        let flag = flag.trim_start_matches("-l");

        if flag.starts_with('/') {
            // llvm-config returns absolute paths for dynamically linked libraries.
            let path = Path::new(flag);

            println!(
                "cargo:rustc-link-search={}",
                path.parent().unwrap().display()
            );
            println!(
                "cargo:rustc-link-lib={}",
                trim_library_name(path.file_name().unwrap().to_str().unwrap()).unwrap()
            );
        } else {
            println!("cargo:rustc-link-lib={}", flag);
        }
    }

    if let Some(name) = get_system_libcpp() {
        println!("cargo:rustc-link-lib={}", name);
    }

    std::env::set_var("CXXFLAGS", llvm_config("--cxxflags")?);
    std::env::set_var("CFLAGS", llvm_config("--cflags")?);
    println!("cargo:rustc-link-search={}", &env::var("OUT_DIR")?);

    cc::Build::new()
        .files(
            std::fs::read_dir("cc/lib")?
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap().path())
                .filter(|r| r.is_file() && r.extension().unwrap() == "cpp"),
        )
        .cpp(true)
        .include("cc/include")
        .include(llvm_config("--includedir")?)
        // .flag("-MJcompile_commands.o.json")
        .opt_level(3)
        .compile("CTableGen");

    println!("cargo:rustc-link-lib=static=CTableGen");

    bindgen::builder()
        .header("wrapper.h")
        .clang_arg("-Icc/include")
        .clang_arg(format!("-I{}", llvm_config("--includedir")?))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(Path::new(&env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}

fn get_system_libcpp() -> Option<&'static str> {
    if cfg!(target_env = "msvc") {
        None
    } else if cfg!(target_os = "macos") {
        Some("c++")
    } else {
        Some("stdc++")
    }
}

fn llvm_config(argument: &str) -> Result<String, Box<dyn Error>> {
    let prefix = env::var(format!("TABLEGEN_{}0_PREFIX", LLVM_MAJOR_VERSION))
        .map(|path| Path::new(&path).join("bin"))
        .unwrap_or_default();
    let call = format!(
        "{} --link-static {}",
        prefix.join("llvm-config").display(),
        argument
    );

    Ok(str::from_utf8(
        &if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &call]).output()?
        } else {
            Command::new("sh").arg("-c").arg(&call).output()?
        }
        .stdout,
    )?
    .trim()
    .to_string())
}

fn trim_library_name(name: &str) -> Option<&str> {
    name.strip_prefix("lib")
        .and_then(|name| name.split('.').next())
}
