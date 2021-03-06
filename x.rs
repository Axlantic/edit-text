#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! commandspec = "0.8"
//! failure = "0.1"
//! structopt = "0.2"
//! ```

// Don't add additional noise to cargo-script.
#![deny(warnings)]

#[macro_use]
extern crate commandspec;
#[macro_use]
extern crate structopt;
extern crate failure;

use commandspec::*;
use std::path::Path;
use std::env;
use failure::Error;
use structopt::StructOpt;

fn abs_string_path<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    Ok(Path::new(".")
        .canonicalize()?
        .join(path)
        .to_string_lossy()
        .into_owned())
}

// Thin wrapper around run()
fn main() {
    commandspec::cleanup_on_ctrlc();

    match run() {
        Ok(_) => {},
        Err(ref err) => {
            eprintln!("Error: {}", err);
            ::std::process::exit(1);
        }
    }
}

/// edit-text build scripts
#[derive(StructOpt)]
#[structopt(name = "edit-text build scripts", about = "Build scripts for edit and oatie", author = "")]
enum Cli {
    #[structopt(name = "wasm-build", about = "Compile the WebAssembly bundle.")]
    Wasm {
        #[structopt(name = "no-vendor")]
        no_vendor: bool,
    },

    #[structopt(name = "client-proxy", about = "Run client code in your terminal.")]
    ClientProxy { args: Vec<String> },

    #[structopt(name = "client-proxy-build", about = "Build the client proxy.")]
    ClientProxyBuild { args: Vec<String> },

    #[structopt(name = "oatie-build", about = "Build the operational transform library.")]
    OatieBuild { args: Vec<String> },

    #[structopt(name = "server", about = "Run the edit-text server.")]
    MercutioServerRun {
        #[structopt(long = "log", help = "Export a log")]
        log: bool,
        args: Vec<String>,
    },

    #[structopt(name = "server-build", about = "Build the edit-text server.")]
    MercutioServerBuild { args: Vec<String> },

    #[structopt(name = "replay", about = "Replay an edit-text log.")]
    Replay { args: Vec<String> },

    #[structopt(name = "test")]
    Test { args: Vec<String> },

    #[structopt(name = "build")]
    Build { args: Vec<String> },

    #[structopt(name = "frontend-build", about = "Bundle the frontend JavaScript code.")]
    FrontendBuild { args: Vec<String> },

    #[structopt(name = "frontend-watch", about = "Watch the frontend JavaScript code, building continuously.")]
    FrontendWatch { args: Vec<String> },

    #[structopt(name = "deploy", about = "Deploy to sandbox.edit.io.")]
    Deploy,

    #[structopt(name = "book-build", about = "Builds the book.")]
    BookBuild,

    #[structopt(name = "book-watch", about = "Watches and rebuilds the book.")]
    BookWatch,
}


fn run() -> Result<(), Error> {
    // Pass arguments directly to subcommands: don't capture -h, -v, or verification
    // Do this by adding "--" into the args flag after the subcommand.
    let mut args = ::std::env::args().collect::<Vec<_>>();
    if args.len() > 2 && args[1] != "help" {
        args.insert(2, "--".into());
    }

    // We interpret the --release flag at the build level.
    let release = args.iter().find(|x| *x == "--release").is_some();
    args = args.into_iter().filter(|x| *x != "--release").collect();

    // Respect the CLICOLOR env variable.
    let force_color = ::std::env::var("CLICOLOR").map(|x| x == "1").unwrap_or(false);
    let force_color_flag = if force_color { Some("--color=always") } else { None };

    // Run the subcommand.
    let parsed_args = Cli::from_iter(args.iter());
    match parsed_args {
        Cli::Wasm { no_vendor } => {
            // wasm must always be --release
            let release_flag = Some("--release");

            eprintln!("Building...");

            execute!(
                r"
                    cd edit-client
                    cargo check {release_flag} --lib --target wasm32-unknown-unknown
                ",
                release_flag = release_flag,
            )?;

            execute!(
                r"
                    cd edit-client
                    cargo build {release_flag} --lib --target wasm32-unknown-unknown
                ",
                release_flag = release_flag,
            )?;

            if !no_vendor {
                eprintln!("Vendoring...");

                ::std::fs::create_dir_all("./edit-frontend/src/bindgen")?;

                execute!(
                    r"
                        wasm-bindgen ./target/wasm32-unknown-unknown/release/edit_client.wasm \
                            --out-dir ./edit-frontend/src/bindgen \
                            --typescript
                    ",
                )?;

                execute!(
                    r"
                        cd ./edit-frontend/src/bindgen
                        wasm2es6js \
                            --base64 -o edit_client_bg.js edit_client_bg.wasm
                    ",
                )?;

                ::std::fs::remove_file("./edit-frontend/src/bindgen/edit_client_bg.wasm")?;

                eprintln!("Done.");
            }
        }

        Cli::ClientProxy { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-client
                    export MERCUTIO_WASM_LOG=0
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin edit-client-proxy -- {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::ClientProxyBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-client
                    export MERCUTIO_WASM_LOG=0
                    export RUST_BACKTRACE=1
                    cargo build {release_flag} --bin edit-client-proxy -- {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::OatieBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd oatie
                    cargo build {release_flag} {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::MercutioServerRun { log, args } => {
            if release {
                eprintln!("Building and running edit-text server (release mode)...");
            } else {
                eprintln!("Building and running edit-text server (debug mode)...");
            }

            if !Path::new("edit-server/edit.sqlite3").exists() {
                eprintln!("Building database on first startup...");
                execute!(
                    r"
                        cd edit-server
                        diesel setup
                    ",
                )?;
            } else {
                println!("Database path: edit.sqlite3");
            }

            // if !Path::new("edit-frontend/dist/edit.wasm").exists() {
                // execute!(
                //     r"
                //         ./x.rs wasm-build
                //     "
                // )?;
                // execute!(
                //     r"
                //         ./x.rs frontend-build
                //     ",
                // )?;
            // }
            // Don't print anything if it existed, because we might not have
            // launched in --client-proxy mode.
            // TODO if we can reliably check for --client-proxy or -c, we should
            // not build on first launch.

            eprintln!("Starting server...");

            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-server
                    export MERCUTIO_WASM_LOG={use_log}
                    export RUST_BACKTRACE=1
                    export DATABASE_URL=edit-server/edit.sqlite3
                    cargo run {force_color_flag} {release_flag} \
                        --bin edit-server -- \
                        --period 100 {args}
                ",
                use_log = if log { 1 } else { 0 },
                release_flag = release_flag,
                force_color_flag = force_color_flag,
                args = args,
            )?;

            eprintln!("Server exited.");
        }

        Cli::MercutioServerBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-server
                    export MERCUTIO_WASM_LOG=0
                    export RUST_BACKTRACE=1
                    cargo build {force_color_flag} {release_flag} \
                        --bin edit-server {args}
                ",
                release_flag = release_flag,
                force_color_flag = force_color_flag,
                args = args,
            )?;
        }

        Cli::Replay { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin edit-replay -- {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::Test { args } => {
            eprintln!("building ./x.rs server...");
            execute!(
                r"
                    ./x.rs server-build
                ",
            )?;

            eprintln!("running ./x.rs server...");
            let _server_guard = command!(
                r"
                    ./x.rs server {args}
                ",
                args = args,
            )?.scoped_spawn().unwrap();

            ::std::thread::sleep(::std::time::Duration::from_millis(3000));

            eprintln!("running tests...");
            execute!(
                r"
                    cd tests
                    cargo run
                ",
            )?;
        }

        Cli::Build { args } => {
            eprintln!("[wasm-build]");
            execute!(
                r"
                    ./x.rs wasm-build {args}
                ",
                args = args,
            )?;

            eprintln!("");
            eprintln!("[frontend-build]");
            execute!(
                r"
                    ./x.rs frontend-build {args}
                ",
                args = args,
            )?;

            eprintln!("");
            eprintln!("[server-build]");
            execute!(
                r"
                    ./x.rs server-build {args}
                ",
                args = args,
            )?;

            eprintln!("");
            eprintln!("[client-proxy-build]");
            execute!(
                r"
                    ./x.rs client-proxy-build {args}
                ",
                args = args,
            )?;

            eprintln!("");
            eprintln!("[book-build]");
            execute!(
                r"
                    ./x.rs book-build {args}
                ",
                args = args,
            )?;
        }

        Cli::FrontendBuild { args } => {
            execute!(
                r"
                    cd edit-frontend
                    ./node_modules/.bin/webpack \
                        ./src/index.js --mode development --output-filename='edit.js' {args}
                ",
                args = args,
            )?;
        }

        Cli::FrontendWatch { args } => {
            execute!(
                r"
                    cd edit-frontend
                    ./node_modules/.bin/webpack --watch \
                        ./src/index.js --mode development --output-filename='edit.js' {args}
                ",
                args = args,
            )?;
        }

        Cli::Deploy => {
            let dokku_url = env::var("EDIT_DEPLOY_URL").unwrap_or("sandbox.edit.io".to_string());
            let dokku_name = env::var("EDIT_DOKKU_NAME").unwrap_or("edit-text".to_string());

            // WASM client code
            eprintln!();
            eprintln!("Compiling WebAssembly...");
            execute!(
                "
                    rustup target add wasm32-unknown-unknown
                "
            )?;
            execute!(
                "
                    ./x.rs wasm-build
                "
            )?;

            // Frontend JavaScript
            eprintln!("Building frontend...");
            execute!(
                r"
                    ./x.rs frontend-build
                "
            )?;

            // Linux binary
            eprintln!();
            eprintln!("Building Linux server binary...");
            execute!(
                "
                    docker build \
                        -f dist/build/Dockerfile dist/build/ \
                        -t edit-build-server
                "
            )?;
            eprintln!();
            eprintln!("Running local cargo build...");
            execute!(
                r"
                    docker run --rm \
                        -v {dir_git}:/usr/local/cargo/git \
                        -v {dir_registry}:/usr/local/cargo/registry \
                        -v {dir_rustup}:/usr/local/rustup/toolchains \
                        -v {dir_self}:/app \
                        -w /app/edit-server \
                        -t edit-build-server \
                        cargo build --release --target=x86_64-unknown-linux-gnu --bin edit-server --features 'standalone'
                ",
                dir_git = abs_string_path("dist/build/cargo-git-cache")?,
                dir_registry = abs_string_path("dist/build/cargo-registry-cache")?,
                dir_rustup = abs_string_path("dist/build/rustup-toolchain-cache")?,
                dir_self = abs_string_path(".")?,
            )?;
            eprintln!();
            eprintln!("Copying directories...");
            execute!(
                "
                    cp target/x86_64-unknown-linux-gnu/release/edit-server dist/deploy
                "
            )?;

            // Shell out for uploading the file to dokku.
            eprintln!();
            eprintln!("Uploading...");
            shell_sh!(
                r#"
                    cd dist/deploy

                    # Doing these two commands as one pipe may cause dokku to hang
                    # (from experience) so first, upload the tarball, then load it.
                    tar c . | bzip2 | ssh root@{dokku_url} "bunzip2 > /tmp/edit.tar"
                    ssh root@{dokku_url} 'cat /tmp/edit.tar | dokku tar:in {dokku_name}'
                "#,
                dokku_url = dokku_url,
                dokku_name = dokku_name,
            )?;
        }

        Cli::BookBuild => {
            execute!(
                r"
                    cd docs/src
                    mdbook build
                ",
            )?;
        }

        Cli::BookWatch => {
            execute!(
                r"
                    cd docs/src
                    mdbook serve
                ",
            )?;
        }
    }

    Ok(())
}
