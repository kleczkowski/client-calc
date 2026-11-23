use client_calc::generate_report;

#[test]
fn generates_expected_report_for_all_transaction_kinds() {
    let input = include_bytes!("fixtures/all_transactions.csv");
    let expected = include_str!("fixtures/all_transactions_report.csv");
    let mut output = Vec::new();

    generate_report(input.as_slice(), &mut output).unwrap();

    let actual = std::str::from_utf8(&output).unwrap();
    let mut actual_lines = actual.lines();
    let mut expected_lines = expected.lines();
    assert_eq!(actual_lines.next(), expected_lines.next());

    let mut actual_rows: Vec<_> = actual_lines.collect();
    actual_rows.sort_unstable();
    assert_eq!(actual_rows, expected_lines.collect::<Vec<_>>());
}

#[test]
fn rejects_deposits_and_withdrawals_without_amounts() {
    for input in [
        include_str!("fixtures/deposit_missing_amount.csv"),
        include_str!("fixtures/withdrawal_missing_amount.csv"),
    ] {
        let mut output = Vec::new();

        let error = generate_report(input.as_bytes(), &mut output).unwrap_err();

        assert!(error.to_string().contains("missing amount"), "{error}");
        assert!(output.is_empty());
    }
}

#[test]
fn rejects_negative_deposits_without_writing_a_report() {
    let input = include_bytes!("fixtures/deposit_negative_amount.csv");
    let mut output = Vec::new();

    let error = generate_report(input.as_slice(), &mut output).unwrap_err();

    assert!(error.to_string().contains("negative amount"), "{error}");
    assert!(output.is_empty());
}

#[test]
fn late_csv_error_prevents_report_output() {
    let input = include_bytes!("fixtures/withdrawal_extra_field.csv");
    let mut output = Vec::new();

    let error = generate_report(input.as_slice(), &mut output).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("too many fields in transaction row 2")
    );
    assert!(output.is_empty());
}
