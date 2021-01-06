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
