fn main() -> std::io::Result<()> {
    playdate_build::run_simulator(
        env!("PDX_SOURCE_DIR"),
        env!("PDX_OUT_DIR"),
        env!("PDX_NAME"),
    )
}
