use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResultsFile {
    pub mode: String,
    pub tests: Vec<TestRecord>,
    pub total_duration_ms: f64,
    pub passed: usize,
    pub failed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRecord {
    pub name: String,
    pub duration_ms: f64,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRecord {
    pub command: String,
    pub aush_time: f64,
    pub bash_time: Option<f64>,
    pub zsh_time: Option<f64>,
    pub aush_vs_bash_ratio: Option<f64>,
    pub aush_vs_zsh_ratio: Option<f64>,
    pub min_time: f64,
    pub max_time: f64,
    pub std_dev: f64,
}

/// Generate markdown report from benchmark results
pub fn generate_markdown_report(results_file: &Path, output_file: &Path) -> Result<()> {
    let content = fs::read_to_string(results_file)?;
    let results: BenchmarkResultsFile = serde_json::from_str(&content)?;

    let report = build_markdown_report(&results);
    fs::write(output_file, report)?;

    Ok(())
}

/// Generate HTML report with embedded charts from benchmark results
pub fn generate_html_report(results_file: &Path, output_file: &Path) -> Result<()> {
    let content = fs::read_to_string(results_file)?;
    let results: BenchmarkResultsFile = serde_json::from_str(&content)?;

    let html = build_html_report(&results);
    fs::write(output_file, html)?;

    Ok(())
}

/// Compare current benchmark results with previous results
pub fn compare_results(current_file: &Path, previous_file: &Path) -> Result<ComparisonReport> {
    let current_content = fs::read_to_string(current_file)?;
    let current: BenchmarkResultsFile = serde_json::from_str(&current_content)?;

    let previous_content = fs::read_to_string(previous_file)?;
    let previous: BenchmarkResultsFile = serde_json::from_str(&previous_content)?;

    let mut regressions = Vec::new();
    let mut improvements = Vec::new();

    for current_test in &current.tests {
        if let Some(previous_test) = previous.tests.iter().find(|t| t.name == current_test.name) {
            let change_percent = ((current_test.duration_ms - previous_test.duration_ms)
                / previous_test.duration_ms)
                * 100.0;

            if change_percent > 10.0 {
                regressions.push(TestComparison {
                    name: current_test.name.clone(),
                    previous_duration_ms: previous_test.duration_ms,
                    current_duration_ms: current_test.duration_ms,
                    change_percent,
                });
            } else if change_percent < -10.0 {
                improvements.push(TestComparison {
                    name: current_test.name.clone(),
                    previous_duration_ms: previous_test.duration_ms,
                    current_duration_ms: current_test.duration_ms,
                    change_percent,
                });
            }
        }
    }

    Ok(ComparisonReport {
        regressions,
        improvements,
        total_change_percent: ((current.total_duration_ms - previous.total_duration_ms)
            / previous.total_duration_ms)
            * 100.0,
    })
}

#[derive(Debug, Clone)]
pub struct TestComparison {
    pub name: String,
    pub previous_duration_ms: f64,
    pub current_duration_ms: f64,
    pub change_percent: f64,
}

#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub regressions: Vec<TestComparison>,
    pub improvements: Vec<TestComparison>,
    pub total_change_percent: f64,
}

fn build_markdown_report(results: &BenchmarkResultsFile) -> String {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut report = format!(
        "# AUSH Benchmark Report\n\n\
         **Generated:** {}\n\n\
         ## Summary\n\n\
         - **Mode:** {}\n\
         - **Total Duration:** {:.2}ms\n\
         - **Tests Passed:** {}\n\
         - **Tests Failed:** {}\n\
         - **Total Tests:** {}\n\n",
        timestamp,
        results.mode,
        results.total_duration_ms,
        results.passed,
        results.failed,
        results.passed + results.failed
    );

    // Test results table
    report.push_str("## Test Results\n\n");
    report.push_str("| Test | Duration | Status |\n");
    report.push_str("|------|----------|--------|\n");

    for test in &results.tests {
        let status = if test.passed { "✓ PASS" } else { "✗ FAIL" };
        report.push_str(&format!(
            "| {} | {:.2}ms | {} |\n",
            test.name, test.duration_ms, status
        ));

        if let Some(error) = &test.error {
            report.push_str(&format!("| | **Error:** {} | |\n", error));
        }
    }

    // Methodology
    report.push_str("\n## Methodology\n\n");
    report.push_str("This benchmark report was generated using the AUSH benchmark runner.\n\n");
    report.push_str("### Test Categories\n\n");
    report.push_str("- **Quick Mode:** 5-second smoke test with essential commands\n");
    report.push_str("- **Full Mode:** Comprehensive test suite covering shell features\n");
    report
        .push_str("- **Compare Mode:** Comparison benchmarks across shells (AUSH, Bash, Zsh)\n\n");

    report.push_str("### Metrics\n\n");
    report.push_str("- **Duration:** Time taken to execute the test in milliseconds\n");
    report.push_str("- **Status:** Pass/Fail status of the test\n\n");

    // Notes
    report.push_str("## Notes\n\n");
    report.push_str("- All timings are in milliseconds (ms)\n");
    report.push_str("- Durations include lexing, parsing, and execution\n");
    report.push_str("- Results may vary based on system load and resources\n");

    report
}

fn build_html_report(results: &BenchmarkResultsFile) -> String {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Calculate some basic stats for the chart
    let avg_duration =
        results.tests.iter().map(|t| t.duration_ms).sum::<f64>() / results.tests.len() as f64;
    let max_duration = results
        .tests
        .iter()
        .map(|t| t.duration_ms)
        .fold(0.0, f64::max);

    // Build test data for Chart.js
    let mut test_names = Vec::new();
    let mut test_durations = Vec::new();
    for test in &results.tests {
        test_names.push(format!("\"{}\"", test.name));
        test_durations.push(test.duration_ms);
    }

    let test_names_json = test_names.join(",");
    let test_durations_json = test_durations
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AUSH Benchmark Report</title>
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
            <h1>AUSH Benchmark Report</h1>
            <div class="timestamp">Generated: {}</div>
        </header>

        <main>
            <section>
                <h2>Summary</h2>
                <div class="summary-grid">
                    <div class="summary-card">
                        <div class="label">Mode</div>
                        <div class="value">{}</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Total Duration</div>
                        <div class="value">{:.2}ms</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Tests Passed</div>
                        <div class="value">{}</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Tests Failed</div>
                        <div class="value">{}</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Avg Duration</div>
                        <div class="value">{:.2}ms</div>
                    </div>
                    <div class="summary-card">
                        <div class="label">Max Duration</div>
                        <div class="value">{:.2}ms</div>
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
"#,
        timestamp,
        results.mode,
        results.total_duration_ms,
        results.passed,
        results.failed,
        avg_duration,
        max_duration
    );

    // Add table rows for each test
    for test in &results.tests {
        let status_class = if test.passed {
            "status-pass"
        } else {
            "status-fail"
        };
        let status_text = if test.passed { "✓ PASS" } else { "✗ FAIL" };

        html.push_str(&format!(
            "                        <tr>\n                            <td>{}</td>\n                            <td>{:.2}</td>\n                            <td class=\"{}\">{}</td>\n                        </tr>\n",
            test.name, test.duration_ms, status_class, status_text
        ));

        if let Some(error) = &test.error {
            html.push_str(&format!(
                "                        <tr>\n                            <td colspan=\"3\"><strong>Error:</strong> {}</td>\n                        </tr>\n",
                error
            ));
        }
    }

    let html = html
        + &format!(
            r#"                    </tbody>
                </table>
            </section>

            <section>
                <h2>Methodology</h2>
                <div class="methodology">
                    <h3>Test Categories</h3>
                    <ul>
                        <li><strong>Quick Mode:</strong> 5-second smoke test with essential commands</li>
                        <li><strong>Full Mode:</strong> Comprehensive test suite covering shell features</li>
                        <li><strong>Compare Mode:</strong> Comparison benchmarks across shells (AUSH, Bash, Zsh)</li>
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
                        <li>The "Compare Mode" benchmarks AUSH against other shells</li>
                    </ul>
                </div>
            </section>
        </main>

        <footer>
            <p>AUSH Benchmark Suite | Powered by criterion and custom benchmark tools</p>
        </footer>
    </div>

    <script>
        const ctx = document.getElementById('performanceChart').getContext('2d');
        const chart = new Chart(ctx, {{
            type: 'bar',
            data: {{
                labels: [{}],
                datasets: [{{
                    label: 'Duration (ms)',
                    data: [{}],
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
"#,
            test_names_json, test_durations_json
        );

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_report_generation() {
        let results = BenchmarkResultsFile {
            mode: "quick".to_string(),
            tests: vec![
                TestRecord {
                    name: "test1".to_string(),
                    duration_ms: 1.5,
                    passed: true,
                    error: None,
                },
                TestRecord {
                    name: "test2".to_string(),
                    duration_ms: 2.3,
                    passed: true,
                    error: None,
                },
            ],
            total_duration_ms: 3.8,
            passed: 2,
            failed: 0,
            timestamp: None,
        };

        let report = build_markdown_report(&results);
        assert!(report.contains("AUSH Benchmark Report"));
        assert!(report.contains("quick"));
        assert!(report.contains("test1"));
        assert!(report.contains("test2"));
        assert!(report.contains("Methodology"));
    }

    #[test]
    fn test_html_report_generation() {
        let results = BenchmarkResultsFile {
            mode: "quick".to_string(),
            tests: vec![TestRecord {
                name: "test1".to_string(),
                duration_ms: 1.5,
                passed: true,
                error: None,
            }],
            total_duration_ms: 1.5,
            passed: 1,
            failed: 0,
            timestamp: None,
        };

        let html = build_html_report(&results);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("AUSH Benchmark Report"));
        assert!(html.contains("test1"));
        assert!(html.contains("Chart.js"));
    }
}
