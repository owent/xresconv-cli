#![cfg(windows)]

mod support;

#[test]
fn executable_contains_all_application_icon_frames() {
    // Read actual PE resources through Windows, then compare each embedded PNG
    // with the corresponding frame in the shipped ICO. A shell fallback icon
    // cannot satisfy this check. Python is already required by the shim tests.
    let script = r#"
import ctypes as ct
import struct
import sys
from pathlib import Path

kernel = ct.WinDLL('kernel32', use_last_error=True)
kernel.LoadLibraryExW.argtypes = [ct.c_wchar_p, ct.c_void_p, ct.c_uint32]
kernel.LoadLibraryExW.restype = ct.c_void_p
kernel.FindResourceW.argtypes = [ct.c_void_p, ct.c_void_p, ct.c_void_p]
kernel.FindResourceW.restype = ct.c_void_p
kernel.SizeofResource.argtypes = [ct.c_void_p, ct.c_void_p]
kernel.SizeofResource.restype = ct.c_uint32
kernel.LoadResource.argtypes = [ct.c_void_p, ct.c_void_p]
kernel.LoadResource.restype = ct.c_void_p
kernel.LockResource.argtypes = [ct.c_void_p]
kernel.LockResource.restype = ct.c_void_p
kernel.FreeLibrary.argtypes = [ct.c_void_p]
kernel.FreeLibrary.restype = ct.c_int

module = kernel.LoadLibraryExW(sys.argv[1], None, 0x22)
assert module, ct.WinError(ct.get_last_error())
try:
    def resource(identifier, kind):
        entry = kernel.FindResourceW(module, identifier, kind)
        assert entry, ('missing PE resource', identifier, kind, ct.get_last_error())
        size = kernel.SizeofResource(module, entry)
        handle = kernel.LoadResource(module, entry)
        data = kernel.LockResource(handle)
        assert size and data, ct.WinError(ct.get_last_error())
        return ct.string_at(data, size)

    icon = Path(sys.argv[2]).read_bytes()
    group = resource(1, 14)  # RT_GROUP_ICON
    assert struct.unpack_from('<HHH', icon) == (0, 1, 7)
    assert group[:6] == icon[:6]
    dimensions = []
    for index in range(7):
        width, height, colors, reserved, planes, bits, length, offset = struct.unpack_from(
            '<BBBBHHII', icon, 6 + index * 16)
        entry = struct.unpack_from('<BBBBHHIH', group, 6 + index * 14)
        assert entry[:7] == (width, height, colors, reserved, planes, bits, length)
        assert resource(entry[7], 3) == icon[offset:offset + length]  # RT_ICON
        dimensions.append((width or 256, height or 256))
    assert dimensions == [(size, size) for size in (16, 24, 32, 48, 64, 128, 256)]
finally:
    assert kernel.FreeLibrary(module)
"#;
    let output = std::process::Command::new(support::find_python())
        .args(["-c", script, env!("CARGO_BIN_EXE_xresconv-cli")])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/icons/xresconv-cli.ico"
        ))
        .output()
        .expect("Python 3 is required to inspect Windows icon resources");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
