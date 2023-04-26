#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let mut wrapped = String::with_capacity(data.len() + 2);
    wrapped.push('{');
    wrapped.push_str(data);
    wrapped.push('}');
    if let Ok(x) = serde_json::from_str::<serde_json::Value>(&wrapped) {
        let y = cuebasic::from_str(data).unwrap();
        assert_eq!(x, y)
    }
});
