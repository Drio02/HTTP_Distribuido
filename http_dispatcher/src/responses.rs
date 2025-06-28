pub fn http_response_200(body : &str) -> String {
    let json = format!("{{\"status\":200,\"message\":\"{}\"}}", body);
    println!("Json que genera: {}", json);
    format!(
        "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

// Lògica de respuesta de errores
pub fn http_resonse_404(msg: &str) -> String {
    let json = format!("{{\"status\" : 404, \"error\" : \"{}\"}}", msg);
    format!(
        "HTTP/1.0 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
        json.len(),
        json
    )
}

pub fn http_resonse_400(msg: &str) -> String {
    let json = format!("{{\"status\" : 400, \"error\" : \"{}\"}}", msg);
    format!(
        "HTTP/1.0 400 Bad Request\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
        json.len(),
        json
    )
}

pub fn http_response_500_json(msg: &str) -> String {
    let json = format!("{{\"status\":500,\"message\":\"{}\"}}", msg);
    format!(
        "HTTP/1.0 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

