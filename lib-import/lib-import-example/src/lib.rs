#[no_mangle]
pub extern "C" fn __guest_call(key: i32) -> u32 {
    // println can't work in wasm
    println!("call from guest (web assemply)");
    println!("passing key is {}", key);
    unsafe { __passing_key(key as u32) }
    println!("end of call from guest (web assemply)");

    key as u32 * 2
}

// note import module should have no returns
#[link(wasm_import_module = "wapc")]
extern "C" {
    // pub fn __host_call();

    pub fn __passing_key(key: u32);
}
