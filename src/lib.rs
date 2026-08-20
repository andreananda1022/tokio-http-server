#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String
}

#[derive(Debug)]
pub enum ParseError {
    InvalidRequest(String)
}

pub fn parse_request_line(line: &str) -> Result<Request, ParseError> {
    let trimmed_line = line.trim_end();
    let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(ParseError::InvalidRequest(format!("Invalid request line: {trimmed_line}")));
    }
    
    Ok(Request {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        version: parts[2].to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
}