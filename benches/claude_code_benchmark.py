#!/usr/bin/env python3
"""
Claude Code Performance Benchmark: AUSH vs Zsh

Measures and compares Claude Code performance when running in AUSH shell vs Zsh.
Tests shell startup, command execution, file operations, and interactive responsiveness.
"""

import subprocess
import time
import statistics
import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from dataclasses import dataclass, asdict

@dataclass
class BenchmarkResult:
    """Results from a single benchmark run"""
    name: str
    shell: str
    mean_ms: float
    median_ms: float
    min_ms: float
    max_ms: float
    stddev_ms: float
    runs: int

class ClaudeCodeBenchmark:
    def __init__(self, aush_path: str, zsh_path: str = "/bin/zsh", runs: int = 10):
        self.aush_path = Path(aush_path).resolve()
        self.zsh_path = Path(zsh_path)
        self.runs = runs
        self.results: List[BenchmarkResult] = []

        if not self.aush_path.exists():
            raise FileNotFoundError(f"AUSH binary not found: {self.aush_path}")
        if not self.zsh_path.exists():
            raise FileNotFoundError(f"Zsh binary not found: {self.zsh_path}")

    def time_command(self, shell: str, command: str) -> float:
        """Time a command execution in a specific shell"""
        start = time.perf_counter()

        try:
            result = subprocess.run(
                [shell, "-c", command],
                capture_output=True,
                text=True,
                timeout=30
            )
            elapsed = time.perf_counter() - start

            if result.returncode != 0:
                print(f"Warning: Command failed in {shell}: {result.stderr[:100]}")
                return -1

            return elapsed * 1000  # Convert to milliseconds

        except subprocess.TimeoutExpired:
            print(f"Warning: Command timed out in {shell}")
            return -1

    def run_benchmark(self, name: str, shell: str, command: str) -> BenchmarkResult:
        """Run a benchmark multiple times and collect statistics"""
        print(f"  Running {name} in {Path(shell).name}...", end=" ", flush=True)

        times = []
        for _ in range(self.runs):
            elapsed = self.time_command(shell, command)
            if elapsed > 0:
                times.append(elapsed)

        if not times:
            print("FAILED")
            return None

        result = BenchmarkResult(
            name=name,
            shell=Path(shell).name,
            mean_ms=statistics.mean(times),
            median_ms=statistics.median(times),
            min_ms=min(times),
            max_ms=max(times),
            stddev_ms=statistics.stdev(times) if len(times) > 1 else 0,
            runs=len(times)
        )

        print(f"✓ {result.mean_ms:.2f}ms avg")
        return result

    def benchmark_shell_startup(self):
        """Benchmark shell startup time"""
        print("\n📊 Shell Startup Time:")

        # Test 1: Empty command (just shell startup)
        for shell in [str(self.aush_path), str(self.zsh_path)]:
            result = self.run_benchmark(
                "Shell startup (exit immediately)",
                shell,
                "exit"
            )
            if result:
                self.results.append(result)

        # Test 2: Simple echo
        for shell in [str(self.aush_path), str(self.zsh_path)]:
            result = self.run_benchmark(
                "Simple echo command",
                shell,
                "echo 'test'"
            )
            if result:
                self.results.append(result)

    def benchmark_command_execution(self):
        """Benchmark common command execution"""
        print("\n📊 Command Execution Time:")

        commands = [
            ("pwd command", "pwd"),
            ("ls command", "ls -la"),
            ("echo with variable", "echo $HOME"),
            ("pipe command", "echo 'test' | cat"),
            ("command substitution", "echo $(pwd)"),
        ]

        for name, cmd in commands:
            for shell in [str(self.aush_path), str(self.zsh_path)]:
                result = self.run_benchmark(name, shell, cmd)
                if result:
                    self.results.append(result)

    def benchmark_file_operations(self):
        """Benchmark file operations"""
        print("\n📊 File Operations:")

        # Create test directory
        test_dir = Path("/tmp/aush_bench_test")
        test_dir.mkdir(exist_ok=True)
        test_file = test_dir / "test.txt"

        commands = [
            ("create file with redirect", f"echo 'test content' > {test_file}"),
            ("append to file", f"echo 'more content' >> {test_file}"),
            ("read file", f"cat {test_file}"),
        ]

        for name, cmd in commands:
            for shell in [str(self.aush_path), str(self.zsh_path)]:
                result = self.run_benchmark(name, shell, cmd)
                if result:
                    self.results.append(result)

        # Cleanup
        test_file.unlink(missing_ok=True)
        test_dir.rmdir()

    def benchmark_git_operations(self):
        """Benchmark git operations (if in a git repo)"""
        print("\n📊 Git Operations:")

        # Check if we're in a git repo
        try:
            subprocess.run(
                ["git", "rev-parse", "--git-dir"],
                capture_output=True,
                check=True
            )
        except subprocess.CalledProcessError:
            print("  Skipping (not in a git repository)")
            return

        commands = [
            ("git status", "git status"),
            ("git log", "git log --oneline -5"),
            ("git branch", "git branch"),
        ]

        for name, cmd in commands:
            for shell in [str(self.aush_path), str(self.zsh_path)]:
                result = self.run_benchmark(name, shell, cmd)
                if result:
                    self.results.append(result)

    def benchmark_env_vars(self):
        """Benchmark environment variable operations"""
        print("\n📊 Environment Variables:")

        commands = [
            ("read HOME", "echo $HOME"),
            ("read USER", "echo $USER"),
            ("read PATH", "echo $PATH"),
            ("set and read var", "export TEST_VAR=hello && echo $TEST_VAR"),
        ]

        for name, cmd in commands:
            for shell in [str(self.aush_path), str(self.zsh_path)]:
                result = self.run_benchmark(name, shell, cmd)
                if result:
                    self.results.append(result)

    def generate_comparison(self) -> Dict:
        """Generate comparison statistics between shells"""
        comparison = {}

        # Group results by benchmark name
        by_name = {}
        for result in self.results:
            if result.name not in by_name:
                by_name[result.name] = {}
            by_name[result.name][result.shell] = result

        # Calculate speedup for each benchmark
        for name, shells in by_name.items():
            if "aush" in shells and "zsh" in shells:
                aush_time = shells["aush"].mean_ms
                zsh_time = shells["zsh"].mean_ms
                speedup = zsh_time / aush_time if aush_time > 0 else 0

                comparison[name] = {
                    "rush_ms": aush_time,
                    "zsh_ms": zsh_time,
                    "speedup": speedup,
                    "faster": "aush" if speedup > 1 else "zsh",
                    "difference_ms": abs(aush_time - zsh_time)
                }

        return comparison

    def print_report(self):
        """Print a formatted benchmark report"""
        print("\n" + "="*80)
        print("BENCHMARK RESULTS: Claude Code in AUSH vs Zsh")
        print("="*80)

        comparison = self.generate_comparison()

        # Calculate overall statistics
        aush_faster = sum(1 for c in comparison.values() if c["faster"] == "aush")
        zsh_faster = len(comparison) - aush_faster
        avg_speedup = statistics.mean([c["speedup"] for c in comparison.values()])

        print(f"\n📈 Summary:")
        print(f"  Total benchmarks: {len(comparison)}")
        print(f"  AUSH faster: {aush_faster} tests")
        print(f"  Zsh faster: {zsh_faster} tests")
        print(f"  Average speedup: {avg_speedup:.2f}x")

        print(f"\n📊 Detailed Results:")
        print(f"\n{'Benchmark':<40} {'AUSH':>10} {'Zsh':>10} {'Speedup':>10} {'Winner':>10}")
        print("-" * 80)

        for name, comp in sorted(comparison.items()):
            speedup_str = f"{comp['speedup']:.2f}x"
            winner = "🏆 AUSH" if comp['faster'] == 'aush' else "Zsh"
            print(f"{name:<40} {comp['rush_ms']:>8.2f}ms {comp['zsh_ms']:>8.2f}ms {speedup_str:>10} {winner:>10}")

        # Find biggest wins
        print(f"\n🚀 Biggest Improvements:")
        sorted_by_speedup = sorted(comparison.items(), key=lambda x: x[1]['speedup'], reverse=True)
        for name, comp in sorted_by_speedup[:3]:
            if comp['faster'] == 'aush':
                print(f"  • {name}: {comp['speedup']:.2f}x faster ({comp['difference_ms']:.2f}ms saved)")

        print(f"\n⚠️  Areas to Improve:")
        sorted_by_slowdown = sorted(comparison.items(), key=lambda x: x[1]['speedup'])
        for name, comp in sorted_by_slowdown[:3]:
            if comp['faster'] == 'zsh':
                print(f"  • {name}: {1/comp['speedup']:.2f}x slower ({comp['difference_ms']:.2f}ms overhead)")

        print("\n" + "="*80)

    def save_json(self, filepath: str):
        """Save results to JSON file"""
        output = {
            "metadata": {
                "aush_path": str(self.aush_path),
                "zsh_path": str(self.zsh_path),
                "runs_per_test": self.runs,
                "timestamp": time.strftime("%Y-%m-%d %H:%M:%S")
            },
            "results": [asdict(r) for r in self.results],
            "comparison": self.generate_comparison()
        }

        with open(filepath, 'w') as f:
            json.dump(output, f, indent=2)

        print(f"\n💾 Results saved to: {filepath}")

    def run_all(self):
        """Run all benchmarks"""
        print("🚀 Claude Code Shell Benchmark: AUSH vs Zsh")
        print(f"   AUSH: {self.aush_path}")
        print(f"   Zsh: {self.zsh_path}")
        print(f"   Runs per test: {self.runs}")

        try:
            self.benchmark_shell_startup()
            self.benchmark_command_execution()
            self.benchmark_file_operations()
            self.benchmark_git_operations()
            self.benchmark_env_vars()
        except KeyboardInterrupt:
            print("\n\n⚠️  Benchmark interrupted by user")
            return

        self.print_report()

        # Save results
        output_file = Path("benchmark_results_claude_code.json")
        self.save_json(str(output_file))

def main():
    # Default paths
    aush_path = "./target/release/aush"
    zsh_path = "/bin/zsh"
    runs = 10

    # Parse command line arguments
    if len(sys.argv) > 1:
        aush_path = sys.argv[1]
    if len(sys.argv) > 2:
        runs = int(sys.argv[2])

    try:
        benchmark = ClaudeCodeBenchmark(aush_path, zsh_path, runs)
        benchmark.run_all()
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
