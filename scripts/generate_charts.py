#!/usr/bin/env python3
"""
Generate performance comparison charts for thefuck-rs vs thefuck.

Usage:
    python3 generate_charts.py [results_file.txt] [output_dir]

If no arguments provided, uses default benchmark_results directory.
"""

import sys
import os
import re
from pathlib import Path

# Try to import matplotlib, provide helpful error if missing
try:
    import matplotlib.pyplot as plt
    import matplotlib.patches as mpatches
except ImportError:
    print("matplotlib not found. Install with: pip install matplotlib")
    print("Generating text-based charts instead...")
    plt = None

def parse_results(filepath: str) -> dict:
    """Parse benchmark results from text file."""
    results = {
        "startup": {},
        "correction": {},
        "memory": {},
        "metadata": {}
    }

    current_section = None

    with open(filepath, 'r') as f:
        lines = f.readlines()

    for line in lines:
        line = line.strip()

        if line.startswith("Date:"):
            results["metadata"]["date"] = line.split(":", 1)[1].strip()
        elif line.startswith("Iterations:"):
            results["metadata"]["iterations"] = line.split(":", 1)[1].strip()
        elif line == "STARTUP TIME":
            current_section = "startup"
        elif line == "CORRECTION TIME":
            current_section = "correction"
        elif line == "MEMORY USAGE":
            current_section = "memory"
        elif line.startswith("Test:"):
            match = re.match(r"Test: (\w+)", line)
            if match:
                current_test = match.group(1)
        elif current_section:
            if line.startswith("Rust:"):
                value = line.split(":")[1].strip().split()[0]
                if current_section == "startup":
                    results["startup"]["rust"] = float(value)
                elif current_section == "correction":
                    if "corrections" not in results["correction"]:
                        results["correction"]["corrections"] = {}
                    results["correction"]["corrections"].setdefault(current_test, {})["rust"] = float(value)
                elif current_section == "memory":
                    results["memory"]["rust"] = float(value)
            elif line.startswith("Python:"):
                value = line.split(":")[1].strip().split()[0]
                if current_section == "startup":
                    results["startup"]["python"] = float(value)
                elif current_section == "correction":
                    if "corrections" not in results["correction"]:
                        results["correction"]["corrections"] = {}
                    results["correction"]["corrections"].setdefault(current_test, {})["python"] = float(value)
                elif current_section == "memory":
                    results["memory"]["python"] = float(value)
            elif "Rust:" in line:
                value = line.split("Rust:")[1].strip().split()[0]
                if "corrections" not in results["correction"]:
                    results["correction"]["corrections"] = {}
                results["correction"]["corrections"].setdefault(current_test, {})["rust"] = float(value)
            elif "Python:" in line:
                value = line.split("Python:")[1].strip().split()[0]
                if "corrections" not in results["correction"]:
                    results["correction"]["corrections"] = {}
                results["correction"]["corrections"].setdefault(current_test, {})["python"] = float(value)

    return results


def generate_matplotlib_charts(results: dict, output_dir: str):
    """Generate charts using matplotlib."""
    os.makedirs(output_dir, exist_ok=True)

    # Color scheme
    rust_color = '#DEA584'  # Rust orange
    python_color = '#3776AB'  # Python blue

    # 1. Startup Time Chart
    fig, ax = plt.subplots(figsize=(10, 6))

    if results["startup"]:
        langs = []
        times = []
        colors = []

        if "rust" in results["startup"]:
            langs.append("Rust\n(thefuck-rs)")
            times.append(results["startup"]["rust"] / 1000)  # Convert to ms
            colors.append(rust_color)

        if "python" in results["startup"]:
            langs.append("Python\n(thefuck)")
            times.append(results["startup"]["python"] / 1000)  # Convert to ms
            colors.append(python_color)

        bars = ax.bar(langs, times, color=colors, edgecolor='black', linewidth=1.2)

        for bar, time in zip(bars, times):
            height = bar.get_height()
            ax.annotate(f'{time:.1f} ms',
                       xy=(bar.get_x() + bar.get_width() / 2, height),
                       xytext=(0, 3),
                       textcoords="offset points",
                       ha='center', va='bottom', fontsize=12, fontweight='bold')

        ax.set_ylabel('Time (milliseconds)', fontsize=12)
        ax.set_title('Startup Time Comparison', fontsize=14, fontweight='bold')
        ax.set_ylim(0, max(times) * 1.2)

        # Add speedup annotation if both values exist
        if len(times) == 2 and times[0] > 0:
            speedup = times[1] / times[0]
            ax.annotate(f'{speedup:.1f}x faster',
                       xy=(0.5, 0.95), xycoords='axes fraction',
                       ha='center', fontsize=14, fontweight='bold',
                       color='green',
                       bbox=dict(boxstyle='round', facecolor='lightgreen', alpha=0.8))

    plt.tight_layout()
    plt.savefig(os.path.join(output_dir, 'startup_time.png'), dpi=150)
    plt.close()

    # 2. Correction Time Chart (grouped bar chart)
    if results["correction"].get("corrections"):
        fig, ax = plt.subplots(figsize=(12, 6))

        corrections = results["correction"]["corrections"]
        tests = list(corrections.keys())
        x = range(len(tests))
        width = 0.35

        rust_times = [corrections[t].get("rust", 0) / 1000 for t in tests]
        python_times = [corrections[t].get("python", 0) / 1000 for t in tests]

        bars1 = ax.bar([i - width/2 for i in x], rust_times, width,
                       label='Rust (thefuck-rs)', color=rust_color, edgecolor='black')
        bars2 = ax.bar([i + width/2 for i in x], python_times, width,
                       label='Python (thefuck)', color=python_color, edgecolor='black')

        # Add value labels
        for bar in bars1:
            height = bar.get_height()
            if height > 0:
                ax.annotate(f'{height:.1f}',
                           xy=(bar.get_x() + bar.get_width() / 2, height),
                           xytext=(0, 3), textcoords="offset points",
                           ha='center', va='bottom', fontsize=9)

        for bar in bars2:
            height = bar.get_height()
            if height > 0:
                ax.annotate(f'{height:.1f}',
                           xy=(bar.get_x() + bar.get_width() / 2, height),
                           xytext=(0, 3), textcoords="offset points",
                           ha='center', va='bottom', fontsize=9)

        ax.set_xlabel('Test Case', fontsize=12)
        ax.set_ylabel('Time (milliseconds)', fontsize=12)
        ax.set_title('Correction Time by Test Case', fontsize=14, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels(tests)
        ax.legend()

        plt.tight_layout()
        plt.savefig(os.path.join(output_dir, 'correction_time.png'), dpi=150)
        plt.close()

    # 3. Memory Usage Chart
    if results["memory"]:
        fig, ax = plt.subplots(figsize=(10, 6))

        langs = []
        memory = []
        colors = []

        if "rust" in results["memory"]:
            langs.append("Rust\n(thefuck-rs)")
            memory.append(results["memory"]["rust"])
            colors.append(rust_color)

        if "python" in results["memory"]:
            langs.append("Python\n(thefuck)")
            memory.append(results["memory"]["python"])
            colors.append(python_color)

        bars = ax.bar(langs, memory, color=colors, edgecolor='black', linewidth=1.2)

        for bar, mem in zip(bars, memory):
            height = bar.get_height()
            ax.annotate(f'{mem:.1f} MB',
                       xy=(bar.get_x() + bar.get_width() / 2, height),
                       xytext=(0, 3),
                       textcoords="offset points",
                       ha='center', va='bottom', fontsize=12, fontweight='bold')

        ax.set_ylabel('Peak Memory (MB)', fontsize=12)
        ax.set_title('Memory Usage Comparison', fontsize=14, fontweight='bold')
        ax.set_ylim(0, max(memory) * 1.2)

        # Add memory reduction annotation if both values exist
        if len(memory) == 2 and memory[0] > 0:
            reduction = memory[1] / memory[0]
            ax.annotate(f'{reduction:.1f}x less memory',
                       xy=(0.5, 0.95), xycoords='axes fraction',
                       ha='center', fontsize=14, fontweight='bold',
                       color='green',
                       bbox=dict(boxstyle='round', facecolor='lightgreen', alpha=0.8))

        plt.tight_layout()
        plt.savefig(os.path.join(output_dir, 'memory_usage.png'), dpi=150)
        plt.close()

    # 4. Summary Chart (radar/spider chart)
    fig, ax = plt.subplots(figsize=(10, 6), subplot_kw=dict(projection='polar'))

    # Calculate speedups for each metric
    categories = []
    speedups = []

    if "rust" in results["startup"] and "python" in results["startup"]:
        categories.append('Startup')
        speedups.append(results["startup"]["python"] / results["startup"]["rust"])

    if results["correction"].get("corrections"):
        for test, data in results["correction"]["corrections"].items():
            if "rust" in data and "python" in data and data["rust"] > 0:
                categories.append(f'Correction\n({test})')
                speedups.append(data["python"] / data["rust"])

    if "rust" in results["memory"] and "python" in results["memory"] and results["memory"]["rust"] > 0:
        categories.append('Memory\n(lower is better)')
        speedups.append(results["memory"]["python"] / results["memory"]["rust"])

    if categories:
        # Create radar chart
        angles = [n / float(len(categories)) * 2 * 3.14159 for n in range(len(categories))]
        angles += angles[:1]  # Complete the loop
        speedups += speedups[:1]

        ax.plot(angles, speedups, 'o-', linewidth=2, color=rust_color)
        ax.fill(angles, speedups, alpha=0.25, color=rust_color)
        ax.set_xticks(angles[:-1])
        ax.set_xticklabels(categories)
        ax.set_title('Rust Performance Improvement (x times faster/smaller)',
                    fontsize=14, fontweight='bold', pad=20)

        # Add 1x reference line
        ax.axhline(y=1, color='gray', linestyle='--', alpha=0.5)

        plt.tight_layout()
        plt.savefig(os.path.join(output_dir, 'summary_radar.png'), dpi=150)
        plt.close()

    print(f"Charts saved to {output_dir}/")


def generate_text_charts(results: dict, output_dir: str):
    """Generate ASCII text charts when matplotlib is not available."""
    os.makedirs(output_dir, exist_ok=True)
    output_file = os.path.join(output_dir, 'charts.txt')

    with open(output_file, 'w') as f:
        f.write("=" * 60 + "\n")
        f.write("       TheFuck Performance Comparison Charts\n")
        f.write("=" * 60 + "\n\n")

        # Startup Time
        f.write("STARTUP TIME (lower is better)\n")
        f.write("-" * 40 + "\n")
        if results["startup"]:
            max_time = max(results["startup"].values())
            for lang, time in results["startup"].items():
                bar_len = int(time / max_time * 30) if max_time > 0 else 0
                bar = "█" * bar_len
                f.write(f"{lang:8} {bar:30} {time/1000:.1f} ms\n")

            if "rust" in results["startup"] and "python" in results["startup"]:
                speedup = results["startup"]["python"] / results["startup"]["rust"]
                f.write(f"\n→ Rust is {speedup:.1f}x faster\n")

        f.write("\n")

        # Correction Time
        f.write("CORRECTION TIME (lower is better)\n")
        f.write("-" * 40 + "\n")
        if results["correction"].get("corrections"):
            for test, data in results["correction"]["corrections"].items():
                f.write(f"\n{test}:\n")
                max_time = max(data.values()) if data else 1
                for lang, time in data.items():
                    bar_len = int(time / max_time * 25) if max_time > 0 else 0
                    bar = "█" * bar_len
                    f.write(f"  {lang:8} {bar:25} {time/1000:.1f} ms\n")

                if "rust" in data and "python" in data and data["rust"] > 0:
                    speedup = data["python"] / data["rust"]
                    f.write(f"  → Rust is {speedup:.1f}x faster\n")

        f.write("\n")

        # Memory Usage
        f.write("MEMORY USAGE (lower is better)\n")
        f.write("-" * 40 + "\n")
        if results["memory"]:
            max_mem = max(results["memory"].values())
            for lang, mem in results["memory"].items():
                bar_len = int(mem / max_mem * 30) if max_mem > 0 else 0
                bar = "█" * bar_len
                f.write(f"{lang:8} {bar:30} {mem:.1f} MB\n")

            if "rust" in results["memory"] and "python" in results["memory"]:
                ratio = results["memory"]["python"] / results["memory"]["rust"]
                f.write(f"\n→ Rust uses {ratio:.1f}x less memory\n")

        f.write("\n" + "=" * 60 + "\n")

    print(f"Text charts saved to {output_file}")


def main():
    # Determine paths
    script_dir = Path(__file__).parent.parent
    default_results_dir = script_dir / "benchmark_results"

    if len(sys.argv) >= 2:
        results_file = sys.argv[1]
    else:
        # Find the most recent results file
        if default_results_dir.exists():
            results_files = sorted(default_results_dir.glob("comparison_*.txt"))
            if results_files:
                results_file = str(results_files[-1])
            else:
                print("No results files found. Run benchmark_comparison.sh first.")
                sys.exit(1)
        else:
            print("Results directory not found. Run benchmark_comparison.sh first.")
            sys.exit(1)

    output_dir = sys.argv[2] if len(sys.argv) >= 3 else str(script_dir / "benchmark_charts")

    print(f"Parsing results from: {results_file}")
    results = parse_results(results_file)

    if plt:
        generate_matplotlib_charts(results, output_dir)
    else:
        generate_text_charts(results, output_dir)


if __name__ == "__main__":
    main()
