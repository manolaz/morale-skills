from pgdrsa.skills import load_skill

def test_load_real_benign_skill():
    s = load_skill("data/skills/benign/076-memory-note")
    assert s.name == "note"
    assert any(t.endswith("capture_note.py") for t in s.exec_targets)

def test_load_real_malicious_skill():
    s = load_skill("data/skills/malicious/001-network-ip-info")
    assert s.name == "ip-info"
    assert any(t.endswith("ip-info.py") for t in s.exec_targets)
