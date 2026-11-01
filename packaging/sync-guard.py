#!/usr/bin/env python3
"""Stop a sync when upstream's licence text changed (CUSTOM.md, "ถ้าใบอนุญาตของต้นทางเปลี่ยน").

    python3 packaging/sync-guard.py [REF]      REF defaults to origin/main

Compares the licence documents of HEAD with those of REF and exits 1 if one was added, removed or
edited. It only says that the text differs; reading the change and deciding is the owner's job.
"""
import re
import subprocess
import sys

# LICENSE, LICENSE-MIT, LICENSE-APACHE, COPYING, NOTICE ... at the root. LICENSES.toml and licenses/
# are not here: they list third-party icons and change with almost every upstream release.
ROOT_DOC = re.compile(r"^(LICENSE|LICENCE|COPYING|NOTICE|PATENTS)(-[A-Za-z0-9._]+)?(\.(txt|md))?$")
EXTRA = ["docs/brand/LICENSE-brand.txt"]


def git(*args):
    done = subprocess.run(["git", *args], capture_output=True, text=True)
    return done.returncode, done.stdout.strip()


def documents(ref):
    """Blob id of every licence document in `ref`."""
    code, listing = git("ls-tree", "--name-only", ref)
    if code != 0:
        sys.exit(f"sync-guard: cannot read {ref}; run `git fetch origin main` first")
    found = {}
    for name in [n for n in listing.splitlines() if ROOT_DOC.match(n)] + EXTRA:
        code, blob = git("rev-parse", "--verify", "--quiet", f"{ref}:{name}")
        if code == 0:
            found[name] = blob
    return found


def main():
    ref = sys.argv[1] if len(sys.argv) > 1 else "origin/main"
    ours, theirs = documents("HEAD"), documents(ref)
    changes = []
    for name in sorted(set(ours) | set(theirs)):
        if name not in ours:
            changes.append(f"  + {name}: new upstream, we have no such file")
        elif name not in theirs:
            changes.append(f"  - {name}: upstream removed it")
        elif ours[name] != theirs[name]:
            changes.append(f"  ~ {name}: the text differs")
    if not changes:
        print(f"sync-guard: ok ({len(ours)} licence documents are the same in HEAD and {ref})")
        return 0
    print(f"sync-guard: upstream's licence text changed in {ref}:")
    print("\n".join(changes))
    print("STOP: do not merge. Read the change (git diff HEAD " + ref + " -- <file>) and follow CUSTOM.md.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
