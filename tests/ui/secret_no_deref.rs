use define_err::Secret;

fn main() {
    let token = Secret::new(String::from("s3cr3t"));
    let _leak: &str = &*token;
}
