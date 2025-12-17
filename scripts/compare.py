#!/usr/bin/env python3
# /// script
# dependencies = ["matplotlib"]
# ///
"""
Generate comparison charts from Criterion benchmark output.

Usage:
    uv run scripts/compare.py

Reads benchmark data from target/criterion/ and generates comparison charts.
"""

import json
import platform
import subprocess
from pathlib import Path
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np


def get_system_info() -> dict:
    """Gather system information for benchmark context."""
    info = {
        "os": platform.system(),
        "os_version": platform.version(),
        "arch": platform.machine(),
        "python": platform.python_version(),
    }

    # Get CPU info
    if platform.system() == "Darwin":
        # Use system_profiler JSON for reliable info on macOS
        try:
            result = subprocess.run(
                ["system_profiler", "SPHardwareDataType", "-json"],
                capture_output=True, text=True
            )
            hw_data = json.loads(result.stdout)["SPHardwareDataType"][0]

            # Get chip/CPU info
            if "chip_type" in hw_data:
                # Apple Silicon
                info["cpu"] = hw_data["chip_type"]
                info["cpu_freq"] = "dynamic"
            else:
                # Intel Mac
                info["cpu"] = hw_data.get("cpu_type", platform.processor())
                info["cpu_freq"] = hw_data.get("current_processor_speed", "unknown")

            info["machine"] = f"{hw_data.get('machine_name', '')} ({hw_data.get('machine_model', '')})"
            info["memory"] = hw_data.get("physical_memory", "unknown")
        except Exception:
            info["cpu"] = platform.processor()
            info["cpu_freq"] = "unknown"
            info["machine"] = "unknown"
            info["memory"] = "unknown"

    elif platform.system() == "Linux":
        try:
            with open("/proc/cpuinfo") as f:
                for line in f:
                    if line.startswith("model name"):
                        info["cpu"] = line.split(":")[1].strip()
                    if line.startswith("cpu MHz"):
                        freq_mhz = float(line.split(":")[1].strip())
                        info["cpu_freq"] = f"{freq_mhz / 1000:.2f} GHz"
                        break
        except Exception:
            info["cpu"] = platform.processor()
            info["cpu_freq"] = "unknown"

        # Get memory on Linux
        try:
            with open("/proc/meminfo") as f:
                for line in f:
                    if line.startswith("MemTotal"):
                        mem_kb = int(line.split()[1])
                        info["memory"] = f"{mem_kb // (1024 * 1024)} GB"
                        break
        except Exception:
            info["memory"] = "unknown"
        info["machine"] = platform.node()

    else:
        info["cpu"] = platform.processor()
        info["cpu_freq"] = "unknown"
        info["memory"] = "unknown"
        info["machine"] = platform.node()

    # Get Rust version
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True)
        info["rustc"] = result.stdout.strip()
    except Exception:
        info["rustc"] = "unknown"

    return info


def parse_criterion_output(criterion_dir: Path) -> dict:
    """Parse Criterion benchmark output into structured data.

    Returns dict of:
        {type: {(operation, value): {"fastvint": time_ns, "leb128": time_ns}}}
    """
    results = defaultdict(lambda: defaultdict(dict))

    for type_dir in criterion_dir.iterdir():
        if not type_dir.is_dir() or type_dir.name in ("report",):
            continue

        type_name = type_dir.name

        for bench_dir in type_dir.iterdir():
            if not bench_dir.is_dir():
                continue

            # Parse benchmark name: "encode_fastvint" or "decode_leb128"
            bench_name = bench_dir.name
            if "_fastvint" in bench_name:
                impl = "fastvint"
                operation = bench_name.replace("_fastvint", "")
            elif "_leb128" in bench_name:
                impl = "leb128"
                operation = bench_name.replace("_leb128", "")
            else:
                continue

            for value_dir in bench_dir.iterdir():
                if not value_dir.is_dir():
                    continue

                value_name = value_dir.name
                estimates_file = value_dir / "new" / "estimates.json"

                if not estimates_file.exists():
                    continue

                with open(estimates_file) as f:
                    data = json.load(f)

                # Get mean time in nanoseconds
                time_ns = data["mean"]["point_estimate"]

                results[type_name][(operation, value_name)][impl] = time_ns

    return results


def generate_chart(type_name: str, benchmarks: dict, output_dir: Path) -> Path:
    """Generate comparison line charts for a single type (one for encode, one for decode)."""

    # Group by operation
    encode_data = {}
    decode_data = {}

    for (operation, value), times in benchmarks.items():
        if "fastvint" not in times or "leb128" not in times:
            continue

        if operation == "encode":
            encode_data[value] = times
        elif operation == "decode":
            decode_data[value] = times

    if not encode_data and not decode_data:
        return None

    # Create figure with two subplots
    fig, axes = plt.subplots(1, 2, figsize=(14, 5))

    for ax, (op_name, data) in zip(axes, [("encode", encode_data), ("decode", decode_data)]):
        if not data:
            ax.set_visible(False)
            continue

        # Average pos/neg variants for signed types
        averaged_data = {}
        for value, times in data.items():
            # Strip _pos/_neg suffix and average
            base = value.replace("_pos", "").replace("_neg", "")
            if base not in averaged_data:
                averaged_data[base] = {"fastvint": [], "leb128": []}
            averaged_data[base]["fastvint"].append(times["fastvint"])
            averaged_data[base]["leb128"].append(times["leb128"])

        # Compute averages
        data = {
            k: {"fastvint": sum(v["fastvint"]) / len(v["fastvint"]),
                "leb128": sum(v["leb128"]) / len(v["leb128"])}
            for k, v in averaged_data.items()
        }

        # Sort values by byte size
        value_order = [f"{i}_byte" for i in range(1, 18)]
        sorted_values = sorted(data.keys(), key=lambda v: value_order.index(v) if v in value_order else 999)

        x = range(len(sorted_values))
        fastvint_times = [data[v]["fastvint"] for v in sorted_values]
        leb128_times = [data[v]["leb128"] for v in sorted_values]

        ax.plot(x, fastvint_times, 'o-', color="#2ecc71", linewidth=2, markersize=8, label="fastvint")
        ax.plot(x, leb128_times, 's-', color="#3498db", linewidth=2, markersize=8, label="leb128")

        # Add time annotations below each point
        for i, val in enumerate(sorted_values):
            fv_time = fastvint_times[i]
            lb_time = leb128_times[i]
            # Annotate fastvint time below its point
            ax.annotate(f"{fv_time:.1f}", xy=(i, fv_time), ha="center", va="top",
                       fontsize=7, color="#27ae60", fontweight="bold",
                       xytext=(0, -8), textcoords="offset points")
            # Annotate leb128 time below its point
            ax.annotate(f"{lb_time:.1f}", xy=(i, lb_time), ha="center", va="top",
                       fontsize=7, color="#2980b9", fontweight="bold",
                       xytext=(0, -8), textcoords="offset points")

        ax.set_ylabel("Time (ns)")
        ax.set_title(f"{type_name} {op_name}")
        ax.set_xticks(x)
        ax.set_xticklabels(sorted_values, rotation=45, ha="right")
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_ylim(bottom=0)

    plt.tight_layout()

    output_path = output_dir / f"{type_name}.svg"
    plt.savefig(output_path, format='svg')
    plt.close()

    return output_path


def generate_html(chart_paths: list[Path], output_dir: Path, system_info: dict):
    """Generate HTML page embedding all charts."""

    html = f"""<!DOCTYPE html>
<html>
<head>
    <title>fastvint vs leb128 Benchmark Comparison</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{ color: #333; }}
        .system-info {{
            background: white;
            padding: 15px 20px;
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            font-size: 14px;
            color: #555;
        }}
        .system-info h2 {{
            margin: 0 0 10px 0;
            font-size: 16px;
            color: #333;
        }}
        .system-info dl {{
            display: grid;
            grid-template-columns: auto 1fr;
            gap: 5px 15px;
            margin: 0;
        }}
        .system-info dt {{ font-weight: 600; color: #666; }}
        .system-info dd {{ margin: 0; }}
        .chart {{
            background: white;
            padding: 20px;
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .chart img {{
            max-width: 100%;
            height: auto;
        }}
    </style>
</head>
<body>
    <h1>fastvint vs leb128 Benchmark Comparison</h1>
    <div class="system-info">
        <h2>System Information</h2>
        <dl>
            <dt>Machine</dt><dd>{system_info['machine']}</dd>
            <dt>CPU</dt><dd>{system_info['cpu']}</dd>
            <dt>CPU Speed</dt><dd>{system_info['cpu_freq']}</dd>
            <dt>Memory</dt><dd>{system_info['memory']}</dd>
            <dt>Architecture</dt><dd>{system_info['arch']}</dd>
            <dt>OS</dt><dd>{system_info['os']}</dd>
            <dt>Rust</dt><dd>{system_info['rustc']}</dd>
        </dl>
    </div>
"""

    type_order = ["u32", "i32", "u64", "i64", "u128", "i128"]
    chart_paths.sort(key=lambda p: type_order.index(p.stem) if p.stem in type_order else 999)
    for path in chart_paths:
        html += f"""    <div class="chart">
        <img src="{path.name}" alt="{path.stem} comparison">
    </div>
"""

    html += """</body>
</html>
"""

    output_path = output_dir / "comparison.html"
    with open(output_path, "w") as f:
        f.write(html)

    return output_path


def generate_summary(results: dict, output_dir: Path):
    """Generate a JSON summary of benchmark comparisons."""
    summary = {}

    for type_name, benchmarks in sorted(results.items()):
        summary[type_name] = {}
        for (operation, value), times in sorted(benchmarks.items()):
            if "fastvint" in times and "leb128" in times:
                fv = times["fastvint"]
                lb = times["leb128"]
                speedup = lb / fv
                key = f"{operation}/{value}"
                summary[type_name][key] = {
                    "fastvint_ns": round(fv, 2),
                    "leb128_ns": round(lb, 2),
                    "speedup": round(speedup, 2),
                    "winner": "fastvint" if speedup > 1 else "leb128"
                }

    output_path = output_dir / "summary.json"
    with open(output_path, "w") as f:
        json.dump(summary, f, indent=2)

    return output_path


def main():
    criterion_dir = Path("target/criterion")

    if not criterion_dir.exists():
        print("Error: target/criterion not found. Run benchmarks first:")
        print("  cargo bench --features asm,bench --bench comparison")
        return 1

    print("Parsing Criterion output...")
    results = parse_criterion_output(criterion_dir)

    if not results:
        print("Error: No benchmark results found")
        return 1

    print(f"Found benchmarks for: {', '.join(sorted(results.keys()))}")

    # Generate JSON summary
    summary_path = generate_summary(results, criterion_dir)
    print(f"Generated summary: {summary_path}")

    chart_paths = []
    for type_name, benchmarks in sorted(results.items()):
        print(f"Generating chart for {type_name}...")
        path = generate_chart(type_name, benchmarks, criterion_dir)
        if path:
            chart_paths.append(path)

    print("Gathering system info...")
    system_info = get_system_info()

    print("Generating HTML...")
    html_path = generate_html(chart_paths, criterion_dir, system_info)

    print(f"\nDone! Open {html_path}")
    return 0


if __name__ == "__main__":
    exit(main())
