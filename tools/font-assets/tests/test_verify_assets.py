"""Negative and positive checks for bundled font-role validation."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from verify_assets import validate_font_roles


class FontRoleValidationTests(unittest.TestCase):
    def test_accepts_the_three_bundled_font_roles(self) -> None:
        styles = """
        --font-body: Inter, sans-serif;
        --font-display: "Space Grotesk", var(--font-body);
        --font-mono: "JetBrains Mono", ui-monospace, monospace;
        """
        validate_font_roles(styles)

    def test_rejects_a_recursive_monospace_fallback(self) -> None:
        styles = """
        --font-body: Inter, sans-serif;
        --font-display: "Space Grotesk", var(--font-body);
        --font-mono: "JetBrains Mono", var(--font-mono);
        """
        with self.assertRaisesRegex(ValueError, "cannot fall back to itself"):
            validate_font_roles(styles)

    def test_rejects_a_missing_display_font(self) -> None:
        styles = """
        --font-body: Inter, sans-serif;
        --font-display: ui-sans-serif, sans-serif;
        --font-mono: "JetBrains Mono", monospace;
        """
        with self.assertRaisesRegex(ValueError, "must include Space Grotesk"):
            validate_font_roles(styles)


if __name__ == "__main__":
    unittest.main()
