#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""xresconv-cli 兼容入口：提示升级并转交 Rust 可执行文件。

二进制解析顺序：
1. 环境变量 XRESCONV_CLI_BIN 指定的可执行文件路径
2. 缓存目录中已下载的二进制
3. 从 GitHub Releases 下载最新版本并写入缓存

Rust 版本发布页（含各平台预编译二进制）：
https://github.com/xresloader/xresconv-cli/releases
"""

from __future__ import unicode_literals

import hashlib
import io
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from contextlib import closing
try:
    from urllib.request import Request, urlopen
    from urllib.parse import urlparse
except ImportError:  # Preserve the legacy Python 2.7 launcher as well.
    from urllib2 import Request, urlopen
    from urlparse import urlparse

UPGRADE_MESSAGE = (
    "[DEPRECATED] xresconv-cli 已用 Rust 重写，Python 版本不再维护，本入口仅做兼容转发。\n"
    "[DEPRECATED] xresconv-cli has been rewritten in Rust; this Python entry only forwards to the Rust binary.\n"
    "建议直接使用预编译可执行文件 / Please use the prebuilt binary directly:\n"
    "  https://github.com/xresloader/xresconv-cli/releases\n"
)

BIN_ENV = "XRESCONV_CLI_BIN"
RELEASES_API = "https://api.github.com/repos/xresloader/xresconv-cli/releases/latest"
RELEASES_PAGE = "https://github.com/xresloader/xresconv-cli/releases"
BIN_NAME = "xresconv-cli.exe" if os.name == "nt" else "xresconv-cli"


def write_stderr(text):
    try:
        sys.stderr.write(text)
    except UnicodeEncodeError:
        encoding = getattr(sys.stderr, "encoding", None) or "ascii"
        sys.stderr.write(text.encode(encoding, "backslashreplace").decode(encoding))


def detect_asset():
    system = platform.system().lower()
    arch = {
        "x86_64": "x86_64",
        "amd64": "x86_64",
        "arm64": "aarch64",
        "aarch64": "aarch64",
        "riscv64": "riscv64gc",
    }.get(platform.machine().lower())
    if not arch:
        return None
    if system == "windows" and arch in ("x86_64", "aarch64"):
        return arch + "-pc-windows-msvc", ".zip"
    if system == "linux":
        if hasattr(sys, "getandroidapilevel"):
            return ("aarch64-linux-android", ".tar.gz") if arch == "aarch64" else None
        if arch in ("x86_64", "aarch64"):
            return arch + "-unknown-linux-musl", ".tar.gz"
        if arch == "riscv64gc":
            return "riscv64gc-unknown-linux-gnu", ".tar.gz"
        return None
    if system == "darwin" and arch in ("x86_64", "aarch64"):
        return arch + "-apple-darwin", ".tar.gz"
    if system == "freebsd" and arch == "x86_64":
        return "x86_64-unknown-freebsd", ".tar.gz"
    return None


def cache_dir():
    if os.name == "nt":
        base = os.environ.get("LOCALAPPDATA") or os.path.expanduser("~")
        return os.path.join(base, "xresconv-cli")
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
    return os.path.join(base, "xresconv-cli")


def cached_binary():
    path = os.path.join(cache_dir(), "bin", BIN_NAME)
    return path if os.path.isfile(path) and (os.name == "nt" or os.access(path, os.X_OK)) else None


def http_get(url, max_size=128 * 1024 * 1024):
    req = Request(url, headers={"User-Agent": "xresconv-cli-python-shim"})
    with closing(urlopen(req, timeout=30)) as resp:
        data = resp.read(max_size + 1)
        if len(data) > max_size:
            raise RuntimeError("download exceeds size limit")
        return data


def release_url(asset):
    url = asset.get("browser_download_url", "")
    parsed = urlparse(url)
    if (parsed.scheme != "https" or parsed.netloc != "github.com"
            or not parsed.path.startswith("/xresloader/xresconv-cli/releases/download/")):
        raise RuntimeError("invalid GitHub release asset URL")
    return url


def download_latest():
    found = detect_asset()
    if not found:
        raise RuntimeError(
            "当前平台无预编译二进制 no prebuilt binary for this platform: %s/%s"
            % (platform.system(), platform.machine())
        )
    target, ext = found
    meta = json.loads(http_get(RELEASES_API, 2 * 1024 * 1024).decode("utf-8"))
    tag = meta.get("tag_name", "")
    name = "xresconv-cli-%s-%s%s" % (tag.lstrip("v"), target, ext)
    assets = {asset.get("name"): asset for asset in meta.get("assets", [])}
    if name not in assets:
        raise RuntimeError(
            "版本 %s 中未找到平台 %s 的制品 no asset for %s in release %s" % (tag, target, target, tag)
        )
    if name + ".sha256" not in assets:
        raise RuntimeError("release checksum asset missing: " + name)
    asset_url = release_url(assets[name])
    data = http_get(asset_url)
    checksum = http_get(release_url(assets[name + ".sha256"]), 8192).decode("ascii").split()
    if not checksum or len(checksum[0]) != 64 or any(c not in "0123456789abcdefABCDEF" for c in checksum[0]):
        raise RuntimeError("invalid sha256 checksum")
    expected = checksum[0].lower()
    actual = hashlib.sha256(data).hexdigest()
    if expected != actual:
        raise RuntimeError("sha256 校验失败 checksum mismatch: %s" % asset_url)
    return data, ext


def install_binary(archive_data, ext):
    out_dir = os.path.join(cache_dir(), "bin")
    try:
        os.makedirs(out_dir)
    except OSError:
        if not os.path.isdir(out_dir):
            raise
    out_path = os.path.join(out_dir, BIN_NAME)
    fd, staged = tempfile.mkstemp(prefix=".download-", dir=out_dir)
    try:
        with os.fdopen(fd, "wb") as dst:
            if ext == ".zip":
                with zipfile.ZipFile(io.BytesIO(archive_data)) as archive:
                    members = [m for m in archive.infolist() if m.filename in (BIN_NAME, "./" + BIN_NAME)]
                    if len(members) != 1 or (members[0].external_attr >> 16) & 0o170000 == 0o120000:
                        raise RuntimeError("archive must contain one regular binary")
                    if not 0 < members[0].file_size <= 128 * 1024 * 1024:
                        raise RuntimeError("invalid binary size")
                    with archive.open(members[0]) as src:
                        shutil.copyfileobj(src, dst)
            elif ext == ".tar.gz":
                with tarfile.open(fileobj=io.BytesIO(archive_data), mode="r:gz") as archive:
                    members = [m for m in archive.getmembers() if m.name in (BIN_NAME, "./" + BIN_NAME)]
                    if len(members) != 1 or not members[0].isreg():
                        raise RuntimeError("archive must contain one regular binary")
                    if not 0 < members[0].size <= 128 * 1024 * 1024:
                        raise RuntimeError("invalid binary size")
                    with closing(archive.extractfile(members[0])) as src:
                        shutil.copyfileobj(src, dst)
            else:
                raise RuntimeError("unsupported archive type: " + ext)
            dst.flush()
            os.fsync(dst.fileno())
        if os.name != "nt":
            os.chmod(staged, 0o755)
        try:
            getattr(os, "replace", os.rename)(staged, out_path)
        except OSError:
            # Another launcher may have installed it while we were downloading;
            # Windows also refuses to replace a currently executing binary.
            if not cached_binary():
                raise
    finally:
        if os.path.exists(staged):
            os.unlink(staged)
    return out_path


def resolve_binary():
    env_path = os.environ.get(BIN_ENV)
    if env_path:
        env_path = os.path.abspath(os.path.expanduser(env_path))
        if os.path.isfile(env_path):
            return env_path
        write_stderr("[WARNING] %s not found: %s; trying cache and GitHub Releases\n" % (BIN_ENV, env_path))
    cached = cached_binary()
    if cached:
        return cached
    write_stderr(
        "未找到 xresconv-cli 二进制，正在从 GitHub Releases 下载最新版本...\n"
        "xresconv-cli binary not found, downloading the latest release from GitHub...\n"
    )
    try:
        data, ext = download_latest()
        return install_binary(data, ext)
    except Exception as e:
        write_stderr("[ERROR] 下载失败 download failed: %s\n" % e)
        write_stderr(
            "请手动下载并用 %s 指定路径 / download manually and set %s:\n  %s\n" % (BIN_ENV, BIN_ENV, RELEASES_PAGE)
        )
        return None


def main():
    write_stderr(UPGRADE_MESSAGE)
    sys.stderr.flush()
    binary = resolve_binary()
    if not binary:
        return 1
    try:
        code = subprocess.call([binary] + sys.argv[1:])
        # POSIX subprocess uses negative values for signals; shell status is 128+signal.
        return 128 - code if os.name != "nt" and code < 0 else code
    except KeyboardInterrupt:
        return 130
    except OSError as error:
        write_stderr("[ERROR] cannot execute %s: %s\n" % (binary, error))
        return 1


if __name__ == "__main__":
    exit(main())
