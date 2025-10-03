#!/usr/bin/env python3
"""Fix __new__ method signatures in generated .pyi stub files."""

import re
from pathlib import Path


def fix_new_signatures(content: str) -> str:
    pattern = r'(def __new__\([^)]+\)\s*->\s*)tuple\[([A-Za-z_][A-Za-z0-9_]*),\s*[^\]]+\](\s*:\s*\.\.\.)'

    def replace_fn(match):
        prefix = match.group(1)
        class_name = match.group(2)
        suffix = match.group(3)
        return f"{prefix}{class_name}{suffix}"

    return re.sub(pattern, replace_fn, content)


def process_pyi_files(directory: str = "."):
    path = Path(directory)
    fixed_count = 0
    for pyi_file in path.rglob("*.pyi"):
        try:
            original = pyi_file.read_text(encoding="utf-8")
            fixed = fix_new_signatures(original)
            if original != fixed:
                pyi_file.write_text(fixed, encoding="utf-8")
                print(f"✓ Fixed: {pyi_file}")
                fixed_count += 1
            else:
                print(f"  Skipped: {pyi_file} (no changes needed)")
        except Exception as e:
            print(f"✗ Error processing {pyi_file}: {e}")
    print(f"\n{fixed_count} file(s) fixed.")


if __name__ == "__main__":
    process_pyi_files("../../oxapy")
