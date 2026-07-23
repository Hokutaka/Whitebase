use whitebase_runner::{BackendRunStatus, Runner, RunnerConfig};

#[test]
fn measures_and_compares_standard_backends() {
    let runner = Runner::new();

    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    let config = RunnerConfig {
        warmup_iterations: 1,
        measured_iterations: 3,
        ..RunnerConfig::default()
    };

    let report = runner.run_add_f32(&lhs, &rhs, &config).unwrap();

    assert_eq!(report.input_length, 10);
    assert_eq!(report.results.len(), 6);

    for result in report.results {
        match result.status {
            BackendRunStatus::Completed { timing, comparison } => {
                println!(
                    "{}: mean {:.2} ns, match={}",
                    result.backend.display_name(),
                    timing.mean_nanoseconds,
                    comparison.matches_reference,
                );

                assert_eq!(timing.iterations, 3);
                assert!(comparison.matches_reference);
                assert_eq!(comparison.mismatch_count, 0);
            }

            BackendRunStatus::Unavailable => {
                println!("{}: unavailable", result.backend.display_name(),);
            }

            BackendRunStatus::Failed { error } => {
                panic!("{} failed: {error}", result.backend.display_name(),);
            }
        }
    }
}
