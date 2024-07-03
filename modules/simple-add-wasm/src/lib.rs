extern {
    fn say(something: &str);
}

#[link(wasm_import_module = "utils")]
extern {
    fn panic();
} 

#[export_name = "add"]
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[export_name = "run"]
pub fn run() {
    unsafe { say("Hello World!") };
    panic!("some panic");
    //unsafe { panic() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
