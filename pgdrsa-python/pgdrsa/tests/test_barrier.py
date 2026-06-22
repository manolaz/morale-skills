from pgdrsa.barrier import finalize_status

def test_quiescent_then_quiet_is_complete():
    # process lister returns [] (no live procs); receiver got the canary before quiescence
    status = finalize_status(live_procs=lambda: [], receiver_quiet=lambda: True, fs_stable=lambda: True)
    assert status == "COMPLETE"

def test_procs_never_drain_is_censored():
    status = finalize_status(live_procs=lambda: ["curl"], receiver_quiet=lambda: False,
                             fs_stable=lambda: True, max_polls=3)
    assert status == "CENSORED"

def test_unstable_fs_is_censored():
    status = finalize_status(live_procs=lambda: [], receiver_quiet=lambda: True,
                             fs_stable=lambda: False, max_polls=3)
    assert status == "CENSORED"
