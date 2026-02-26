#!/bin/bash
# Performance comparison between Python (thefuck) and Rust (thefuck-rs)
# This script measures startup time, correction time, and memory usage

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARENT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$SCRIPT_DIR/benchmark_results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Number of iterations
ITERATIONS=${1:-100}

# Test commands and their expected outputs
declare -A TEST_COMMANDS
TEST_COMMANDS["git_typo"]="git psuh origin main"
TEST_COMMANDS["permission"]="cat /etc/shadow"
TEST_COMMANDS["mkdir"]="mkdir -p /tmp/test/nested"

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}          TheFuck Performance Comparison${NC}"
echo -e "${BLUE}          Python vs Rust${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Iterations: ${YELLOW}$ITERATIONS${NC}"
echo -e "Date: $(date)"
echo ""

# Build Rust version in release mode
echo -e "${GREEN}Building Rust version (release mode)...${NC}"
cd "$SCRIPT_DIR"
cargo build --release 2>/dev/null
RUST_BIN="$SCRIPT_DIR/target/release/thefuck"
echo ""

# Check if Python thefuck is available
echo -e "${GREEN}Checking Python thefuck installation...${NC}"
if command -v thefuck &> /dev/null; then
    PYTHON_BIN="thefuck"
    PYTHON_VERSION=$(python3 -c "import thefuck; print(thefuck.__version__)" 2>/dev/null || echo "unknown")
    echo -e "Python thefuck version: ${YELLOW}$PYTHON_VERSION${NC}"
else
    echo -e "${RED}Python thefuck not found. Install with: pip install thefuck${NC}"
    PYTHON_BIN=""
fi
echo ""

# Get Rust version
RUST_VERSION=$("$RUST_BIN" version 2>/dev/null || echo "0.1.0")
echo -e "Rust thefuck-rs version: ${YELLOW}$RUST_VERSION${NC}"
echo ""

# Benchmark function for timing
benchmark_startup() {
    local name=$1
    local cmd=$2
    local iterations=$3

    echo -e "${BLUE}Benchmarking startup time: $name${NC}"

    local total=0
    for ((i=1; i<=iterations; i++)); do
        # Measure startup time (help/version is fast to complete)
        local start=$(python3 -c 'import time; print(int(time.time() * 1000000))')
        $cmd --version > /dev/null 2>&1 || true
        local end=$(python3 -c 'import time; print(int(time.time() * 1000000))')
        local elapsed=$((end - start))
        total=$((total + elapsed))
    done

    local avg=$((total / iterations))
    echo -e "  Average: ${GREEN}${avg} µs${NC} (${iterations} iterations)"
    echo "$avg"
}

# Benchmark correction with a command
benchmark_correction() {
    local name=$1
    local bin=$2
    local test_cmd=$3
    local iterations=$4

    echo -e "${BLUE}Benchmarking correction ($name): $test_cmd${NC}"

    # Set up environment to skip actual command execution
    export TF_HISTORY="$test_cmd"

    local total=0
    for ((i=1; i<=iterations; i++)); do
        local start=$(python3 -c 'import time; print(int(time.time() * 1000000))')
        if [[ "$name" == "rust" ]]; then
            $bin -y -f "$test_cmd" > /dev/null 2>&1 || true
        else
            # Python version - use THEFUCK_REQUIRE_CONFIRMATION=false
            THEFUCK_REQUIRE_CONFIRMATION=false $bin -y "$test_cmd" > /dev/null 2>&1 || true
        fi
        local end=$(python3 -c 'import time; print(int(time.time() * 1000000))')
        local elapsed=$((end - start))
        total=$((total + elapsed))
    done

    unset TF_HISTORY

    local avg=$((total / iterations))
    echo -e "  Average: ${GREEN}${avg} µs${NC} (${iterations} iterations)"
    echo "$avg"
}

# Results storage
RESULTS_FILE="$RESULTS_DIR/comparison_$(date +%Y%m%d_%H%M%S).txt"
echo "TheFuck Performance Comparison Results" > "$RESULTS_FILE"
echo "Date: $(date)" >> "$RESULTS_FILE"
echo "Iterations: $ITERATIONS" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}                    STARTUP TIME${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Benchmark startup times
echo "STARTUP TIME" >> "$RESULTS_FILE"

RUST_STARTUP=$(benchmark_startup "Rust" "$RUST_BIN" $ITERATIONS)
echo "Rust: $RUST_STARTUP µs" >> "$RESULTS_FILE"

if [[ -n "$PYTHON_BIN" ]]; then
    PYTHON_STARTUP=$(benchmark_startup "Python" "$PYTHON_BIN" $((ITERATIONS / 10)))
    echo "Python: $PYTHON_STARTUP µs" >> "$RESULTS_FILE"

    # Calculate speedup
    if [[ $RUST_STARTUP -gt 0 ]]; then
        SPEEDUP=$(python3 -c "print(f'{$PYTHON_STARTUP / $RUST_STARTUP:.1f}')")
        echo -e "\n${GREEN}Rust is ${SPEEDUP}x faster for startup${NC}"
        echo "Speedup: ${SPEEDUP}x" >> "$RESULTS_FILE"
    fi
fi
echo ""

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}                 CORRECTION TIME${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "" >> "$RESULTS_FILE"
echo "CORRECTION TIME" >> "$RESULTS_FILE"

for test_name in "${!TEST_COMMANDS[@]}"; do
    test_cmd="${TEST_COMMANDS[$test_name]}"
    echo -e "${BLUE}Test case: $test_name${NC}"
    echo "" >> "$RESULTS_FILE"
    echo "Test: $test_name ($test_cmd)" >> "$RESULTS_FILE"

    RUST_TIME=$(benchmark_correction "rust" "$RUST_BIN" "$test_cmd" $ITERATIONS)
    echo "  Rust: $RUST_TIME µs" >> "$RESULTS_FILE"

    if [[ -n "$PYTHON_BIN" ]]; then
        PYTHON_TIME=$(benchmark_correction "python" "$PYTHON_BIN" "$test_cmd" $((ITERATIONS / 10)))
        echo "  Python: $PYTHON_TIME µs" >> "$RESULTS_FILE"

        if [[ $RUST_TIME -gt 0 ]]; then
            SPEEDUP=$(python3 -c "print(f'{$PYTHON_TIME / $RUST_TIME:.1f}')")
            echo -e "  ${GREEN}Rust is ${SPEEDUP}x faster${NC}"
            echo "  Speedup: ${SPEEDUP}x" >> "$RESULTS_FILE"
        fi
    fi
    echo ""
done

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}                 MEMORY USAGE${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo "" >> "$RESULTS_FILE"
echo "MEMORY USAGE" >> "$RESULTS_FILE"

# Measure peak memory usage (macOS-specific using /usr/bin/time)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}Measuring peak memory (macOS)...${NC}"

    # Rust
    RUST_MEM=$(/usr/bin/time -l "$RUST_BIN" --version 2>&1 | grep "maximum resident" | awk '{print $1}')
    RUST_MEM_MB=$(python3 -c "print(f'{$RUST_MEM / 1024 / 1024:.2f}')")
    echo -e "Rust peak memory: ${GREEN}${RUST_MEM_MB} MB${NC}"
    echo "Rust: $RUST_MEM_MB MB" >> "$RESULTS_FILE"

    if [[ -n "$PYTHON_BIN" ]]; then
        PYTHON_MEM=$(/usr/bin/time -l "$PYTHON_BIN" --version 2>&1 | grep "maximum resident" | awk '{print $1}')
        PYTHON_MEM_MB=$(python3 -c "print(f'{$PYTHON_MEM / 1024 / 1024:.2f}')")
        echo -e "Python peak memory: ${GREEN}${PYTHON_MEM_MB} MB${NC}"
        echo "Python: $PYTHON_MEM_MB MB" >> "$RESULTS_FILE"

        RATIO=$(python3 -c "print(f'{$PYTHON_MEM / $RUST_MEM:.1f}')")
        echo -e "\n${GREEN}Rust uses ${RATIO}x less memory${NC}"
        echo "Memory ratio: ${RATIO}x" >> "$RESULTS_FILE"
    fi
elif command -v /usr/bin/time &> /dev/null; then
    echo -e "${BLUE}Measuring peak memory (Linux)...${NC}"

    # Rust
    RUST_MEM=$(/usr/bin/time -v "$RUST_BIN" --version 2>&1 | grep "Maximum resident" | awk '{print $NF}')
    RUST_MEM_MB=$(python3 -c "print(f'{$RUST_MEM / 1024:.2f}')")
    echo -e "Rust peak memory: ${GREEN}${RUST_MEM_MB} MB${NC}"
    echo "Rust: $RUST_MEM_MB MB" >> "$RESULTS_FILE"

    if [[ -n "$PYTHON_BIN" ]]; then
        PYTHON_MEM=$(/usr/bin/time -v "$PYTHON_BIN" --version 2>&1 | grep "Maximum resident" | awk '{print $NF}')
        PYTHON_MEM_MB=$(python3 -c "print(f'{$PYTHON_MEM / 1024:.2f}')")
        echo -e "Python peak memory: ${GREEN}${PYTHON_MEM_MB} MB${NC}"
        echo "Python: $PYTHON_MEM_MB MB" >> "$RESULTS_FILE"

        RATIO=$(python3 -c "print(f'{$PYTHON_MEM / $RUST_MEM:.1f}')")
        echo -e "\n${GREEN}Rust uses ${RATIO}x less memory${NC}"
        echo "Memory ratio: ${RATIO}x" >> "$RESULTS_FILE"
    fi
fi

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}                    SUMMARY${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Results saved to: ${BLUE}$RESULTS_FILE${NC}"
echo ""

# Also run criterion benchmarks
echo -e "${GREEN}Running Criterion micro-benchmarks...${NC}"
cd "$SCRIPT_DIR"
cargo bench --quiet 2>/dev/null || echo "Criterion benchmarks skipped (run 'cargo bench' manually)"

echo ""
echo -e "${GREEN}Benchmark comparison complete!${NC}"
