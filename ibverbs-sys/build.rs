extern crate bindgen;

use std::env;
use std::fs::*;
use std::io::*;
use std::path::*;
use std::process::Command;

/// We make additional wrapper functions for existing bindings.
/// To avoid collision, we add a magic prefix for each.
// static PREFIX: &str = "prefix_e163e82e_";

/// Information needed to generate ibverbs binding.
///
/// Each information is filled at different build stages.
#[derive(Debug, Default)]
struct State {
    /// Location of this crate.
    project_path: PathBuf,

    /// Location of generated files.
    out_path: PathBuf,

    /// Essential link path for C standard library.
    include_path: Vec<PathBuf>,

    /// List of ibverbs header files.
    ib_headers: Vec<String>,
}

impl State {
    /// Initialize Cargo envs.
    fn new() -> Self {
        let project_path = PathBuf::from(".").canonicalize().unwrap();
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap())
            .canonicalize()
            .unwrap();
        let mut ret = Self::default();
        ret.project_path = project_path;
        ret.out_path = out_path;
        ret
    }

    fn check_os(&self) {
        #[cfg(not(unix))]
        panic!("Currently, only xnix OS is supported.");
    }

    /// Check compiler and retrieve link path for C standard libs.
    fn check_compiler(&mut self) {
        let output = Command::new("bash")
            .args([
                "-c",
                "cc -march=native -Wp,-v -x c - -fsyntax-only < /dev/null 2>&1 | sed -e '/^#include <...>/,/^End of search/{ //!b };d'",
            ])
            .output()
            .expect("failed to extract cc include path");
        let message = String::from_utf8(output.stdout).unwrap();
        self.include_path.extend(
            message
                .lines()
                .map(|x| String::from(x.trim()))
                .map(PathBuf::from),
        );
    }

    /// Fint verbs.h and find the infiniband/.. directory
    fn find_ibverbs(&mut self) {
        // To find correct lib path of this platform.
        let candiate = vec!["/usr/local/include/infiniband", "/usr/include/infiniband"];
        let sig_file = "ib.h";
        for path_name in candiate {
            let path = PathBuf::from(path_name);
            if path.exists() {
                let file_path = path.join(sig_file);
                if file_path.exists() {
                    println!("cargo:rerun-if-changed={}", path_name);
                    self.include_path.push(path.clone());
                    for entry in path.read_dir().expect("read_dir failed").flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "h" {
                                self.ib_headers
                                    .push(path.file_name().unwrap().to_str().unwrap().to_string());
                            }
                        }
                    }
                    // println!("cargo:warning={:?}", self.ib_headers);
                    return;
                }
            }
        }
        panic!("Cannot find ibverbs headers");
    }

    fn make_all_in_one_header(&mut self) {
        let ibverbs_h = self.out_path.join("ibverbs.h");
        let mut target: File = File::create(ibverbs_h).unwrap();
        let mut template_string = String::new();
        for header_name in &self.ib_headers {
            template_string += &format!("#include <infiniband/{}>\n", header_name);
        }
        target.write_fmt(format_args!("{}", template_string)).ok();
    }

    /// Generate Rust bindings
    fn generate_rust_def(&mut self) {
        let ibverbs_h = self.out_path.join("ibverbs.h");
        let ibverbs_rs = self.out_path.join("ibverbs.rs");
        bindgen::builder()
            .header(ibverbs_h.to_str().unwrap())
            .generate()
            .unwrap()
            .write_to_file(ibverbs_rs)
            .ok();
    }

    /// Do compile.
    fn compile(&mut self) {
        println!("cargo:rustc-link-lib=ibverbs");
    }
}

fn main() {
    let mut state = State::new();
    state.check_os();
    state.check_compiler();
    state.find_ibverbs();
    state.make_all_in_one_header();
    state.generate_rust_def();
    state.compile();
}
