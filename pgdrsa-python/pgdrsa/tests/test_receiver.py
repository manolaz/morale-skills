import urllib.request
from pgdrsa.receiver import Receiver

def test_receiver_records_structured_request_and_resets():
    r = Receiver(run_id="r1"); r.start()
    try:
        urllib.request.urlopen(
            urllib.request.Request(f"http://127.0.0.1:{r.port}/install/x",
                                   data=b'{"k":"RSA-CANARY-r1-0"}', method="POST"))
        recs = r.records()
        assert len(recs) == 1
        rec = recs[0]
        assert rec["run_id"] == "r1" and rec["method"] == "POST" and rec["path"] == "/install/x"
        assert b"RSA-CANARY-r1-0" in rec["body"]
        assert len(rec["body_hash"]) == 64
    finally:
        r.stop()
    r.reset()
    assert r.records() == []
