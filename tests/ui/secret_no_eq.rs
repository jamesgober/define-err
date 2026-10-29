use define_err::Secret;

fn main() {
    let a = Secret::new("s3cr3t");
    let b = Secret::new("guess");
    let _same = a == b;
}
