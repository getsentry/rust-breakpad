error_chain! {
    errors {
        MinidumpError(desc: String) {
            description("Minidump Error")
            display("Minidump Error: {}", &desc)
        }
    }
}
