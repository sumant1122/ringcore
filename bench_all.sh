#!/bin/bash

# Master Benchmark Script for RingCore
set -e

echo "====================================================="
echo "   RingCore vs. Standard vs. Tokio Benchmark Suite   "
echo "====================================================="

# 1. Compilation
echo -e "\n[1/4] Compiling all benchmark targets..."
cargo build --release --examples > /dev/null 2>&1
echo "Done."

# 2. Sequential File I/O (Cat 100MB)
echo -e "\n[2/4] Benchmarking Sequential File I/O (100MB)..."
TEST_FILE="bench_large_data.bin"
if [ ! -f "$TEST_FILE" ]; then
    dd if=/dev/urandom of="$TEST_FILE" bs=1M count=100 2>/dev/null
fi

STD_CAT_TIME=$( { time ./target/release/examples/std_cat "$TEST_FILE" > /dev/null; } 2>&1 | grep real | awk '{print $2}')
TOKIO_CAT_TIME=$( { time ./target/release/examples/tokio_cat "$TEST_FILE" > /dev/null; } 2>&1 | grep real | awk '{print $2}')
RING_CAT_TIME=$( { time ./target/release/examples/cat "$TEST_FILE" > /dev/null; } 2>&1 | grep real | awk '{print $2}')

# 3. Concurrent File I/O (100 Small Files)
echo -e "[3/4] Benchmarking Concurrent File Reads (100 files)..."
STD_CONC=$(./target/release/examples/std_concurrent_reads | grep "in" | sed 's/.*in //')
TOKIO_CONC=$(./target/release/examples/tokio_concurrent_reads | grep "in" | sed 's/.*in //')
RING_CONC=$(./target/release/examples/concurrent_reads | grep "in" | sed 's/.*in //')

# 4. Networking (100 HTTP Requests)
echo -e "[4/4] Benchmarking HTTP Networking (100 reqs)..."

run_net_bench() {
    SERVER_BIN=$1
    CLIENT_BIN=$2
    $SERVER_BIN > /dev/null 2>&1 &
    PID=$!
    sleep 2
    # Standard client: 100 requests in 12.828445ms
    # RingCore Stress Client: Finished 200 tasks in 64.848107ms
    RESULT=$($CLIENT_BIN | grep "Finished" | sed 's/.*in //')
    if [ -z "$RESULT" ]; then
        # Fallback for simple clients that don't say "Finished"
        RESULT=$($CLIENT_BIN | grep "in" | sed 's/.*in //')
    fi
    kill $PID 2>/dev/null || true
    wait $PID 2>/dev/null || true
    echo "$RESULT"
}
NET_STD=$(run_net_bench "./target/release/examples/std_http_server" "./target/release/examples/std_http_client")
NET_TOKIO=$(run_net_bench "./target/release/examples/tokio_http_server" "./target/release/examples/tokio_http_client")
NET_RING=$(run_net_bench "./target/release/examples/http_server" "./target/release/examples/http_client")

# 5. High Concurrency Networking (1000 requests, 200 concurrent tasks)
echo -e "[5/5] Benchmarking Stress Test (1000 total, 200 concurrent)..."
NET_STD_STRESS=$(run_net_bench "./target/release/examples/std_http_server" "./target/release/examples/std_http_stress_client")
NET_TOKIO_STRESS=$(run_net_bench "./target/release/examples/tokio_http_server" "./target/release/examples/tokio_http_stress_client")
NET_RING_STRESS=$(run_net_bench "./target/release/examples/http_server" "./target/release/examples/http_stress_client")

# 6. Results Summary
echo -e "\n====================================================="
echo "                FINAL BENCHMARK RESULTS               "
echo "====================================================="
printf "%-20s | %-12s | %-12s | %-12s\n" "Test Case" "Standard" "Tokio" "RingCore"
echo "-----------------------------------------------------"
printf "%-20s | %-12s | %-12s | %-12s\n" "Seq Cat (100MB)" "$STD_CAT_TIME" "$TOKIO_CAT_TIME" "$RING_CAT_TIME"
printf "%-20s | %-12s | %-12s | %-12s\n" "Conc Reads (100f)" "$STD_CONC" "$TOKIO_CONC" "$RING_CONC"
printf "%-20s | %-12s | %-12s | %-12s\n" "HTTP (100 reqs)" "$NET_STD" "$NET_TOKIO" "$NET_RING"
printf "%-20s | %-12s | %-12s | %-12s\n" "Stress (1k reqs)" "$NET_STD_STRESS" "$NET_TOKIO_STRESS" "$NET_RING_STRESS"
echo "====================================================="

# 8. Timer Accuracy (Informational)

echo -e "\n[Bonus] Comparing Timer Accuracy (1s sleep)..."
run_timer() {
    ./target/release/examples/$1 | grep "1 second passed" | sed 's/.*at //'
}
T_STD=$(run_timer "std_timer")
T_TOKIO=$(run_timer "tokio_timer")
T_RING=$(run_timer "timer")

echo "Std   : $T_STD"
echo "Tokio : $T_TOKIO"
echo "Ring  : $T_RING"

echo "====================================================="

# Cleanup
rm -f "$TEST_FILE"
rm -f std_app.log tokio_app.log ring_app.log
