# satmux ✈️🛰️

Turn broken, high-latency geostationary (GEO) inflight Wi-Fi (Inmarsat GX / Viasat / Panasonic / SITA) into a fast, drop-free connection using a `$0.03/h` disposable cloud VM and **zero third-party dependencies** (pure Python 3 stdlib).

Built & benchmarked live at 38,000 ft over an Inmarsat GX Ka-band satellite link (`~620 ms` RTT).

```text
  ╭──────────────────────────────────────────────────────────────────────────────────╮
  │  ✈️  INFLIGHT GEO SATELLITE BENCHMARK   ·  Inmarsat GX Ka-band (~620 ms RTT)     │
  │  Direct Cabin Wi-Fi (:443 PEP)  vs  12× Warm BBR + 1-RTT Ground-TLS + zlib       │
  ╰──────────────────────────────────────────────────────────────────────────────────╯

  ORIGIN (6× PARALLEL)  RAW CABIN WI-FI (:443)    1-RTT BBR + ZLIB MUX    SPEEDUP
  ────────────────────  ────────────────────────  ──────────────────────  ─────────
  GitHub Repo           ████████████    TIMEOUT  ███░░░░░░░░░    4.66s  FIXED ✓
  GitHub API            ████████████  TCP RESET  ██░░░░░░░░░░    2.66s  FIXED ✓
  Hacker News           ████████████    TIMEOUT  ██░░░░░░░░░░    2.28s  FIXED ✓
  MDN Docs              ████████░░░░     12.01s  ██░░░░░░░░░░    2.30s     5.2×
  Cloudflare            ██░░░░░░░░░░      2.97s  █░░░░░░░░░░░    1.40s     2.1×
  PyPI JSON API         █████░░░░░░░      7.09s  ██░░░░░░░░░░    2.64s     2.7×
  ──────────────────────────────────────────────────────────────────────────────────
  6-Origin Burst:    19.13s (3/6 failed)  →  5.35s (0/6 failed)   [3.6× faster · -76% TTFB]
  Tunnel Stack:      12× warm TCP pipes · 0-RTT Open · 1-RTT Ground-TLS · Linux BBR · zlib(6)
  Payload Delivered: 535 KB uncompressed (~118 KB on satellite wire via $0.03/h EU VM)
```

---

## Why Inflight Wi-Fi Feels Worse Than Its Speed Cap

Even when you pay for inflight Wi-Fi on a non-Starlink aircraft, three bottlenecks ruin it before bandwidth even matters:

1. **3–4 trips to space before byte 1 (`~2.0–2.6s` wasted per host):** A geostationary satellite sits at `35,786 km` (`~600–680 ms` round-trip). Every new domain normally pays multiple satellite round-trips for DNS + TCP + TLS handshakes.
2. **Overloaded onboard Port-443 proxy & hijacked DNS:** Most airlines run an onboard transparent proxy (`:443` Split-TCP PEP + spoofed `:53` DNS) whose connection table constantly overflows—causing `Connection reset by peer` and `10–50s` freezes on `github.com`, `api.github.com`, package registries, and dev tools.
3. **Packet-loss TCP collapse:** Standard TCP mistakes random cabin Wi-Fi retries for congestion and slashes your download window, dropping a `4.5 Mbps` seat allowance down to `0.8–1.5 Mbps`.

---

## How `satmux` Works

```text
BEFORE (Direct Cabin Wi-Fi):
  Laptop ──▶ [Hijacked DNS + Broken Plane :443 Proxy] ══(3–4× 36,000 km GEO hops)══▶ TCP Resets / 12s loads

AFTER (satmux):
  Laptop ══(12× pre-opened TCP :8444 BBR + zlib pipes — bypasses :443 proxy)══▶ Ground Fiber VM ──(2 ms)──▶ Internet
```

- **Bypasses `:443` middleboxes & DNS hijacking:** Holds **12 persistent, pre-warmed TCP connections** open on a non-443 port (`:8444`) to your ground VM.
- **0-RTT Pipelined `SOCKS5` & `HTTP CONNECT` (`127.0.0.1:1080` / `:8080`):** Replies `OK` locally in `0.1 ms` and ships `FT_OPEN + TLS ClientHello` in the same satellite packet, saving a full satellite RTT on every browser & CLI connection while keeping end-to-end TLS intact.
- **1-RTT Ground-TLS + `zlib` Accelerator (`127.0.0.1:8081`):** Sends an encrypted (`BLAKE2b-512` keystream + `HMAC-SHA256`) fetch frame over a warm pipe so your ground VM resolves DNS, TLS 1.3, and HTTP redirects in **~2–5 ms on fiber**, compresses plaintext HTML/JSON **70–80% with `zlib`**, and streams it down in **1 satellite RTT**.
- **Fixes Coding Agents (`git` + `gh` CLI):** Go binaries like `gh` ignore macOS system proxies and fail with `Post "https://api.github.com/graphql": connection reset by peer`. `satmux` exposes a local `0600` Unix socket (`http_unix_socket`) that wraps `gh` traffic in verified end-to-end TLS on your laptop and tunnels it over the warm carrier pool—fixing already-running agents immediately.
- **Linux BBR + `MSS 1280`:** Ground VM paces packets using Google's BBR congestion control instead of collapsing on cabin Wi-Fi jitter.

---

## What to Expect (Honest Physics)

| What improves dramatically | What stays bounded by physics / airline router |
| --- | --- |
| **0 dropped connections** on GitHub, APIs, registries (`gh`, `git`, browsers) | **Latency floor (~580–650 ms):** Radio waves still travel `~143,000 km` to GEO orbit and back |
| **2×–5× faster web, API, JSON, docs & code loads** (`-70%` handshake lag + `zlib` wire compression) | **Raw binary ceiling (~4–5 Mbps):** Already-compressed `.mp4`/`.zip` files still respect your single paid seat's cabin router cap |
| **Steady max-cap downloads** (`~3.8–4.5 Mbps` via BBR + parallel `satmux-get` vs `1 Mbps` dips) | **Single legitimate subscription:** Accelerates *your* paid seat session; does not bypass airline billing |

---

## What You Need

1. **One cheap Linux VPS (`~$0.01–$0.03/h`)** with Python 3.9+ and an open TCP port (e.g., `8444`):
   - **Where to pick the region:** Pick a datacenter near your airline's satellite ground gateway!
     - **Middle East / European carriers (Emirates, Qatar, Lufthansa, BA, AF):** **EU** (`Hetzner Helsinki / Falkenstein`, `Frankfurt`, `Amsterdam` — Inmarsat GX / SITA Europe lands in NL/DE).
     - **North American carriers:** `us-east` (Virginia/NY) or `us-west`.
     - **Asian/Pacific carriers:** `Singapore` or `Tokyo`.
2. **macOS or Linux laptop** with Python 3.9+.

*(Tip: Pause VPNs like Tailscale/WireGuard while inflight—aircraft firewalls drop UDP, and background UDP retry storms steal your cabin Wi-Fi airtime.)*

---

## Quick Start (60 Seconds)

### Option A: Just ask your AI coding agent
> *"Set up `https://github.com/Th0rgal/satmux` on a temporary EU Ubuntu VM (`:8444`), start the local 12-carrier client, enable `satmux-macos on`, and run `satmux-bench`."*

### Option B: Manual 3-step setup

**1. Generate a pre-shared key on your laptop:**
```bash
git clone https://github.com/Th0rgal/satmux.git && cd satmux
openssl rand -hex 32 > mux.psk && chmod 600 mux.psk
```

**2. Start the relay on your Linux VM (`<SERVER_IP>`):**
```bash
scp satmux.py mux.psk root@<SERVER_IP>:/root/
ssh root@<SERVER_IP> 'nohup env SATMUX_PSK=$(cat /root/mux.psk) python3 /root/satmux.py server --port 8444 >/var/log/satmux.log 2>&1 &'
```

**3. Start the local client & toggle macOS routing on:**
```bash
SATMUX_PSK=$(cat mux.psk) python3 satmux.py client --host <SERVER_IP> --port 8444 --carriers 12 &
./bin/satmux-macos on <SERVER_IP>
```

**4. Verify & benchmark:**
```bash
./bin/satmux-bench
# Parallel multi-carrier downloader for large releases/tarballs:
./bin/satmux-get https://example.com/large-archive.tar.gz
```

**5. When you land (important!):**
```bash
./bin/satmux-macos off
```
This cleanly restores macOS Wi-Fi proxy settings, `git`, `gh`, and `launchd` back to normal ground defaults. Destroy your temporary cloud VM and you're done!

---

## License

MIT
