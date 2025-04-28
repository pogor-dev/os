use core::fmt::{self, Write};

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (i, byte) in s.bytes().enumerate() {
            unsafe {
                *VGA_BUFFER.add(i * 2) = byte;
                *VGA_BUFFER.add(i * 2 + 1) = 0x0f; // white on black
            }
        }
        Ok(())
    }
}

pub(crate) fn print_welcome() {
    let mut writer = Writer;
    write!(writer, "Hello from MyOS!").unwrap();
}
