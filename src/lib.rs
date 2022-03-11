extern "C" {
    fn hello();
}

#[no_mangle]
fn link() {
    unsafe { hello() };
}