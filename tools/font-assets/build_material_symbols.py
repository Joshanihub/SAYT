"""Build and verify the small Material Symbols subset used by SAYT."""

from __future__ import annotations

import hashlib
import io
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.request import urlopen

from fontTools.ttLib import TTFont


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
LIGATURES_PATH = Path(__file__).with_name("material-symbols-m1-ligatures.txt")
OUTPUT_PATH = (
    REPOSITORY_ROOT
    / "apps"
    / "desktop"
    / "src"
    / "assets"
    / "fonts"
    / "material-symbols-outlined-m1.woff2"
)
SOURCE_URL = (
    "https://fonts.gstatic.com/s/materialsymbolsoutlined/v387/"
    "kJEhBvYX7BgnkSrUwT8OhrdQw4oELdPIeeII9v6oFsI.woff2"
)
SOURCE_SHA256 = "632a98f65389d16fa46d925092888d5bd3e0c4b7376da4bad74c398de6bd544f"


def ligature_map(font: TTFont) -> dict[str, str]:
    """Return text ligatures and their resulting Material Symbols glyph names."""
    reverse_cmap = {
        glyph_name: chr(codepoint)
        for codepoint, glyph_name in font.getBestCmap().items()
        if codepoint <= 0x10FFFF
    }
    mappings: dict[str, str] = {}
    for lookup in font["GSUB"].table.LookupList.Lookup:
        for extension in lookup.SubTable:
            substitution = getattr(extension, "ExtSubTable", extension)
            for first_glyph, ligatures in getattr(
                substitution, "ligatures", {}
            ).items():
                for ligature in ligatures:
                    try:
                        text = reverse_cmap[first_glyph] + "".join(
                            reverse_cmap[glyph] for glyph in ligature.Component
                        )
                    except KeyError:
                        continue
                    mappings[text] = ligature.LigGlyph
    return mappings


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    icon_names = [
        name.strip()
        for name in LIGATURES_PATH.read_text(encoding="utf-8").splitlines()
        if name.strip()
    ]
    if not icon_names:
        raise SystemExit("Icon manifest is empty.")

    with urlopen(SOURCE_URL, timeout=30) as response:
        source_bytes = response.read()
    source_hash = sha256_bytes(source_bytes)
    if source_hash != SOURCE_SHA256:
        raise SystemExit(
            f"Pinned Material Symbols source hash mismatch: {source_hash}"
        )

    source_font = TTFont(io.BytesIO(source_bytes))
    source_mappings = ligature_map(source_font)
    missing = [name for name in icon_names if name not in source_mappings]
    if missing:
        raise SystemExit(f"Pinned font lacks requested ligatures: {missing}")
    output_glyphs = sorted({source_mappings[name] for name in icon_names})

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=OUTPUT_PATH.parent) as temporary_dir:
        temporary = Path(temporary_dir)
        input_font = temporary / "material-symbols-source.woff2"
        glyph_names_path = temporary / "glyph-names.txt"
        ligatures_path = temporary / "ligatures.txt"
        subset_font = temporary / "material-symbols-subset.woff2"
        input_font.write_bytes(source_bytes)
        glyph_names_path.write_text("\n".join(output_glyphs), encoding="utf-8")
        ligatures_path.write_text("\n".join(icon_names), encoding="utf-8")

        subprocess.run(
            [
                sys.executable,
                "-m",
                "fontTools.subset",
                str(input_font),
                f"--glyphs-file={glyph_names_path}",
                f"--text-file={ligatures_path}",
                f"--output-file={subset_font}",
                "--flavor=woff2",
                "--layout-features=*",
                "--no-layout-closure",
                "--name-IDs=*",
                "--glyph-names",
            ],
            check=True,
        )

        subset_bytes = subset_font.read_bytes()
        subset_mappings = ligature_map(TTFont(io.BytesIO(subset_bytes)))
        invalid = [
            name
            for name in icon_names
            if subset_mappings.get(name) != source_mappings[name]
        ]
        if invalid:
            raise SystemExit(f"Subset lost or changed ligatures: {invalid}")

        temporary_output = OUTPUT_PATH.with_suffix(".verified.woff2")
        temporary_output.write_bytes(subset_bytes)
        temporary_output.replace(OUTPUT_PATH)

    print(f"Verified {len(icon_names)} Material Symbols ligatures.")
    print(f"Output: {OUTPUT_PATH.relative_to(REPOSITORY_ROOT)}")
    print(f"Bytes: {len(subset_bytes)}")
    print(f"SHA-256: {sha256_bytes(subset_bytes)}")


if __name__ == "__main__":
    main()
