use xml::reader;

error_chain! {
    foreign_links {
        Xml(reader::Error);
    }

    errors {
        EmptyTokenError {
            description("empty token")
            display("empty token")
        }
    }
}
