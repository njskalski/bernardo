in debian:

sudo apt install linux-perf hotspot
sudo sysctl kernel.perf_event_paranoid=-1

run:
cargo flamegraph --bin gladius

hotspot perf.data