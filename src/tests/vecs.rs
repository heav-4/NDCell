use super::assert_output;

#[test]
fn test_vector_types() {
    assert_output(
        Err(unimplemented!()),
        "
        @transition {
            set x = [1, 2, 3]
            set y = [4, 5, 6]
            set x = y
        }",
    );
    assert_output(
        Err(unimplemented!()),
        "
        @transition {
            set x = [1, 2, 3]
            set y = [1]
            set x = y
        }",
    );
    assert_output(
        Err(unimplemented!()),
        "
        @transition {
            set x = [1, 2, 3]
            set x = [1, 2, 3, 4]
        }",
    );
}

#[test]
fn test_vector_access() {
    assert_output(
        Ok(9),
        "
        @transition {
            set v = [1, 10, 100]
            become #(v.y - v.x)
        }",
    );
    assert_output(
        Ok(0),
        "
        @transition {
            set v = [1, 10, 100]
            become #(v.w)
        }",
    );
    assert_output(
        Ok(0),
        "
        @transition {
            set v = [1, 10, 100]
            become #(v.w)
        }",
    );
}

#[test]
fn test_vector_ops() {
    // Test addition, product, and sum.
    assert_output(
        Ok(7778),
        "
        @transition {
            set v = [1, 10, 100]
            v += [2, 2, 2]
            // v = [2, 20, 200]
            // v.product = 2 * 20 * 200 = 8000
            // v.sum = 2 + 20 + 200 = 222
            // v.product - v.sum = 8000 - 222 = 7778
            become #(v.product - v.sum)
        }",
    );
    // Test ops on vectors of different lengths.
    assert_output(
        Ok(100),
        "
        @transition {
            set v = [1, 10, 100] - [1, 10]
            become #(v.sum)
        }",
    );
    // Test that multiplication and bitwise AND between vectors of different
    // lengths shrink vectors instead of extending them.
    assert_output(
        Ok(122),
        "
        @transition {
            set a = [1, 10, 100] & [3, 3] // [1, 2]
            set b = [1, -10, 100] * [3, 4] // [3, -40]
            // a.product = 1 * 2 = 2
            // b.product = 3 * -40 = -120

            // If the vectors were extended, then the product would be zero.
            become #(a.product + -b.product)
        }",
    );
}

#[test]
fn test_vector_constructor() {
    assert_output(
        Ok(16),
        "
        @transition {
            become #(vec4(2).product)
        }",
    );
    assert_output(
        Ok(2),
        "
        @transition {
            become #(vec2().len - vec2().sum)
        }",
    );
}
