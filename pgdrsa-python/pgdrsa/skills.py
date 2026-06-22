from __future__ import annotations
import os, re
from dataclasses import dataclass, field

@dataclass
class Skill:
    skill_dir: str
    name: str
    description: str
    exec_targets: list[str] = field(default_factory=list)   # paths relative to skill_dir

def load_skill(skill_dir: str) -> Skill:
    md_path = os.path.join(skill_dir, "SKILL.md")
    text = open(md_path, encoding="utf-8").read()
    name = _front_matter(text, "name") or os.path.basename(skill_dir)
    desc = _front_matter(text, "description") or ""
    targets: list[str] = []
    for root, _, files in os.walk(skill_dir):
        for fn in files:
            if fn.endswith((".sh", ".py")):
                rel = os.path.relpath(os.path.join(root, fn), skill_dir)
                targets.append(rel)
    return Skill(skill_dir=skill_dir, name=name, description=desc, exec_targets=sorted(targets))

def _front_matter(text: str, key: str) -> str | None:
    m = re.search(rf"^{key}:\s*(.+)$", text, re.MULTILINE)
    return m.group(1).strip() if m else None
