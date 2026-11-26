"""Verify bundled Stitch fonts, icon subset, and third-party notices."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = Path(__file__).with_name("asset-manifest.json")
CSS_PATH = REPOSITORY_ROOT / "apps" / "desktop" / "src" / "fonts.css"
STYLE_PATH = REPOSITORY_ROOT / "apps" / "desktop" / "src" / "styles.css"
ICON_COMPONENT_PATH = (
    REPOSITORY_ROOT / "apps" / "desktop" / "src" / "MaterialIcon.tsx"
)
ICON_MANIFEST_PATH = Path(__file__).with_name("material-symbols-m1-ligatures.txt")


def validate_font_roles(styles: str) -> None:
    """Reject missing families and recursive CSS variable fallbacks."""
    required_font_roles = {
        "--font-body": "Inter",
        "--font-display": "Space Grotesk",
        "--font-mono": "JetBrains Mono",
    }
    for token, family in required_font_roles.items():
        declaration = re.search(rf"{re.escape(token)}\s*:\s*([^;]+);", styles)
        if declaration is None or family not in declaration.group(1):
            raise ValueError(f"Font role {token} must include {family}.")
        if token in declaration.group(1):
            raise ValueError(f"Font role {token} cannot fall back to itself.")


def read_verified_file(relative_path: str, expected_bytes: int, expected_sha256: str) -> bytes:
    path = (REPOSITORY_ROOT / relative_path).resolve()
    if not path.is_relative_to(REPOSITORY_ROOT):
        raise ValueError(f"Manifest path escapes the repository: {relative_path}")
    content = path.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    if len(content) != expected_bytes or digest != expected_sha256:
        raise ValueError(f"Asset integrity mismatch: {relative_path}")
    return content


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8-sig"))
    assets = manifest["assets"]
    notices = manifest["licenseNotices"]
    for record in [*assets, *notices]:
        read_verified_file(record["path"], record["bytes"], record["sha256"])

    css = CSS_PATH.read_text(encoding="utf-8-sig")
    if re.search(r"https?://", css, flags=re.IGNORECASE):
        raise ValueError("Font CSS must load local assets without network access.")
    css_assets: set[str] = set()
    for reference in re.findall(r"url\(\s*['\"]?([^'\")]+)", css):
        asset_path = (CSS_PATH.parent / reference).resolve()
        if not asset_path.is_relative_to(REPOSITORY_ROOT):
            raise ValueError(f"Font CSS URL escapes the repository: {reference}")
        css_assets.add(asset_path.relative_to(REPOSITORY_ROOT).as_posix())
        if not asset_path.is_file():
            raise ValueError(f"Font CSS asset is missing: {reference}")

    manifest_assets = {record["path"] for record in assets}
    if css_assets != manifest_assets:
        raise ValueError("Font CSS references and the pinned asset manifest differ.")

    validate_font_roles(STYLE_PATH.read_text(encoding="utf-8-sig"))

    icon_names = {
        name.strip()
        for name in ICON_MANIFEST_PATH.read_text(encoding="utf-8-sig").splitlines()
        if name.strip()
    }
    component = ICON_COMPONENT_PATH.read_text(encoding="utf-8-sig")
    names_block = re.search(
        r"M1_MATERIAL_ICON_NAMES\s*=\s*\[(.*?)\]\s+as const",
        component,
        flags=re.DOTALL,
    )
    if names_block is None:
        raise ValueError("Could not read the M1 icon name list from its component.")
    m1_names = set(re.findall(r'"([a-z0-9_]+)"', names_block.group(1)))
    if not m1_names or not m1_names.issubset(icon_names):
        raise ValueError("An M1 MaterialIcon is absent from the bundled subset list.")

    required_notice_names = {
        "Inter-OFL.txt",
        "Space-Grotesk-OFL.txt",
        "JetBrains-Mono-OFL.txt",
        "Material-Design-Icons-Apache-2.0.txt",
    }
    actual_notice_names = {Path(record["path"]).name for record in notices}
    if not required_notice_names.issubset(actual_notice_names):
        raise ValueError("One or more upstream font license notices are missing.")

    print(
        f"Verified {len(assets)} local WOFF2 assets, {len(notices)} license notices, "
        f"and {len(m1_names)} M1 icon names."
    )


if __name__ == "__main__":
    main()
