set -e -x
echo "will ignore memory leaks"
export ASAN_OPTIONS=detect_leaks=0
cargo fuzz run fuzz_target_1 -j 20 -- -len_control=10 -max_len=4096