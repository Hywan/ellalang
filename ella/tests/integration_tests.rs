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
