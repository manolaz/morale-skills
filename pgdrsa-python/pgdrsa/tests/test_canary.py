from pgdrsa.canary import make_canary, find_canaries

def test_make_and_find():
    tok = make_canary("r1-0")
    assert tok == "RSA-CANARY-r1-0"
    assert find_canaries('{"data":"RSA-CANARY-r1-0"}', [tok]) == [tok]
    assert find_canaries("nothing", [tok]) == []
    assert find_canaries("anything", [""]) == []
