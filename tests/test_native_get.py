"""extras/pulse-native-get: what it installs, and what it refuses (a fake release on disk, through file:// URLs)."""
import json
import os
import platform
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path

GET = Path(__file__).resolve().parent.parent / "extras" / "pulse-native-get"


@unittest.skipUnless(platform.machine() == "x86_64" and shutil.which("ssh-keygen") and shutil.which("curl")
                     and shutil.which("sha256sum"), "needs x86_64, ssh-keygen, curl and sha256sum")
class NativeGetTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        self.home = self.tmp / "home"
        self.share = self.home / ".local/share/omarchy-pulse"
        self.share.mkdir(parents=True)
        self.exe(self.share / "omarchy-pulse", "#!/bin/sh\nexit 0\n")
        self.marker = self.tmp / "installed"
        self.dl = self.tmp / "dl"
        subprocess.run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(self.tmp / "key")], check=True)
        subprocess.run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(self.tmp / "other")], check=True)

    def exe(self, path, text):
        path.write_text(text)
        path.chmod(0o755)

    def release(self, ver, signed_with=None):
        """A release as the CI makes it: the archive and SHA256SUMS, and SHA256SUMS.sig when someone signed it."""
        d = self.dl / f"pulse-native-v{ver}"
        src = self.tmp / f"src-{ver}" / "pulse-native"
        src.mkdir(parents=True)
        self.exe(src / "pulse-native", f"#!/bin/sh\necho pulse-native {ver}\n")
        self.exe(src / "install-native", f"#!/bin/sh\necho {ver} > '{self.marker}'\n")
        d.mkdir(parents=True)
        with tarfile.open(d / "pulse-native-x86_64.tar.gz", "w:gz") as t:
            t.add(src, arcname="pulse-native")
        sums = subprocess.run(["sha256sum", "pulse-native-x86_64.tar.gz"], cwd=d, capture_output=True, text=True,
                              check=True).stdout
        (d / "SHA256SUMS").write_text(sums)
        if signed_with:
            subprocess.run(["ssh-keygen", "-q", "-Y", "sign", "-f", str(self.tmp / signed_with), "-n", "file",
                            str(d / "SHA256SUMS")], check=True, capture_output=True)
        return d

    def know_key(self, name="key"):
        pub = (self.tmp / f"{name}.pub").read_text().split()
        (self.share / "native-allowed-signers").write_text(f'pulse-native namespaces="file" {pub[0]} {pub[1]}\n')

    def api(self, *releases):
        """The GitHub releases list, newest first: (version, has a signature)."""
        rows = [{"tag_name": f"pulse-native-v{v}", "draft": False, "prerelease": False,
                 "assets": [{"name": "pulse-native-x86_64.tar.gz"}, {"name": "SHA256SUMS"}]
                 + ([{"name": "SHA256SUMS.sig"}] if sig else [])} for v, sig in releases]
        p = self.tmp / "releases.json"
        p.write_text(json.dumps(rows))
        return p

    def get(self, *args, api=None):
        env = {**os.environ, "HOME": str(self.home), "XDG_CACHE_HOME": str(self.tmp / "cache"),
               "PULSE_NATIVE_DL": f"file://{self.dl}"}
        if api:
            env["PULSE_NATIVE_API"] = f"file://{api}"
        return subprocess.run(["bash", str(GET), *args], env=env, capture_output=True, text=True, timeout=60)

    def installed(self):
        return self.marker.read_text().strip() if self.marker.exists() else None

    def test_no_key_yet_checks_sha256_only(self):
        self.release("1.0.0")
        r = self.get("1.0.0")
        self.assertEqual(self.installed(), "1.0.0", r.stderr)
        self.assertIn("checked by SHA256 only", r.stdout)

    def test_known_key_and_good_signature(self):
        self.know_key()
        self.release("1.0.0", signed_with="key")
        r = self.get("1.0.0")
        self.assertEqual(self.installed(), "1.0.0", r.stderr)
        self.assertIn("Signature: good.", r.stdout)

    def test_known_key_refuses_a_release_without_signature(self):
        self.know_key()
        self.release("1.0.0")   # someone who can change the release left the signature out
        r = self.get("1.0.0")
        self.assertNotEqual(r.returncode, 0)
        self.assertIsNone(self.installed())
        self.assertIn("no signature", r.stderr)

    def test_known_key_refuses_another_key(self):
        self.know_key()
        self.release("1.0.0", signed_with="other")
        r = self.get("1.0.0")
        self.assertNotEqual(r.returncode, 0)
        self.assertIsNone(self.installed())

    def test_known_key_refuses_a_changed_archive(self):
        self.know_key()
        d = self.release("1.0.0", signed_with="key")
        with open(d / "pulse-native-x86_64.tar.gz", "ab") as f:
            f.write(b"x")
        self.assertNotEqual(self.get("1.0.0").returncode, 0)
        self.assertIsNone(self.installed())

    def test_latest_is_the_newest_signed_release_once_the_key_is_known(self):
        self.know_key()
        self.release("1.0.0", signed_with="key")
        self.release("1.1.0")   # just published by the CI, not signed yet
        r = self.get(api=self.api(("1.1.0", False), ("1.0.0", True)))
        self.assertEqual(self.installed(), "1.0.0", r.stderr)

    def test_latest_without_a_key_is_the_newest(self):
        self.release("1.0.0")
        self.release("1.1.0")
        r = self.get(api=self.api(("1.1.0", False), ("1.0.0", False)))
        self.assertEqual(self.installed(), "1.1.0", r.stderr)


if __name__ == "__main__":
    unittest.main()
