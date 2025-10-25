use crate::model::{Error, HttpMethod};

pub(crate) fn extract_http_details(request_line: &str) -> crate::model::Result<(HttpMethod, &str)> {
    // Check for an entirely empty string first
    if request_line.trim().is_empty() {
        return Err(Error::InvalidRequestLine);
    }

    let mut parts = request_line.split_whitespace();

    // The first part should be the HTTP method
    let method = parts.next().ok_or(Error::MissingHttpMethod)?;
    let method = method.parse::<HttpMethod>()?;

    // The second part should be the HTTP path
    let path = parts.next().ok_or(Error::MissingRequestPath)?;
    if !path.starts_with("/") {
        return Err(Error::MissingRequestPath);
    }

    Ok((method, path))
}

#[cfg(test)]
mod test {
    use super::*;

    // --- SUCCESS CASES ---

    #[test]
    fn test_success_get_root() {
        let request = "GET / HTTP/1.1\r\n";

        let result = extract_http_details(request);

        assert!(
            result.is_ok(),
            "Expected Ok result but got error: {:?}",
            result.err()
        );
        // Safely unwrap the result and compare the inner tuple
        let (method, path) = result.unwrap();
        assert_eq!(method, HttpMethod::Get);
        assert_eq!(path, "/");
    }

    #[test]
    fn test_success_post_with_query() {
        let request = "POST /api/users?action=create HTTP/1.1\r\n";

        let result = extract_http_details(request);

        match result {
            Ok((method, path)) => {
                assert_eq!(method, HttpMethod::Post);
                assert_eq!(path, "/api/users?action=create");
            }
            Err(e) => panic!("Expected Ok result but got error: {:?}", e),
        }
    }

    #[test]
    fn test_success_case_insensitivity() {
        let request = "dElEtE /item/99 HTTP/1.1\r\n";

        let result = extract_http_details(request);

        match result {
            Ok((method, path)) => {
                assert_eq!(method, HttpMethod::Delete);
                assert_eq!(path, "/item/99");
            }
            Err(e) => panic!("Expected Ok result but got error: {:?}", e),
        }
    }

    // --- FAILURE CASES ---

    #[test]
    fn test_error_empty_string() {
        let request = "";

        let result = extract_http_details(request);

        // Assert using pattern matching on the Err variant
        match result {
            Err(Error::InvalidRequestLine) => { /* Success */ }
            _ => panic!("Expected Error::InvalidRequestLine, got {:?}", result),
        }
    }

    #[test]
    fn test_error_missing_method() {
        // String that is too short, leading to UnrecognizedMethod when parsing "HTTP/1.1"
        let request = "  HTTP/1.1\r\n";

        let result = extract_http_details(request);

        match result {
            Err(Error::UnrecognizedHttpMethod) => { /* Success */ }
            _ => panic!("Expected Error::UnrecognizedHttpMethod, got {:?}", result),
        }
    }

    #[test]
    fn test_error_missing_path() {
        let request = "GET HTTP/1.1\r\n";

        let result = extract_http_details(request);

        match result {
            Err(Error::MissingRequestPath) => { /* Success */ }
            _ => panic!("Expected Error::MissingRequestPath, got {:?}", result),
        }
    }

    #[test]
    fn test_error_unrecognized_method() {
        let request = "CUSTOM /path HTTP/1.1\r\n";

        let result = extract_http_details(request);

        match result {
            Err(Error::UnrecognizedHttpMethod) => { /* Success */ }
            _ => panic!("Expected Error::UnrecognizedHttpMethod, got {:?}", result),
        }
    }
}
