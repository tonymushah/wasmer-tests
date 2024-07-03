extern "C" {
    fn say(something: &str);
}

#[link(wasm_import_module = "utils")]
extern "C" {
    fn panic();
}

#[export_name = "add"]
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[export_name = "run"]
pub fn run() {
    unsafe {
        say("Hello World!");
        say("Yes! it worked")
    };
    unsafe { panic() }
}

#[export_name = "tell"]
pub fn tell(something: &str) {
    let to_say_i = format!("Oh you just told me that `{something}`");
    let to_say: &str = to_say_i.as_ref();
    unsafe { say(to_say) }
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
