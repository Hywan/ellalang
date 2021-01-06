use ella_value::Value;

pub fn print(args: &mut [Value]) -> Value {
    let arg = &args[0];
    print!("{}", arg);

    Value::Bool(true)
}

pub fn println(args: &mut [Value]) -> Value {
    let arg = &args[0];
    println!("{}", arg);

    Value::Bool(true)
}

pub fn assert(args: &mut [Value]) -> Value {
    let arg = &args[0];

    match arg {
        Value::Number(_) => {}
        Value::Bool(val) => assert!(*val),
        Value::Object(_) => {}
    }
    Value::Bool(true)
}

pub fn assert_eq(args: &mut [Value]) -> Value {
    let left = &args[0];
    let right = &args[1];

    assert_eq!(left, right);
    Value::Bool(true)
}

pub fn clock(_args: &mut [Value]) -> Value {
    let now = std::time::SystemTime::now();
    let since_the_epoch_ns = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    Value::Number(since_the_epoch_ns)
}
