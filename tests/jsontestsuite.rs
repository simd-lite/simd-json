use std::fs;
use std::path::PathBuf;

fn json_test_files(group: &str) -> Vec<PathBuf> {
    let directory = PathBuf::from("data/json_test_suite").join(group);
    let mut files = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn parsing() {
    let mut failures = Vec::new();

    for path in json_test_files("test_parsing") {
        let name = path.file_name().unwrap().to_string_lossy();
        let expected = match name.as_bytes()[0] {
            b'y' => Some(true),
            b'n' => Some(false),
            b'i' => None,
            prefix => panic!("unknown JSONTestSuite prefix {prefix} in {name}"),
        };

        let mut input = fs::read(&path).unwrap();
        let accepted = simd_json::to_borrowed_value(&mut input).is_ok();
        if expected.is_some_and(|expected| accepted != expected) {
            failures.push(path.display().to_string());
        }
    }

    assert!(
        failures.is_empty(),
        "unexpected parsing results: {failures:?}"
    );
}

#[test]
fn transform_inputs_do_not_panic() {
    for path in json_test_files("test_transform") {
        let mut input = fs::read(path).unwrap();
        // JSONTestSuite has no expected result for transform inputs.
        let _ = simd_json::to_borrowed_value(&mut input);
    }
}
