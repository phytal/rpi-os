use crate::kprintln;
use core::panic::PanicInfo;

const MESSSAGE: &str = r#"
            (
       (      )     )
         )   (    (
        (          `
    .-""^"""^""^"""^""-.
  (//\\//\\//\\//\\//\\//)
   ~\^^^^^^^^^^^^^^^^^^/~
     `================`

    The pi is overdone.

---------- PANIC ----------
"#;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("{}", MESSSAGE);
    if let Some(location) = _info.location() {
        kprintln!(
            "FILE: {}\nLINE: {}\nCOL: {}\n\n{}",
            location.file(),
            location.line(),
            location.column(),
            _info.message()
        );
    }
    loop {}
}
