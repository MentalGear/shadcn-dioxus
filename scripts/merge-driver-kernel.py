#!/usr/bin/env python3
"""Git merge driver for the decision kernel's committed artefact `primitives/src/kernel/*` (backlog row 509).

    python3 -I scripts/merge-driver-kernel.py %O %A %B %P      merge into %A; exit 0 (clean) or 1 (conflict)
    python3 -I scripts/merge-driver-kernel.py --self-test

`decision-kernel.wasm`, `decision-kernel.b64` and `kernel.manifest` are written by scripts/build-kernel.sh from the crate's
sources, so two lanes that each add a behaviour both change all three and git always conflicts on the binary. The merge of
the artefact is not a merge of its bytes but a REBUILD of the merged sources, and this driver does that.

Git runs a merge driver BEFORE the working tree holds the merge (no MERGE_HEAD yet, the other side's files are not checked
out), so the driver cannot read the merged sources from the tree, and it sees one artefact file at a time. What it does have
are `HEAD` (ours) and the environment `git merge` sets for the merge, `GITHEAD_<commit>=<name>` (theirs: git 2.43 has no
%X or %Y placeholder to pass), and every `kernel.manifest` records the inputs of its module by git blob id. So from the
manifests of the merge base, of ours and of theirs it reads the three
versions of every input (the crate's `Cargo.toml` and sources, `rust-toolchain.toml`, the text of `[profile.kernel]`) out of
the object store, merges them per path (a side that changed a path wins over a side that did not; both changed it: a 3-way
text merge with `git merge-file`, a real conflict stays a conflict and the driver exits 1), lays the merged inputs out as a
one-crate workspace, builds it with `scripts/build-kernel.sh --root`, and writes the requested artefact into %A. The three
files of one merge are built once (the result is cached by the hash of the merged inputs). An unresolvable side, a
manifest that is missing, a conflicting input or a failed build exits 1: the conflict is left to a person, never guessed. A
merge that sets no `GITHEAD_` (a rebase, a cherry-pick) or sets several (an octopus) is that case too.

The build uses `$CARGO_TARGET_DIR` when that is an absolute path (the lane's isolated target), else a scratch directory; it
does not take the build lock (a one-crate, dependency-free build). `KERNEL_MERGE_BUILD` replaces the builder (the self-test);
`KERNEL_MERGE_CACHE` is the cache directory. Registered by scripts/setup-merge-drivers.sh (the SessionStart hook runs it);
scripts/check-merge-drivers.sh runs the self-test, which does a real two-branch merge in a scratch repository with a stub
builder, and keeps the attribute, the driver and the registration in step.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

KERNEL_DIR = "primitives/src/kernel"
MANIFEST = f"{KERNEL_DIR}/kernel.manifest"
OUTPUTS = ("decision-kernel.wasm", "decision-kernel.b64", "kernel.manifest")


def git(*args, cwd=None, data=None):
    return subprocess.run(["git", *args], cwd=cwd, input=data, capture_output=True)


def commit_of(rev):
    """The commit `rev` names, or None."""
    r = git("rev-parse", "--verify", "--quiet", rev + "^{commit}")
    return r.stdout.decode().strip() if r.returncode == 0 else None


def theirs_commit():
    """The one commit being merged in: `git merge` sets `GITHEAD_<commit>=<name>` for each head it merges."""
    heads = [k[len("GITHEAD_"):] for k in os.environ if k.startswith("GITHEAD_")]
    return commit_of(heads[0]) if len(heads) == 1 else None


def manifest_of(commit):
    r = git("show", f"{commit}:{MANIFEST}")
    if r.returncode != 0:
        return None
    m = {"src": {}, "profile": []}
    for line in r.stdout.decode().split("\n"):
        if line.startswith("src "):
            parts = line.split(" ", 3)
            if len(parts) == 4:
                m["src"][parts[3]] = parts[2]
        elif line.startswith("profile> "):
            m["profile"].append(line[len("profile> "):])
    return m


def blob(sha):
    if sha is None:
        return b""
    r = git("cat-file", "blob", sha)
    if r.returncode != 0:
        raise RuntimeError(f"the object {sha} is not in this repository")
    return r.stdout


def merge3(base, ours, theirs):
    """Three versions of one input (bytes; None is absent) -> merged bytes, None for 'deleted', or raises on conflict."""
    if ours == theirs:
        return ours
    if base == ours:
        return theirs
    if base == theirs:
        return ours
    if ours is None or theirs is None:
        raise RuntimeError("one side deleted an input the other changed")
    with tempfile.TemporaryDirectory() as tmp:
        paths = []
        for name, data in (("ours", ours), ("base", base if base is not None else b""), ("theirs", theirs)):
            p = Path(tmp, name)
            p.write_bytes(data)
            paths.append(str(p))
        r = subprocess.run(["git", "merge-file", "-p", *paths], capture_output=True)
        if r.returncode != 0:
            raise RuntimeError("the two sides changed the same lines of an input")
        return r.stdout


def merged_inputs(base_m, ours_m, theirs_m):
    """({path: bytes}, profile text) of the merged inputs."""
    files = {}
    for path in sorted(set(base_m["src"]) | set(ours_m["src"]) | set(theirs_m["src"])):
        b, o, t = (m["src"].get(path) for m in (base_m, ours_m, theirs_m))
        merged = merge3(blob(b) if b else None, blob(o) if o else None, blob(t) if t else None) if (b, o, t) != (None,) * 3 else None
        if merged is not None:
            files[path] = merged
    text = lambda m: ("\n".join(m["profile"]) + "\n").encode()
    profile = merge3(text(base_m), text(ours_m), text(theirs_m))
    return files, profile.decode()


def write_workspace(root, files, profile):
    """The merged inputs as a workspace `scripts/build-kernel.sh --root` builds (the crate, the toolchain pin, the profile)."""
    for path, data in files.items():
        target = Path(root, path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    # the crate takes its lint table from the workspace; lints do not change the module's bytes
    Path(root, "Cargo.toml").write_text(
        '[workspace]\nresolver = "3"\nmembers = ["decision-kernel"]\n\n[workspace.lints.rust]\nunsafe_code = "deny"\n\n'
        '[workspace.lints.clippy]\nallow_attributes_without_reason = "warn"\n\n' + profile
    )


def key_of(files, profile):
    h = hashlib.sha256()
    for path in sorted(files):
        h.update(path.encode() + b"\0" + hashlib.sha256(files[path]).digest())
    h.update(profile.encode())
    h.update(os.environ.get("KERNEL_MERGE_BUILD", "").encode())
    return h.hexdigest()


def build(files, profile, out):
    with tempfile.TemporaryDirectory() as ws:
        write_workspace(ws, files, profile)
        builder = os.environ.get("KERNEL_MERGE_BUILD") or str(Path(__file__).resolve().parent / "build-kernel.sh")
        env = dict(os.environ)
        if not env.get("CARGO_TARGET_DIR", "").startswith("/"):
            env["CARGO_TARGET_DIR"] = os.path.join(ws, "target")
        r = subprocess.run([builder, "--root", ws, "--out", out], env=env, capture_output=True, text=True)
        if r.returncode != 0:
            raise RuntimeError("the kernel build failed:\n" + (r.stdout + r.stderr)[-1500:])


def drive(a, path):
    ours, theirs = commit_of("HEAD"), theirs_commit()
    if not ours or not theirs:
        raise RuntimeError("the other side of the merge is not known (git merge sets GITHEAD_<commit> for exactly one head)")
    base = git("merge-base", ours, theirs).stdout.decode().strip()
    if not base:
        raise RuntimeError("no merge base")
    base_m, ours_m, theirs_m = manifest_of(base), manifest_of(ours), manifest_of(theirs)
    if not (base_m and ours_m and theirs_m):
        raise RuntimeError(f"{MANIFEST} is missing from the merge base or a side")
    files, profile = merged_inputs(base_m, ours_m, theirs_m)
    cache = Path(os.environ.get("KERNEL_MERGE_CACHE") or Path(tempfile.gettempdir(), "dx-kernel-merge-cache"), key_of(files, profile))
    if not all(Path(cache, n).is_file() for n in OUTPUTS):
        scratch = Path(str(cache) + f".tmp{os.getpid()}")
        shutil.rmtree(scratch, ignore_errors=True)
        scratch.mkdir(parents=True)
        build(files, profile, str(scratch))
        shutil.rmtree(cache, ignore_errors=True)
        scratch.rename(cache)
    name = os.path.basename(path)
    if name not in OUTPUTS:
        raise RuntimeError(f"{path} is not an artefact of the decision kernel")
    shutil.copyfile(Path(cache, name), a)


def self_test():
    """A real two-branch merge in a scratch repository, with a stub builder that is a pure function of the merged inputs."""
    root_scripts = Path(__file__).resolve().parent
    failures = []
    with tempfile.TemporaryDirectory() as tmp:
        r = Path(tmp, "repo")
        r.mkdir()
        env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid", "GIT_COMMITTER_NAME": "t",
               "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull}

        def g(*args, check=True):
            p = subprocess.run(["git", *args], cwd=r, env=env, capture_output=True, text=True)
            if check and p.returncode != 0:
                raise RuntimeError(f"git {' '.join(args)}: {p.stderr}")
            return p

        # the stub builder: the artefact is a function of the inputs under --root and nothing else
        stub = Path(tmp, "stub-build.py")
        stub.write_text(
            "#!/usr/bin/env python3\nimport base64, hashlib, subprocess, sys\nfrom pathlib import Path\n"
            "a = sys.argv\nroot, out = Path(a[a.index('--root') + 1]), Path(a[a.index('--out') + 1])\n"
            "files = sorted(p for p in root.rglob('*') if p.is_file() and 'target' not in p.parts and p.name != 'Cargo.lock')\n"
            "inputs = [p for p in files if str(p.relative_to(root)).startswith(('decision-kernel/', 'rust-toolchain'))]\n"
            "profile = ''.join(l for l in (root / 'Cargo.toml').read_text().split('[profile.kernel]')[1:])\n"
            "wasm = b'STUB' + b''.join(p.read_bytes() for p in inputs) + profile.encode()\n"
            "out.mkdir(parents=True, exist_ok=True)\n(out / 'decision-kernel.wasm').write_bytes(wasm)\n"
            "(out / 'decision-kernel.b64').write_text(base64.b64encode(wasm).decode())\n"
            "lines = ['# stub manifest']\n"
            "for p in inputs:\n    d = p.read_bytes()\n    lines.append('src %s %s %s' % (hashlib.sha256(d).hexdigest(), subprocess.run(['git', 'hash-object', '--stdin'], input=d, capture_output=True).stdout.decode().strip(), p.relative_to(root)))\n"
            "lines += ['profile> ' + l for l in ('[profile.kernel]' + profile).split('\\n') if l.strip()]\n"
            "(out / 'kernel.manifest').write_text('\\n'.join(lines) + '\\n')\n"
        )
        stub.chmod(0o755)
        for key, value in (("KERNEL_MERGE_BUILD", str(stub)), ("KERNEL_MERGE_CACHE", str(Path(tmp, "cache")))):
            os.environ[key] = value
            env[key] = value

        (r / "scripts").mkdir()
        shutil.copy(root_scripts / "merge-driver-kernel.py", r / "scripts")
        shutil.copy(root_scripts / "setup-merge-drivers.sh", r / "scripts")
        (r / ".gitattributes").write_text(f"{KERNEL_DIR}/* merge=kernel-artefact\n")
        (r / "decision-kernel/src").mkdir(parents=True)
        (r / "decision-kernel/Cargo.toml").write_text("[package]\nname = \"decision-kernel\"\n")
        (r / "decision-kernel/src/a.rs").write_text("".join(f"fn a{i}() {{}}\n" for i in range(12)))
        (r / "decision-kernel/src/b.rs").write_text("".join(f"fn b{i}() {{}}\n" for i in range(12)))
        (r / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.97.0"\n')
        (r / "Cargo.toml").write_text('[workspace]\n\n[profile.kernel]\nopt-level = "z"\n')
        g("init", "-q", "-b", "main")
        sub = subprocess.run([str(r / "scripts/setup-merge-drivers.sh"), "--repo", str(r)], capture_output=True, text=True)
        if sub.returncode != 0:
            return [f"setup-merge-drivers.sh --repo failed: {sub.stderr}"]

        def rebuild():
            subprocess.run([str(stub), "--root", str(r), "--out", str(r / KERNEL_DIR)], check=True, env=env)

        rebuild()
        g("add", "-A")
        g("commit", "-qm", "base")

        def branch(name, edits):
            g("checkout", "-q", "-b", name, "main")
            for rel, text in edits.items():
                (r / rel).write_text(text)
            rebuild()
            g("add", "-A")
            g("commit", "-qm", name)

        a_text, b_text = (r / "decision-kernel/src/a.rs").read_text(), (r / "decision-kernel/src/b.rs").read_text()
        branch("left", {"decision-kernel/src/a.rs": a_text.replace("fn a1()", "fn a1_left()")})
        branch("right", {"decision-kernel/src/b.rs": b_text.replace("fn b1()", "fn b1_right()")})
        branch("clash", {"decision-kernel/src/a.rs": a_text.replace("fn a1()", "fn a1_clash()")})
        g("checkout", "-q", "left")
        pre = g("rev-parse", "HEAD").stdout.strip()
        merged = g("merge", "--no-edit", "right", check=False)
        if merged.returncode != 0:
            failures.append(f"two lanes that each changed a different kernel source conflicted on the artefact: {merged.stdout}{merged.stderr}")
        else:
            expect = Path(tmp, "expect")
            subprocess.run([str(stub), "--root", str(r), "--out", str(expect)], check=True, env=env)
            for n in OUTPUTS:
                if (r / KERNEL_DIR / n).read_bytes() != (expect / n).read_bytes():
                    failures.append(f"{n} after the merge is not the rebuild of the merged sources")
            merged_src = (r / "decision-kernel/src/a.rs").read_text() + (r / "decision-kernel/src/b.rs").read_text()
            if "a1_left" not in merged_src or "b1_right" not in merged_src:
                failures.append("the merged sources lack a side's change")
            for n in OUTPUTS:
                if b"<<<<<<<" in (r / KERNEL_DIR / n).read_bytes():
                    failures.append(f"{n} holds a conflict marker")
        # the scenario is a real test: without the driver the same merge conflicts
        g("reset", "-q", "--hard", pre)
        g("config", "--remove-section", "merge.kernel-artefact")
        plain = g("merge", "--no-edit", "right", check=False)
        if plain.returncode == 0:
            failures.append("the scratch merge is clean without the driver: the scenario does not exercise it")
        g("merge", "--abort", check=False)
        g("reset", "-q", "--hard", pre)
        subprocess.run([str(r / "scripts/setup-merge-drivers.sh"), "--repo", str(r)], capture_output=True, check=True)
        # a real conflict (both sides changed the same line of one source) stays a conflict
        clash = g("merge", "--no-edit", "clash", check=False)
        if clash.returncode == 0:
            failures.append("two lanes that changed the same line of a kernel source merged cleanly: a conflict must stay a conflict")
        g("merge", "--abort", check=False)
        # a rebase sets no GITHEAD_: refused, never guessed
        g("reset", "-q", "--hard", pre)
        bare = subprocess.run(["python3", "-I", str(r / "scripts/merge-driver-kernel.py"), "x", str(Path(tmp, "out")), "y", f"{KERNEL_DIR}/kernel.manifest"],
                              cwd=r, env={k: v for k, v in env.items() if not k.startswith("GITHEAD_")}, capture_output=True, text=True)
        if bare.returncode == 0:
            failures.append("a merge with no GITHEAD_ was driven: the other side must be known, never guessed")
    os.environ.pop("KERNEL_MERGE_BUILD", None)
    os.environ.pop("KERNEL_MERGE_CACHE", None)
    return failures


def main(argv):
    if argv[:1] == ["--self-test"]:
        failures = self_test()
        for f in failures:
            print(f"merge-driver-kernel: self-test: {f}", file=sys.stderr)
        if not failures:
            print("merge-driver-kernel: self-test OK (two lanes that each changed a kernel source merged to the rebuild of the merged "
                  "sources; the same line changed on both sides stayed a conflict)")
        return 1 if failures else 0
    if len(argv) < 4:
        print(__doc__, file=sys.stderr)
        return 2
    _base, a, _theirs, path = argv[:4]
    try:
        drive(a, path)
    except (RuntimeError, OSError) as error:
        print(f"merge-driver-kernel: {path}: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
