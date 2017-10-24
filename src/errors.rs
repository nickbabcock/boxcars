error_chain! {
    errors {
        Parsing(err: String) {
            description("An error occurred while parsing")
            display("Parsing error: {}", err)
        }
    }
}
