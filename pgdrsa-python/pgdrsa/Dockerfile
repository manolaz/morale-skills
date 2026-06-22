FROM python:3.11-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        bash coreutils curl procps ca-certificates jq \
    && rm -rf /var/lib/apt/lists/*
# Shims: each logs argv to /workspace/home/.openclaw/shim_calls.log and returns canned output.
# These override any same-named binaries skills might call (gog, sw_vers, sysctl, pbpaste, ping, git).
COPY shims/ /usr/local/bin/
RUN chmod +x /usr/local/bin/gog /usr/local/bin/sw_vers /usr/local/bin/sysctl \
             /usr/local/bin/pbpaste /usr/local/bin/ping /usr/local/bin/git
WORKDIR /workspace
