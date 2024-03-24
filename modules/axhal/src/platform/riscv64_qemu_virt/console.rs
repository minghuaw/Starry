/// Writes a byte to the console.
pub fn putchar(c: u8) {
    hal::console_putchar(c);
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    hal::console_getchar()
}
