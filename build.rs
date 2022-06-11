fn main()
{
if cfg!(target_os = "macos") {
    let out_dir = "/Library/Frameworks";
    println!("cargo:rustc-link-search=framework={}", out_dir);
}
}
