use std::{io::Read, net::TcpStream, time::Instant};

/*
    Funcion encargada de gestionar la conexion
*/
pub fn handle_connection(mut stream: TcpStream, start_time: Instant) {
    let mut buffer = [0u8; 1024];
    let read_bytes = stream.read(&mut buffer).unwrap_or(0);

    if read_bytes == 0 {
        return ;
    }

    let request = String::from_utf8_lossy(&buffer[..]);
    let (method, path) = parse_request(&request);

    
}

/*
Extrae mètodo HTTP de la solicitud y la ruta.
*/
pub fn parse_request(request: &str) -> (&str, String) {
    let mut lines = request.lines();
    if let Some(request_line) = lines.next() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return (parts[0], parts[1].to_string());
        }
    }
    ("", "".to_string())
}

/*
Router principal
Determina el tipo de tarea a realizar
*/