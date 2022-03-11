fn pdx_build_dir<P>(out_dir: P) -> std::path::PathBuf
where
    std::path::PathBuf: From<P>,
{
    std::path::PathBuf::from(out_dir).join("pdx_build")
}

pub fn make_pdx_build_dir(out_dir: &str) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(pdx_build_dir(out_dir))
}

fn touch_pdx_bin(out_dir: &str) -> Result<(), std::io::Error> {
    std::fs::write(pdx_build_dir(out_dir), "")
}
