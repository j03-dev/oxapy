#!/usr/bin/env python3
"""Comprehensive stub generation and fixing script for oxapy.

This script:
1. Generates stub files using the Rust stub_gen binary
2. Applies multiple fix patterns to correct common stub generation issues
3. Provides detailed logging and statistics
"""

import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple, Dict


class StubFixer:
    """Handles multiple types of stub file fixes."""

    def __init__(self):
        self.fixes_applied = 0
        self.patterns_fixed = {}

    def fix_new_method_tuple_returns(self, content: str) -> str:
        """Fix __new__ methods that incorrectly return tuple[ClassName, ...] instead of ClassName."""
        # Pattern 1: tuple[ClassName, OtherClass] -> ClassName
        pattern1 = r'(def __new__\([^)]+\)\s*->\s*)tuple\[([A-Za-z_][A-Za-z0-9_]*),\s*[A-Za-z_][A-Za-z0-9_]*\](\s*:\s*\.\.\.)'

        def replace_fn1(match):
            prefix = match.group(1)
            class_name = match.group(2)
            suffix = match.group(3)
            return f"{prefix}{class_name}{suffix}"

        new_content = re.sub(pattern1, replace_fn1, content)
        fixes1 = len(re.findall(pattern1, content))

        # Pattern 2: tuple[ClassName, ...] (general case)
        pattern2 = r'(def __new__\([^)]+\)\s*->\s*)tuple\[([A-Za-z_][A-Za-z0-9_]*),\s*[^\]]+\](\s*:\s*\.\.\.)'

        def replace_fn2(match):
            prefix = match.group(1)
            class_name = match.group(2)
            suffix = match.group(3)
            return f"{prefix}{class_name}{suffix}"

        new_content = re.sub(pattern2, replace_fn2, new_content)
        fixes2 = len(re.findall(pattern2, content))

        total_fixes = fixes1 + fixes2
        if total_fixes > 0:
            self.patterns_fixed["__new__ tuple returns"] = total_fixes
            self.fixes_applied += total_fixes

        return new_content

    def fix_property_annotations(self, content: str) -> str:
        """Fix property method annotations that may be incorrect."""
        # Fix getter/setter property pairs that might have incorrect types
        pattern = r'(@property\s+def\s+\w+\([^)]+\)\s*->\s*)typing\.Optional\[([^\]]+)\](\s*:\s*\.\.\.)'

        def replace_fn(match):
            prefix = match.group(1)
            inner_type = match.group(2)
            suffix = match.group(3)
            # For properties, often the Optional wrapper is unnecessary in getters
            return f"{prefix}{inner_type}{suffix}"

        new_content = re.sub(pattern, replace_fn, content)
        fixes = len(re.findall(pattern, content))

        if fixes > 0:
            self.patterns_fixed["property annotations"] = fixes
            self.fixes_applied += fixes

        return new_content

    def fix_enum_new_methods(self, content: str) -> str:
        """Fix enum class __new__ methods that may have incorrect signatures."""
        # Pattern for enum inner classes with __new__ methods
        pattern = r'(class\s+\w+\s*\[L\d+-\d+\]:\s+def __new__\([^)]+\)\s*->\s*)([A-Za-z_][A-Za-z0-9_]*\.[A-Za-z_][A-Za-z0-9_]*)'

        def replace_fn(match):
            prefix = match.group(1)
            full_class_name = match.group(2)
            # Extract just the inner class name
            inner_class = full_class_name.split('.')[-1]
            return f"{prefix}{inner_class}"

        new_content = re.sub(pattern, replace_fn, content)
        fixes = len(re.findall(pattern, content))

        if fixes > 0:
            self.patterns_fixed["enum __new__ methods"] = fixes
            self.fixes_applied += fixes

        return new_content

    def fix_generic_type_issues(self, content: str) -> str:
        """Fix various generic type annotation issues."""
        fixes = 0

        # Fix: Remove redundant typing. prefixes where builtins would work
        pattern1 = r'typing\.(str|int|float|bool|dict|list|tuple)'
        new_content = re.sub(pattern1, r'builtins.\1', content)
        fixes += len(re.findall(pattern1, content))

        # Fix: Simplify complex Optional[Union[...]] patterns
        pattern2 = r'typing\.Optional\[typing\.Union\[([^,]+),\s*None\]\]'
        new_content = re.sub(pattern2, r'typing.Optional[\1]', new_content)
        fixes += len(re.findall(pattern2, content))

        if fixes > 0:
            self.patterns_fixed["generic type issues"] = fixes
            self.fixes_applied += fixes

        return new_content

    def fix_serializer_field_issues(self, content: str) -> str:
        """Fix specific issues in serializer field definitions."""
        fixes = 0

        # Fix Field subclass __new__ methods returning incorrect tuple types
        pattern = r'(class\s+\w*Field[^:]*:.*?def __new__\([^)]+\)\s*->\s*)tuple\[(\w+Field),\s*Field\]'

        def replace_fn(match):
            prefix = match.group(1)
            field_class = match.group(2)
            return f"{prefix}{field_class}"

        new_content = re.sub(pattern, replace_fn, content, flags=re.DOTALL)
        fixes += len(re.findall(pattern, content, re.DOTALL))

        if fixes > 0:
            self.patterns_fixed["serializer field returns"] = fixes
            self.fixes_applied += fixes

        return new_content

    def apply_all_fixes(self, content: str) -> str:
        """Apply all available fixes to the content."""
        content = self.fix_new_method_tuple_returns(content)
        content = self.fix_property_annotations(content)
        content = self.fix_enum_new_methods(content)
        content = self.fix_generic_type_issues(content)
        content = self.fix_serializer_field_issues(content)
        return content


def generate_stubs() -> bool:
    """Generate stub files using the Rust binary."""
    print("🔧 Generating stub files...")
    try:
        result = subprocess.run(
            ["cargo", "run", "--bin", "stub_gen", "--features", "stub-gen"],
            cwd="../../",  # Run from src/bin directory
            capture_output=True,
            text=True,
            check=True
        )
        print("✓ Stub generation completed successfully")
        if result.stdout.strip():
            print(f"  Output: {result.stdout.strip()}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"✗ Stub generation failed with exit code {e.returncode}")
        if e.stdout:
            print(f"  Stdout: {e.stdout}")
        if e.stderr:
            print(f"  Stderr: {e.stderr}")
        return False
    except FileNotFoundError:
        print("✗ Could not find cargo command")
        return False


def process_pyi_files(directory: str = "../../oxapy") -> Dict[str, int]:
    """Process all .pyi files in the directory."""
    path = Path(directory)
    fixer = StubFixer()
    stats = {
        "processed": 0,
        "modified": 0,
        "errors": 0,
        "total_fixes": 0
    }

    print(f"🔍 Processing .pyi files in {path.resolve()}")

    for pyi_file in path.rglob("*.pyi"):
        stats["processed"] += 1
        try:
            original = pyi_file.read_text(encoding="utf-8")

            # Create a new fixer instance for each file to track per-file stats
            file_fixer = StubFixer()
            fixed = file_fixer.apply_all_fixes(original)

            if original != fixed:
                pyi_file.write_text(fixed, encoding="utf-8")
                stats["modified"] += 1
                stats["total_fixes"] += file_fixer.fixes_applied

                print(f"✓ Fixed: {pyi_file.relative_to(path)} ({file_fixer.fixes_applied} fixes)")
                if file_fixer.patterns_fixed:
                    for pattern, count in file_fixer.patterns_fixed.items():
                        print(f"    - {pattern}: {count}")
            else:
                print(f"  Skipped: {pyi_file.relative_to(path)} (no changes needed)")

        except Exception as e:
            print(f"✗ Error processing {pyi_file.relative_to(path)}: {e}")
            stats["errors"] += 1

    return stats


def main():
    """Main entry point."""
    print("🚀 Starting comprehensive stub generation and fixing...")
    print("=" * 60)

    # Step 1: Generate stubs
    if not generate_stubs():
        print("\n❌ Failed to generate stubs. Exiting.")
        sys.exit(1)

    print()

    # Step 2: Fix stubs
    stats = process_pyi_files()

    print("\n" + "=" * 60)
    print("📊 Summary:")
    print(f"  Files processed: {stats['processed']}")
    print(f"  Files modified: {stats['modified']}")
    print(f"  Total fixes applied: {stats['total_fixes']}")
    print(f"  Errors encountered: {stats['errors']}")

    if stats['errors'] > 0:
        print(f"\n⚠️  {stats['errors']} file(s) had errors during processing")
        sys.exit(1)
    elif stats['modified'] > 0:
        print(f"\n✅ Successfully processed and fixed {stats['modified']} stub file(s)")
    else:
        print("\n✅ All stub files were already up to date")


if __name__ == "__main__":
    main()
