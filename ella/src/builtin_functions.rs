use ella_value::Value;

pub fn assert_eq(args: &mut [Value]) -> Value {
    let left = &args[0];
    let right = &args[1];

    assert_eq!(left, right);
    Value::Bool(true)
}
