use cryptoscript::{parse, Executor};

fn main() {
    let instructions = parse(
        r#"
        push b"I am the walrus.";
        hash_sha256;
        push 0x475b03e74f7ee448273dbde5ab892746c7b23a2b4d050ccb7d9270b6fb152b72;
        check_equal;
        assert_true;
    "#,
    )
    .expect("failed to parse the input");
    Executor::default()
        .consume(instructions)
        .expect("error processing instructions");
}
