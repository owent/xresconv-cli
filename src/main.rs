fn main() {
    let code = xresconv_cli::run();
    // Normal return lets the C runtime flush its exit hooks on Windows. Keep
    // process::exit for legacy signed Windows error codes (-1/-2/-4).
    if code != 0 {
        std::process::exit(code);
    }
}
