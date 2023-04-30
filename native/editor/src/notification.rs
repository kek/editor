pub(crate) fn produce(typ: &str, msg: &str) {
    println!("{}", serde_json::json!({ "type": typ, "message": msg }));
}
