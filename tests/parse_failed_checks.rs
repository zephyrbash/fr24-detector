use fr24_detector::parse_failed_checks;

#[test]
fn detects_failed_entries_from_status_output() {
    let output = "FR24 Feeder/Decoder Process: running.\nFR24 Stats Timestamp: .\nFR24 Link: unknown ... failed!\nReceiver: down ... failed!";

    let failed_checks = parse_failed_checks(output);

    assert_eq!(failed_checks, vec!["FR24 Link", "Receiver"]);
}

#[test]
fn returns_no_failed_entries_when_none_are_present() {
    let output = "FR24 Feeder/Decoder Process: running.\nFR24 Stats Timestamp: .";

    let failed_checks = parse_failed_checks(output);

    assert!(failed_checks.is_empty());
}
