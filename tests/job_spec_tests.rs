use aush::jobs::JobManager;

#[cfg(test)]
mod job_spec_tests {
    use super::*;

    #[test]
    fn test_parse_job_spec_by_number() {
        let manager = JobManager::new();
        let job_id = manager.add_job(12345, "sleep 100".to_string());

        let result = manager.parse_job_spec(&format!("%{}", job_id));
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, job_id);
        assert_eq!(job.pid, 12345);
    }

    #[test]
    fn test_parse_job_spec_plain_number() {
        let manager = JobManager::new();
        let job_id = manager.add_job(12345, "sleep 100".to_string());

        // Plain number without % should also work
        let result = manager.parse_job_spec(&job_id.to_string());
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, job_id);
    }

    #[test]
    fn test_parse_job_spec_current_job_percent_percent() {
        let manager = JobManager::new();
        manager.add_job(100, "first".to_string());
        let latest_id = manager.add_job(200, "latest".to_string());

        let result = manager.parse_job_spec("%%");
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, latest_id);
        assert_eq!(job.pid, 200);
    }

    #[test]
    fn test_parse_job_spec_current_job_percent_plus() {
        let manager = JobManager::new();
        manager.add_job(100, "first".to_string());
        let latest_id = manager.add_job(200, "latest".to_string());

        let result = manager.parse_job_spec("%+");
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, latest_id);
        assert_eq!(job.pid, 200);
    }

    #[test]
    fn test_parse_job_spec_current_job_bare_percent() {
        let manager = JobManager::new();
        manager.add_job(100, "first".to_string());
        let latest_id = manager.add_job(200, "latest".to_string());

        let result = manager.parse_job_spec("%");
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, latest_id);
    }

    #[test]
    fn test_parse_job_spec_previous_job() {
        let manager = JobManager::new();
        manager.add_job(100, "first".to_string());
        manager.add_job(200, "second".to_string());
        manager.add_job(300, "third".to_string());

        let result = manager.parse_job_spec("%-");
        assert!(result.is_ok());
        let job = result.unwrap();
        // Previous should be the second-to-last job (job 2)
        assert_eq!(job.pid, 200);
    }

    #[test]
    fn test_parse_job_spec_previous_job_only_one() {
        let manager = JobManager::new();
        manager.add_job(100, "only".to_string());

        let result = manager.parse_job_spec("%-");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No previous job"));
    }

    #[test]
    fn test_parse_job_spec_no_current_job() {
        let manager = JobManager::new();

        let result = manager.parse_job_spec("%%");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No current job"));
    }

    #[test]
    fn test_parse_job_spec_by_command_prefix() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());
        let grep_id = manager.add_job(200, "grep pattern file.txt".to_string());

        let result = manager.parse_job_spec("%grep");
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, grep_id);
        assert_eq!(job.pid, 200);
    }

    #[test]
    fn test_parse_job_spec_by_command_prefix_ambiguous() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());
        manager.add_job(200, "sleep 200".to_string());

        let result = manager.parse_job_spec("%sleep");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ambiguous job specification"));
    }

    #[test]
    fn test_parse_job_spec_by_command_prefix_no_match() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());

        let result = manager.parse_job_spec("%grep");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such job"));
    }

    #[test]
    fn test_parse_job_spec_containing_string() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());
        let grep_id = manager.add_job(200, "grep pattern file.txt".to_string());
        manager.add_job(300, "cat file.log".to_string());

        let result = manager.parse_job_spec("%?pattern");
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, grep_id);
        assert_eq!(job.pid, 200);
    }

    #[test]
    fn test_parse_job_spec_containing_string_multiple_words() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());
        let target_id = manager.add_job(200, "grep pattern file.txt".to_string());

        let result = manager.parse_job_spec("%?pattern file");
        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id, target_id);
    }

    #[test]
    fn test_parse_job_spec_containing_string_ambiguous() {
        let manager = JobManager::new();
        manager.add_job(100, "grep pattern file1.txt".to_string());
        manager.add_job(200, "grep pattern file2.txt".to_string());

        let result = manager.parse_job_spec("%?pattern");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ambiguous job specification"));
    }

    #[test]
    fn test_parse_job_spec_containing_string_no_match() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());

        let result = manager.parse_job_spec("%?pattern");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No job contains"));
    }

    #[test]
    fn test_parse_job_spec_containing_empty_string() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());

        let result = manager.parse_job_spec("%?");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid job specification"));
    }

    #[test]
    fn test_parse_job_spec_nonexistent_number() {
        let manager = JobManager::new();
        manager.add_job(100, "sleep 100".to_string());

        let result = manager.parse_job_spec("%999");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such job"));
    }

    #[test]
    fn test_parse_job_spec_invalid_format() {
        let manager = JobManager::new();

        let result = manager.parse_job_spec("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid job specification"));
    }

    #[test]
    fn test_parse_job_spec_distinguishes_prefix_vs_contains() {
        let manager = JobManager::new();
        let cat_id = manager.add_job(100, "cat file.txt".to_string());
        manager.add_job(200, "grep cat file.txt".to_string());

        // %cat should match job starting with "cat"
        let result = manager.parse_job_spec("%cat");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, cat_id);

        // %?cat should be ambiguous (both contain "cat")
        let result = manager.parse_job_spec("%?cat");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ambiguous"));
    }

    #[test]
    fn test_parse_job_spec_complex_scenario() {
        let manager = JobManager::new();
        manager.add_job(100, "vim document.txt".to_string());
        let sleep_id = manager.add_job(200, "sleep 300".to_string());
        let grep_id = manager.add_job(300, "grep -r pattern src/".to_string());

        // Test %n
        assert_eq!(manager.parse_job_spec("%1").unwrap().pid, 100);
        assert_eq!(manager.parse_job_spec("%2").unwrap().pid, 200);
        assert_eq!(manager.parse_job_spec("%3").unwrap().pid, 300);

        // Test %+ and %%
        assert_eq!(manager.parse_job_spec("%+").unwrap().id, grep_id);
        assert_eq!(manager.parse_job_spec("%%").unwrap().id, grep_id);

        // Test %-
        assert_eq!(manager.parse_job_spec("%-").unwrap().id, sleep_id);

        // Test %string (prefix)
        assert_eq!(manager.parse_job_spec("%vim").unwrap().pid, 100);
        assert_eq!(manager.parse_job_spec("%sleep").unwrap().pid, 200);
        assert_eq!(manager.parse_job_spec("%grep").unwrap().pid, 300);

        // Test %?string (contains)
        assert_eq!(manager.parse_job_spec("%?document").unwrap().pid, 100);
        assert_eq!(manager.parse_job_spec("%?300").unwrap().pid, 200);
        assert_eq!(manager.parse_job_spec("%?-r").unwrap().pid, 300);
        assert_eq!(manager.parse_job_spec("%?pattern").unwrap().pid, 300);
    }
}
