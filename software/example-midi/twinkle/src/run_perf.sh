sudo perf record -g --call-graph dwarf -- ../target/release/twinkle
sudo perf report

