use ella::interpret;

#[test]
#[should_panic]
fn smoke_assert() {
    interpret(
        r#"
        assert(false);"#,
    );
}

#[test]
#[should_panic]
fn smoke_assert_eq() {
    interpret(
        r#"
        assert_eq(1, 2);"#,
    );
}

#[test]
fn variables() {
    interpret(
        r#"
        let x = 1;
        assert_eq(x, 1);
        let y = x + 1;
        assert_eq(y, 2);
        assert_eq(y, x + 1);
        x = 10;
        assert_eq(x, 10);"#,
    );
}

#[test]
fn comments() {
    interpret(
        r#"
        let x = 1; // a comment
        assert_eq(x, 1);"#,
    );
}

#[test]
fn is_nan() {
    interpret(r#"assert(!is_nan(1));"#);
    interpret(r#"assert(is_nan(0/0));"#);
}

#[test]
fn parse_number() {
    interpret(r#"assert(!is_nan(parse_number("1")));"#);
    interpret(r#"assert(!is_nan(parse_number("1.2")));"#);
    interpret(r#"assert(is_nan(parse_number("not a number")));"#);
    interpret(r#"assert(is_nan(parse_number(1)));"#);
}

#[test]
fn op_assign() {
    interpret(
        r#"
        let x = 10;
        assert_eq(x, 10);

        assert_eq(x = 12, 12);// should be new value
        assert_eq(x += 10, 22);
        assert_eq(x -= 20, 2);
        assert_eq(x *= 2, 4);
        assert_eq(x /= 4, 1);"#,
    );
}

mod functions {
    use super::*;

    #[test]
    fn functions() {
        interpret(
            r#"
            fn foo() {
                return 1;
            }
            assert_eq(foo(), 1);"#,
        );
    }

    #[test]
    fn functions_with_params() {
        interpret(
            r#"
            fn double(x) {
                let result = x * 2;
                return result;
            }
            assert_eq(double(10), 20);
            assert_eq(double(-2), -4);"#,
        );
    }

    #[test]
    fn functions_implicit_return() {
        interpret(
            r#"
            fn foo() { }
            assert_eq(foo(), 0);"#,
        );
    }

    #[test]
    fn higher_order_function() {
        interpret(
            r#"
            fn twice(f, v) {
                return f(f(v));
            }
            fn double(x) {
                return x * 2;
            }
            
            assert_eq(twice(double, 10), 40);
            assert_eq(twice(double, -2), -8);"#,
        );
    }

    mod closures {
        use super::*;

        #[test]
        fn close_global_variable() {
            interpret(
                r#"
                fn assert_test() {
                    assert(true); // captures assert fn from global scope
                }
                assert_test();"#,
            );
        }

        #[test]
        fn basic_closures() {
            interpret(
                r#"
                fn closure() {
                    let x = 1;
                    fn inner() {
                        return x + 1;
                    }
                    return inner();
                }
                assert_eq(closure(), 2);"#,
            );
        }

        #[test]
        fn capture_with_block_stmt() {
            interpret(
                r#"
                {
                    assert(true);
                }"#,
            )
        }

        #[test]
        fn close_upvalues() {
            interpret(
                r#"
                fn createAdder(x) {
                    fn adder(y) {
                        return x + y;
                    }
                    return adder;
                }
                let addTwo = createAdder(2);
                assert_eq(addTwo(1), 3);"#,
            );
            interpret(
                r#"
                fn compose(f, g) {
                    fn inner(x) {
                        return f(g(x));
                    }
                    return inner;
                }
                fn addOne(x) { return x + 1; }
                fn addTwo(x) { return x + 2; }
                let addThree = compose(addOne, addTwo);
                assert_eq(addThree(2), 5);"#,
            );
        }

        #[test]
        fn capture_by_ref() {
            interpret(
                r#"
                let globalSet = 0;
                let globalGet = 0;

                fn main() {
                    let a = "initial";

                    fn set() { a = "updated"; }
                    fn get() { return a; }

                    globalSet = set;
                    globalGet = get;
                }

                main();
                assert_eq(globalGet(), "initial");
                globalSet();
                assert_eq(globalGet(), "updated");"#,
            );
        }

        #[test]
        fn nested_upvalues() {
            interpret(
                r#"
                fn outer() {
                    let x = "value";

                    fn middle() {
                        fn inner() {
                            return x;
                        }
                        return inner;
                    }
                    return middle;
                }

                let mid = outer();
                let in = mid();
                assert_eq(in(), "value");"#,
            );
        }
    }
}

mod control_flow {
    use super::*;

    #[test]
    fn if_and_else() {
        interpret(
            r#"
            let x = 0;

            let condition = true;
            if condition {
                x = 1;
            } else {
                x = 2;
            }
            assert_eq(x, 1);

            condition = false;
            if condition {
                x = 1;
            } else {
                x = 2;
            }
            assert_eq(x, 2);"#,
        );
    }

    #[test]
    fn fibonacci() {
        interpret(
            r#"
            fn fib(x) {
                if x <= 1 { return 1; }
                else { return fib(x - 1) + fib(x - 2); }
            }
            
            assert_eq(fib(20), 10946);"#,
        );
    }

    #[test]
    fn accumulate() {
        interpret(
            r#"
            let x = 0;
            let i = 0;
            while i < 20 {
                x = x + i;
                i = i + 1;
            }
            assert_eq(x, 190);"#,
        );
    }
}
