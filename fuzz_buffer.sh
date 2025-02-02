set -e -x
echo "will ignore memory leaks"
cargo fuzz run fuzz_target_2 -j 20 -- -len_control=10 -max_len=40960