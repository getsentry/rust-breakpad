use process_state::ProcessResult;

error_chain! {
    errors {
        /// An error raised when processing a dump by `ProcessState`.
        ProcessError(result: ProcessResult) {
            description("Process Error")
            display("Minidump Error: {}", &result)
        }

        /// An error raised by the `Resolver` during source line resolution.
        ResolverError(desc: String) {
            description("Resolver Error")
            display("Resolver Error: {}", &desc)
        }

        // An error raised by `convert_symbols` when generating Breakpad symbols.
        ConversionError(desc: String) {
            description("Conversion Error")
            display("Symbol Conversion Error: {}", &desc)
        }
    }
}
