#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    if data.contains("\\") {
        // for now...
        return;
    }
    if let Ok(x) = serde_json::from_str::<serde_json::Value>(data) {
        let y = cuebasic::from_str(&x.to_string()).unwrap();
        assert_eq!(y, x)
    }
});
