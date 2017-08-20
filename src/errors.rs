error_chain! {
    errors {
        /// An error raised during minidump processing.
        MinidumpError(desc: String) {
            description("Minidump Error")
            display("Minidump Error: {}", &desc)
        }

        /// An error raised during source line resolution.
        ResolverError(desc: String) {
            description("Resolver Error")
            display("Resolver Error: {}", &desc)
        }
    }
}
