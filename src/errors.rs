use processor::ProcessResult;

error_chain! {
    foreign_links {
        IoError(::std::io::Error);
        UuidParseError(::uuid::ParseError);
        ParseIntError(::std::num::ParseIntError);
    }

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

        // An error raised when parsing `CodeModuleId`.
        ParseIdError(desc: String) {
            description("CodeModule ID Parse Error")
            display("CodeModule ID Parse Error: {}", &desc)
        }
    }
}
