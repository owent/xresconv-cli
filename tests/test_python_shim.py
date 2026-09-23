"""Offline launcher contracts; Cargo invokes this suite with its local binary."""
import concurrent.futures
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import tarfile
import tempfile
import unittest
from unittest import mock
import zipfile

spec = importlib.util.spec_from_file_location("shim", Path(__file__).resolve().parents[1] / "xresconv_cli.py")
shim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(shim)


class LauncherTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.cache = Path(self.temp.name)
        patch = mock.patch.object(shim, "cache_dir", return_value=str(self.cache))
        patch.start()
        self.addCleanup(patch.stop)
        patch = mock.patch.dict(os.environ, {}, clear=True)
        patch.start()
        self.addCleanup(patch.stop)
        # No test may accidentally contact a real release or use a personal cache.
        patch = mock.patch.object(shim, "urlopen", side_effect=AssertionError("unexpected network"))
        patch.start()
        self.addCleanup(patch.stop)

    def archive(self, ext, name=None, content=b"binary", link=False):
        data = io.BytesIO()
        name = shim.BIN_NAME if name is None else name
        if ext == ".zip":
            with zipfile.ZipFile(data, "w") as archive:
                member = zipfile.ZipInfo(name)
                if link:
                    member.external_attr = (stat.S_IFLNK | 0o777) << 16
                archive.writestr(member, content)
        else:
            with tarfile.open(fileobj=data, mode="w:gz") as archive:
                member = tarfile.TarInfo(name)
                member.size = len(content)
                if link:
                    member.type = tarfile.SYMTYPE
                    member.linkname = "outside"
                archive.addfile(member, io.BytesIO(content))
        return data.getvalue()

    def test_platform_matrix(self):
        cases = [
            ("Windows", "AMD64", "x86_64-pc-windows-msvc", ".zip"),
            ("Windows", "ARM64", "aarch64-pc-windows-msvc", ".zip"),
            ("Windows", "x86", "i686-pc-windows-msvc", ".zip"),
            ("Linux", "x86_64", "x86_64-unknown-linux-musl", ".tar.gz"),
            ("Linux", "aarch64", "aarch64-unknown-linux-musl", ".tar.gz"),
            ("Linux", "armv7l", "armv7-unknown-linux-gnueabihf", ".tar.gz"),
            ("Linux", "riscv64", "riscv64gc-unknown-linux-gnu", ".tar.gz"),
            ("Linux", "loongarch64", "loongarch64-unknown-linux-gnu", ".tar.gz"),
            ("Darwin", "arm64", "aarch64-apple-darwin", ".tar.gz"),
            ("Darwin", "x86_64", "x86_64-apple-darwin", ".tar.gz"),
            ("FreeBSD", "amd64", "x86_64-unknown-freebsd", ".tar.gz"),
        ]
        for system, machine, target, ext in cases:
            with self.subTest(system=system, machine=machine), mock.patch.object(shim.platform, "system", return_value=system), mock.patch.object(shim.platform, "machine", return_value=machine):
                self.assertEqual(shim.detect_asset(), (target, ext))

    def test_unsupported_platform(self):
        with mock.patch.object(shim.platform, "machine", return_value="unknown"):
            self.assertIsNone(shim.detect_asset())
            with self.assertRaisesRegex(RuntimeError, "no prebuilt binary"):
                shim.download_latest()

    def test_existing_env_path_wins(self):
        path = self.cache / "local binary 中文"
        path.write_bytes(b"local")
        with mock.patch.dict(os.environ, {shim.BIN_ENV: str(path)}):
            self.assertEqual(shim.resolve_binary(), str(path))

    def test_missing_env_path_falls_back_to_download_then_cache(self):
        data = self.archive(".zip")
        with mock.patch.dict(os.environ, {shim.BIN_ENV: "not-present"}), mock.patch.object(shim, "download_latest", return_value=(data, ".zip")) as download:
            first = shim.resolve_binary()
            self.assertEqual(Path(first).read_bytes(), b"binary")
            self.assertEqual(shim.resolve_binary(), first)
            download.assert_called_once_with()

    def test_download_failure_is_diagnostic(self):
        with mock.patch.object(shim, "download_latest", side_effect=RuntimeError("network down")), mock.patch.object(shim.sys, "stderr", new_callable=io.StringIO) as stderr:
            self.assertIsNone(shim.resolve_binary())
            self.assertIn("network down", stderr.getvalue())
            self.assertIn(shim.BIN_ENV, stderr.getvalue())

    def release(self, checksum=None, missing=None, url=None):
        name = "xresconv-cli-2.0.0-x86_64-unknown-linux-musl.tar.gz"
        asset_url = "https://github.com/xresloader/xresconv-cli/releases/download/v2.0.0/" + name
        data = b"archive"
        meta = {"tag_name": "v2.0.0", "assets": [
            {"name": name, "browser_download_url": url or asset_url},
            {"name": name + ".sha256", "browser_download_url": asset_url + ".sha256"},
        ]}
        if missing is not None:
            meta["assets"].pop(missing)
        responses = {shim.RELEASES_API: json.dumps(meta).encode(), asset_url: data,
                     asset_url + ".sha256": checksum if checksum is not None else (hashlib.sha256(data).hexdigest() + "  " + name).encode()}
        return mock.patch.object(shim, "http_get", side_effect=lambda url, *args: responses[url])

    @mock.patch.object(shim, "detect_asset", return_value=("x86_64-unknown-linux-musl", ".tar.gz"))
    def test_release_and_checksum(self, _):
        with self.release():
            self.assertEqual(shim.download_latest(), (b"archive", ".tar.gz"))
        for checksum in [b"0" * 64, b"", b"invalid", b"g" * 64]:
            with self.subTest(checksum=checksum), self.release(checksum=checksum), self.assertRaises(RuntimeError):
                shim.download_latest()

    @mock.patch.object(shim, "detect_asset", return_value=("x86_64-unknown-linux-musl", ".tar.gz"))
    def test_missing_release_assets_and_foreign_url(self, _):
        for args in [{"missing": 0}, {"missing": 1}, {"url": "https://example.com/binary"}]:
            with self.subTest(args=args), self.release(**args), self.assertRaises(RuntimeError):
                shim.download_latest()

    def test_http_timeout_and_size_limit(self):
        with mock.patch.object(shim, "urlopen", return_value=io.BytesIO(b"data")) as request:
            self.assertEqual(shim.http_get(shim.RELEASES_API), b"data")
            self.assertEqual(request.call_args.kwargs["timeout"], 30)
        with mock.patch.object(shim, "urlopen", return_value=io.BytesIO(b"too large")), self.assertRaisesRegex(RuntimeError, "size limit"):
            shim.http_get(shim.RELEASES_API, 3)

    def test_zip_and_tar_atomic_install(self):
        for ext in [".zip", ".tar.gz"]:
            with self.subTest(ext=ext):
                path = shim.install_binary(self.archive(ext, "./" + shim.BIN_NAME), ext)
                self.assertEqual(Path(path).read_bytes(), b"binary")
                if os.name != "nt":
                    self.assertTrue(os.access(path, os.X_OK))
                self.assertEqual(list((self.cache / "bin").iterdir()), [Path(path)])

    def test_invalid_archives_preserve_existing_cache(self):
        path = Path(shim.install_binary(self.archive(".zip"), ".zip"))
        for ext in [".zip", ".tar.gz"]:
            for kwargs in [{"name": "../" + shim.BIN_NAME}, {"name": "missing"}, {"link": True}, {"content": b""}]:
                with self.subTest(ext=ext, kwargs=kwargs), self.assertRaises(RuntimeError):
                    shim.install_binary(self.archive(ext, **kwargs), ext)
                self.assertEqual(path.read_bytes(), b"binary")
        self.assertEqual(list(path.parent.iterdir()), [path])
        with self.assertRaises(Exception):
            shim.install_binary(b"corrupt", ".zip")
        self.assertEqual(list(path.parent.iterdir()), [path])

    def test_simultaneous_install_is_complete(self):
        data = self.archive(".zip", content=b"complete" * 4096)
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            paths = list(pool.map(lambda _: shim.install_binary(data, ".zip"), range(8)))
        self.assertEqual(len(set(paths)), 1)
        self.assertEqual(Path(paths[0]).read_bytes(), b"complete" * 4096)
        self.assertEqual(len(list(Path(paths[0]).parent.iterdir())), 1)

    def test_argument_and_exit_forwarding(self):
        args = ["launcher", "列表 中文.xml", "--", "--output", "a b", "literal\"quote"]
        with mock.patch.object(shim, "resolve_binary", return_value="binary"), mock.patch.object(shim.sys, "argv", args), mock.patch.object(shim.subprocess, "call", return_value=7) as run:
            self.assertEqual(shim.main(), 7)
            run.assert_called_once_with(["binary"] + args[1:])

    def test_execution_failure_and_interrupt(self):
        with mock.patch.object(shim, "resolve_binary", return_value="binary"), mock.patch.object(shim.subprocess, "call", side_effect=OSError("denied")):
            self.assertEqual(shim.main(), 1)
        with mock.patch.object(shim, "resolve_binary", return_value="binary"), mock.patch.object(shim.subprocess, "call", side_effect=KeyboardInterrupt):
            self.assertEqual(shim.main(), 130)


if __name__ == "__main__":
    unittest.main()
