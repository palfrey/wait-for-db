fn main() {
    // Env variables here are only actually used by the tests
    println!("cargo:rerun-if-env-changed=SQLITE_DRIVER");
    println!(
        "cargo:rustc-cfg=sqlite_driver=\"{}\"",
        std::env::var("SQLITE_DRIVER").unwrap_or_else(|_| "".to_string())
    );
    println!("cargo:rerun-if-env-changed=POSTGRES_DRIVER");
    println!(
        "cargo:rustc-cfg=postgres_driver=\"{}\"",
        std::env::var("POSTGRES_DRIVER").unwrap_or_else(|_| "".to_string())
    );
}
