#!/usr/bin/env bash

# generate_benchmark_report.sh - Generate publishable benchmark reports from results
# Converts benchmark JSON results to markdown and HTML formats
# Supports historical comparison and regression detection

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RESULTS_DIR="${RESULTS_DIR:-.}"
RESULTS_FILE="${RESULTS_DIR}/benchmark_results.json"
COMPARISON_RESULTS_FILE="${RESULTS_DIR}/benchmark_comparison.json"
REPORT_DIR="${RESULTS_DIR}/reports"
HISTORICAL_DIR="${RESULTS_DIR}/historical"

# Script directory for Rust utility functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Helper function to print usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Generate publishable benchmark reports from benchmark results.

OPTIONS:
    -h, --help                    Show this help message
    -r, --results FILE            Path to benchmark_results.json (default: $RESULTS_FILE)
    -c, --comparison FILE         Path to benchmark_comparison.json (optional)
    -o, --output DIR              Output directory for reports (default: $REPORT_DIR)
    -s, --historical DIR          Historical results directory (default: $HISTORICAL_DIR)
    --markdown                    Generate markdown report only
    --html                        Generate HTML report only
    --all                         Generate all reports (default)
    --compare PREVIOUS_FILE       Compare with previous benchmark results
    --no-historical               Don't save results to historical directory

EXAMPLES:
    # Generate all reports
    $0

    # Generate only markdown report
    $0 --markdown

    # Use custom results file
    $0 --results /path/to/results.json

    # Compare with previous run
    $0 --compare historical/benchmark_2024-01-20.json

    # Custom output directory
    $0 --output ./my_reports

EOF
    exit "${1:-0}"
}

# Parse command line arguments
GENERATE_MARKDOWN=true
GENERATE_HTML=true
COMPARE_FILE=""
SAVE_HISTORICAL=true

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage 0
            ;;
        -r|--results)
            RESULTS_FILE="$2"
            shift 2
            ;;
        -c|--comparison)
            COMPARISON_RESULTS_FILE="$2"
            shift 2
            ;;
        -o|--output)
            REPORT_DIR="$2"
            shift 2
            ;;
        -s|--historical)
            HISTORICAL_DIR="$2"
            shift 2
            ;;
        --markdown)
            GENERATE_HTML=false
            shift
            ;;
        --html)
            GENERATE_MARKDOWN=false
            shift
            ;;
        --all)
            GENERATE_MARKDOWN=true
            GENERATE_HTML=true
            shift
            ;;
        --compare)
            COMPARE_FILE="$2"
            shift 2
            ;;
        --no-historical)
            SAVE_HISTORICAL=false
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage 1
            ;;
    esac
done

# Verify results file exists
if [ ! -f "$RESULTS_FILE" ]; then
    echo -e "${RED}Error: Results file not found: $RESULTS_FILE${NC}"
    echo "Run benchmarks first with: cargo run --bin aush -- --benchmark quick"
    exit 1
fi

# Create output directories
mkdir -p "$REPORT_DIR"
mkdir -p "$HISTORICAL_DIR"

echo -e "${BLUE}=== Rush Benchmark Report Generator ===${NC}\n"
echo "Results file: $RESULTS_FILE"
echo "Output directory: $REPORT_DIR"
echo ""

# Extract timestamp from results file for naming
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
RESULTS_FILENAME="benchmark_${TIMESTAMP}"

# Generate markdown report
if [ "$GENERATE_MARKDOWN" = true ]; then
    MARKDOWN_OUTPUT="${REPORT_DIR}/${RESULTS_FILENAME}.md"
    echo -e "${BLUE}Generating markdown report...${NC}"

    # Create markdown report using Rust if available, otherwise use shell script
    python3 - <<'PYTHON_EOF' "$RESULTS_FILE" "$MARKDOWN_OUTPUT"
import json
import sys
from datetime import datetime

results_file = sys.argv[1]
output_file = sys.argv[2]

try:
    with open(results_file, 'r') as f:
        data = json.load(f)

    # Build markdown report
    report = f"""# Rush Benchmark Report

**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

## Summary

- **Mode:** {data['mode']}
- **Total Duration:** {data['total_duration_ms']:.2f}ms
- **Tests Passed:** {data['passed']}
- **Tests Failed:** {data['failed']}
- **Total Tests:** {data['passed'] + data['failed']}

## Test Results

| Test | Duration | Status |
|------|----------|--------|
"""

    for test in data['tests']:
        status = "✓ PASS" if test['passed'] else "✗ FAIL"
        report += f"| {test['name']} | {test['duration_ms']:.2f}ms | {status} |\n"

        if test['error']:
            report += f"| | **Error:** {test['error']} | |\n"

    report += """
## Methodology

This benchmark report was generated using the Rush benchmark runner.

### Test Categories

- **Quick Mode:** 5-second smoke test with essential commands
- **Full Mode:** Comprehensive test suite covering shell features
- **Compare Mode:** Comparison benchmarks across shells (Rush, Bash, Zsh)

### Metrics

- **Duration:** Time taken to execute the test in milliseconds
- **Status:** Pass/Fail status of the test

## Notes

- All timings are in milliseconds (ms)
- Durations include lexing, parsing, and execution
- Results may vary based on system load and resources
"""

    with open(output_file, 'w') as f:
        f.write(report)

    print(f"✓ Markdown report: {output_file}")

except Exception as e:
    print(f"Error generating markdown report: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_EOF

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Markdown report generated successfully${NC}"
    else
        echo -e "${YELLOW}Warning: Could not generate markdown report${NC}"
    fi
fi

# Generate HTML report
if [ "$GENERATE_HTML" = true ]; then
    HTML_OUTPUT="${REPORT_DIR}/${RESULTS_FILENAME}.html"
    echo -e "${BLUE}Generating HTML report...${NC}"

    python3 - <<'PYTHON_EOF' "$RESULTS_FILE" "$HTML_OUTPUT"
import json
import sys
from datetime import datetime
import math

results_file = sys.argv[1]
output_file = sys.argv[2]

try:
    with open(results_file, 'r') as f:
        data = json.load(f)

    # Calculate statistics
    durations = [test['duration_ms'] for test in data['tests']]
    avg_duration = sum(durations) / len(durations) if durations else 0
    max_duration = max(durations) if durations else 0

    # Build test data for chart
    test_names = [f'"{test["name"]}"' for test in data['tests']]
    test_durations = [str(test['duration_ms']) for test in data['tests']]

    test_names_json = ','.join(test_names)
    test_durations_json = ','.join(test_durations)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rush Benchmark Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@3.9.1/dist/chart.min.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 2rem;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            overflow: hidden;
        }}

        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem;
        }}

        h1 {{
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
        }}

        .timestamp {{
            opacity: 0.9;
            font-size: 0.95rem;
        }}

        main {{
            padding: 2rem;
        }}

        section {{
            margin-bottom: 3rem;
        }}

        h2 {{
            color: #333;
            border-bottom: 2px solid #667eea;
            padding-bottom: 0.5rem;
            margin-bottom: 1.5rem;
            font-size: 1.8rem;
        }}

        .summary-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }}

        .summary-card {{
            background: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 8px;
            padding: 1.5rem;
            text-align: center;
        }}

        .summary-card .label {{
            color: #6c757d;
            font-size: 0.9rem;
            margin-bottom: 0.5rem;
        }}

        .summary-card .value {{
            font-size: 2rem;
            font-weight: bold;
            color: #667eea;
        }}

        .chart-container {{
            position: relative;
            height: 400px;
            margin-bottom: 3rem;
        }}

        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }}

        thead {{
            background: #f8f9fa;
            border-bottom: 2px solid #dee2e6;
        }}

        th {{
            padding: 1rem;
            text-align: left;
            font-weight: 600;
            color: #333;
        }}

        td {{
            padding: 1rem;
            border-bottom: 1px solid #dee2e6;
        }}

        tr:hover {{
            background: #f8f9fa;
        }}

        .status-pass {{
            color: #28a745;
            font-weight: 600;
        }}

        .status-fail {{
            color: #dc3545;
            font-weight: 600;
        }}

        .methodology {{
            background: #f8f9fa;
            padding: 1.5rem;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}

        .methodology h3 {{
            color: #333;
            margin-top: 1rem;
            margin-bottom: 0.5rem;
        }}

        .methodology h3:first-child {{
            margin-top: 0;
        }}

        ul {{
            margin-left: 2rem;
            color: #555;
            line-height: 1.6;
        }}

        li {{
            margin-bottom: 0.5rem;
        }}

        footer {{
            background: #f8f9fa;
            padding: 1rem 2rem;
            text-align: center;
            color: #6c757d;
            font-size: 0.9rem;
            border-top: 1px solid #dee2e6;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Rush Benchmark Report</h1>
            <div class="timestamp">Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</div>
        </header>

        <main>
            <section>
                <h2>Summary</h2>
                <div class="summary-grid">
                    <div class="summary-card">
                        <div class="label">Mode</div>
                        <div class="value">{data['mode']}</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Total Duration</div>
                        <div class="value">{data['total_duration_ms']:.2f}ms</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Tests Passed</div>
                        <div class="value">{data['passed']}</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Tests Failed</div>
                        <div class="value">{data['failed']}</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Avg Duration</div>
                        <div class="value">{avg_duration:.2f}ms</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Max Duration</div>
                        <div class="value">{max_duration:.2f}ms</div>
                    </div>
                </div>
            </section>

            <section>
                <h2>Performance Chart</h2>
                <div class="chart-container">
                    <canvas id="performanceChart"></canvas>
                </div>
            </section>

            <section>
                <h2>Test Results</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Test Name</th>
                            <th>Duration (ms)</th>
                            <th>Status</th>
                        </tr>
                    </thead>
                    <tbody>
"""

    for test in data['tests']:
        status_class = "status-pass" if test['passed'] else "status-fail"
        status_text = "✓ PASS" if test['passed'] else "✗ FAIL"

        html += f"""                        <tr>
                            <td>{test['name']}</td>
                            <td>{test['duration_ms']:.2f}</td>
                            <td class="{status_class}">{status_text}</td>
                        </tr>
"""

        if test['error']:
            html += f"""                        <tr>
                            <td colspan="3"><strong>Error:</strong> {test['error']}</td>
                        </tr>
"""

    html += f"""                    </tbody>
                </table>
            </section>

            <section>
                <h2>Methodology</h2>
                <div class="methodology">
                    <h3>Test Categories</h3>
                    <ul>
                        <li><strong>Quick Mode:</strong> 5-second smoke test with essential commands</li>
                        <li><strong>Full Mode:</strong> Comprehensive test suite covering shell features</li>
                        <li><strong>Compare Mode:</strong> Comparison benchmarks across shells (Rush, Bash, Zsh)</li>
                    </ul>

                    <h3>Metrics</h3>
                    <ul>
                        <li><strong>Duration:</strong> Time taken to execute the test in milliseconds</li>
                        <li><strong>Status:</strong> Pass/Fail status of the test</li>
                    </ul>

                    <h3>Notes</h3>
                    <ul>
                        <li>All timings are in milliseconds (ms)</li>
                        <li>Durations include lexing, parsing, and execution</li>
                        <li>Results may vary based on system load and resources</li>
                        <li>The "Quick Mode" provides rapid feedback for CI/CD pipelines</li>
                        <li>The "Full Mode" performs comprehensive testing for release validation</li>
                        <li>The "Compare Mode" benchmarks Rush against other shells</li>
                    </ul>
                </div>
            </section>
        </main>

        <footer>
            <p>Rush Benchmark Suite | Powered by criterion and custom benchmark tools</p>
        </footer>
    </div>

    <script>
        const ctx = document.getElementById('performanceChart').getContext('2d');
        const chart = new Chart(ctx, {{
            type: 'bar',
            data: {{
                labels: [{test_names_json}],
                datasets: [{{
                    label: 'Duration (ms)',
                    data: [{test_durations_json}],
                    backgroundColor: 'rgba(102, 126, 234, 0.8)',
                    borderColor: 'rgba(102, 126, 234, 1)',
                    borderWidth: 1,
                    borderRadius: 4
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                indexAxis: 'y',
                plugins: {{
                    legend: {{
                        display: true,
                        position: 'top'
                    }},
                    title: {{
                        display: true,
                        text: 'Test Performance'
                    }}
                }},
                scales: {{
                    x: {{
                        beginAtZero: true,
                        ticks: {{
                            callback: function(value) {{
                                return value.toFixed(2) + 'ms';
                            }}
                        }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"""

    with open(output_file, 'w') as f:
        f.write(html)

    print(f"✓ HTML report: {output_file}")

except Exception as e:
    print(f"Error generating HTML report: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_EOF

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}HTML report generated successfully${NC}"
    else
        echo -e "${YELLOW}Warning: Could not generate HTML report${NC}"
    fi
fi

# Save to historical directory
if [ "$SAVE_HISTORICAL" = true ]; then
    HISTORICAL_FILE="${HISTORICAL_DIR}/benchmark_${TIMESTAMP}.json"
    cp "$RESULTS_FILE" "$HISTORICAL_FILE"
    echo -e "${GREEN}Historical copy saved: $HISTORICAL_FILE${NC}"
fi

# Compare with previous run if specified
if [ -n "$COMPARE_FILE" ] && [ -f "$COMPARE_FILE" ]; then
    echo -e "${BLUE}Comparing with previous run...${NC}"

    python3 - <<'PYTHON_EOF' "$RESULTS_FILE" "$COMPARE_FILE"
import json
import sys

current_file = sys.argv[1]
previous_file = sys.argv[2]

try:
    with open(current_file, 'r') as f:
        current = json.load(f)

    with open(previous_file, 'r') as f:
        previous = json.load(f)

    print("\n=== Performance Comparison ===\n")
    print(f"{'Test':<30} {'Previous':<12} {'Current':<12} {'Change':<10}")
    print("-" * 64)

    regressions = []
    improvements = []

    for current_test in current['tests']:
        prev_test = next((t for t in previous['tests'] if t['name'] == current_test['name']), None)
        if prev_test:
            change_ms = current_test['duration_ms'] - prev_test['duration_ms']
            change_pct = (change_ms / prev_test['duration_ms'] * 100) if prev_test['duration_ms'] > 0 else 0

            change_str = f"{change_pct:+.1f}%"
            if change_pct > 10:
                change_str += " ⚠"
                regressions.append((current_test['name'], change_pct))
            elif change_pct < -10:
                change_str += " ✓"
                improvements.append((current_test['name'], change_pct))

            print(f"{current_test['name']:<30} {prev_test['duration_ms']:<12.2f} {current_test['duration_ms']:<12.2f} {change_str:<10}")

    if regressions:
        print("\n⚠ Performance Regressions:")
        for name, change in regressions:
            print(f"  - {name}: {change:+.1f}%")

    if improvements:
        print("\n✓ Performance Improvements:")
        for name, change in improvements:
            print(f"  - {name}: {change:+.1f}%")

    overall_change = ((current['total_duration_ms'] - previous['total_duration_ms']) / previous['total_duration_ms'] * 100)
    print(f"\nOverall change: {overall_change:+.1f}%")

except Exception as e:
    print(f"Error comparing results: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_EOF
fi

echo ""
echo -e "${GREEN}Report generation complete!${NC}"
echo ""
echo "Generated reports:"
if [ "$GENERATE_MARKDOWN" = true ]; then
    echo "  - Markdown: ${REPORT_DIR}/${RESULTS_FILENAME}.md"
fi
if [ "$GENERATE_HTML" = true ]; then
    echo "  - HTML: ${REPORT_DIR}/${RESULTS_FILENAME}.html"
fi
echo ""
echo "To share reports:"
echo "  - Copy HTML file to your blog or GitHub Pages"
echo "  - Include markdown file in GitHub repository"
echo "  - View HTML in web browser: open ${REPORT_DIR}/${RESULTS_FILENAME}.html"
