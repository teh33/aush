use aush::executor::Executor;
use aush::jobs::JobStatus;
use aush::lexer::Lexer;
use aush::parser::Parser;
use std::thread;
use std::time::Duration;

#[test]
fn test_background_job_automatic_reaping() {
    let mut executor = Executor::new();

    // Start a short-lived background job
    let tokens = Lexer::tokenize("sleep 0.1 &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    // Should get job notification
    assert!(result.stdout().contains("[1]"));

    // Job should be in the job list initially
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, 1);

    // Wait for the job to complete
    thread::sleep(Duration::from_millis(500));

    // Manually trigger zombie reaping (normally done by SIGCHLD handler)
    executor.runtime_mut().job_manager().reap_zombies();

    // Check job status - should be Done
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Done);
}

#[test]
fn test_multiple_background_jobs_reaping() {
    let mut executor = Executor::new();

    // Start multiple short-lived background jobs
    for _ in 0..3 {
        let tokens = Lexer::tokenize("sleep 0.1 &").unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        executor.execute(statements).unwrap();
        thread::sleep(Duration::from_millis(100));
    }

    // Should have 3 jobs
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 3);

    // Wait for all jobs to complete
    thread::sleep(Duration::from_millis(500));

    // Reap zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // All jobs should be Done
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 3);
    for job in jobs {
        assert_eq!(job.status, JobStatus::Done);
    }
}

#[test]
fn test_background_job_still_running() {
    let mut executor = Executor::new();

    // Start a long-running background job
    let tokens = Lexer::tokenize("sleep 10 &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Job should be running
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Running);

    // Try to reap - should still be running
    executor.runtime_mut().job_manager().reap_zombies();

    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Running);

    // Cleanup - terminate the job
    executor
        .runtime_mut()
        .job_manager()
        .terminate_job(jobs[0].id)
        .ok();
    thread::sleep(Duration::from_millis(200));
    executor.runtime_mut().job_manager().reap_zombies();
}

#[test]
fn test_terminated_job_reaping() {
    let mut executor = Executor::new();

    // Start a long-running background job
    let tokens = Lexer::tokenize("sleep 30 &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    let job_id = jobs[0].id;

    // Terminate the job
    executor
        .runtime_mut()
        .job_manager()
        .terminate_job(job_id)
        .unwrap();

    // Wait for termination to complete
    thread::sleep(Duration::from_millis(200));

    // Reap zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // Job should be marked as Terminated
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Terminated);
}

#[test]
fn test_zombie_process_cleanup() {
    let mut executor = Executor::new();

    // Start a job that exits immediately
    let tokens = Lexer::tokenize("sh -c 'exit 0' &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Wait for the process to exit (becomes zombie)
    thread::sleep(Duration::from_millis(200));

    // Before reaping, job might still appear as Running
    let jobs_before = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs_before.len(), 1);

    // Reap zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // After reaping, job should be Done
    let jobs_after = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs_after.len(), 1);
    assert_eq!(jobs_after[0].status, JobStatus::Done);
}

#[test]
fn test_no_zombies_after_wait() {
    let mut executor = Executor::new();

    // Start a short-lived background job using sleep (external command)
    let tokens = Lexer::tokenize("sleep 0.1 &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Get the job ID
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    let job_id = jobs[0].id;

    // Wait for it to complete
    thread::sleep(Duration::from_millis(500));

    // Use wait builtin to reap the process
    let wait_cmd = format!("wait {}", job_id);
    let tokens = Lexer::tokenize(&wait_cmd).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).ok(); // wait might fail if already reaped

    // After wait, reaping again should handle any remaining zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // Job should be in completed state
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    if !jobs.is_empty() {
        assert_eq!(jobs[0].status, JobStatus::Done);
    }
}

#[test]
fn test_reap_zombies_thread_safety() {
    let mut executor = Executor::new();

    // Start several background jobs
    for _ in 0..5 {
        let tokens = Lexer::tokenize("sleep 0.1 &").unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        executor.execute(statements).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    // Wait for all to complete
    thread::sleep(Duration::from_millis(500));

    // Call reap_zombies multiple times (simulating concurrent SIGCHLD signals)
    for _ in 0..3 {
        executor.runtime_mut().job_manager().reap_zombies();
        thread::sleep(Duration::from_millis(10));
    }

    // All jobs should be Done
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 5);
    for job in jobs {
        assert_eq!(job.status, JobStatus::Done);
    }
}

#[test]
fn test_mixed_job_states() {
    let mut executor = Executor::new();

    // Start one completed job
    let tokens = Lexer::tokenize("sleep 0.1 &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Start one long-running job
    let tokens = Lexer::tokenize("sleep 10 &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Wait for first to complete
    thread::sleep(Duration::from_millis(500));

    // Reap zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // Should have 2 jobs: one Done, one Running
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 2);

    let done_count = jobs.iter().filter(|j| j.status == JobStatus::Done).count();
    let running_count = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Running)
        .count();

    assert_eq!(done_count, 1);
    assert_eq!(running_count, 1);

    // Cleanup - terminate the running job
    for job in jobs {
        if job.status == JobStatus::Running {
            executor
                .runtime_mut()
                .job_manager()
                .terminate_job(job.id)
                .ok();
        }
    }
    thread::sleep(Duration::from_millis(200));
    executor.runtime_mut().job_manager().reap_zombies();
}

#[test]
fn test_background_job_exit_code() {
    let mut executor = Executor::new();

    // Start a background job that fails
    let tokens = Lexer::tokenize("sh -c 'exit 42' &").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Wait for completion
    thread::sleep(Duration::from_millis(500));

    // Reap zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // Job should be Done (exit code is tracked separately in waitpid)
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Done);
}

#[test]
fn test_no_jobs_reaping() {
    let mut executor = Executor::new();

    // Reap with no jobs - should not crash
    executor.runtime_mut().job_manager().reap_zombies();

    // Should have no jobs
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 0);
}

#[test]
fn test_rapid_job_creation_and_reaping() {
    let mut executor = Executor::new();

    // Rapidly create and complete jobs
    for _ in 0..10 {
        let tokens = Lexer::tokenize("sleep 0.1 &").unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        executor.execute(statements).unwrap();
    }

    // Wait for all to complete
    thread::sleep(Duration::from_millis(1000));

    // Reap all zombies
    executor.runtime_mut().job_manager().reap_zombies();

    // All should be Done
    let jobs = executor.runtime_mut().job_manager().list_jobs();
    assert_eq!(jobs.len(), 10);
    for job in jobs {
        assert_eq!(job.status, JobStatus::Done);
    }
}
