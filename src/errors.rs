use process_state::ProcessResult;

error_chain! {
    errors {
        /// An error raised during minidump processing.
        ProcessError(result: ProcessResult) {
            description("Process Error")
            display("Minidump Error: {}", &result)
        }

        /// An error raised during source line resolution.
        ResolverError(desc: String) {
            description("Resolver Error")
            display("Resolver Error: {}", &desc)
        }
    }
}
