error_chain! {
    errors {
        MinidumpError(desc: String) {
            description("Minidump Error")
            display("Minidump Error: {}", &desc)
        }

        ResolverError(desc: String) {
            description("Resolver Error")
            display("Resolver Error: {}", &desc)
        }
    }
}
