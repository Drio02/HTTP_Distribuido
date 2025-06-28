use std::{collections::HashMap, io::{Read, Write}, net::TcpStream, thread::sleep, time::{Duration, Instant}};

use crate::{endpoints::{calculate_monte_carlo, create_file, delete_file, fibonacci, generate_random_numbers, rerverse_text, sha256_hash, timestamp_iso}, responses::{http_resonse_400, http_resonse_404, http_response_200, http_response_500}};

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

    let response = route_request(&path);

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Fallo al escribir la respuesta en el stream: {}", e);
        return;
    }
    if let Err(e) = stream.flush() {
        eprintln!("Fallo al hacer flush en el stream: {}", e);
    }
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
pub fn route_request(path: &str) -> String {
    let (route, params) = parse_query(path);

    match route.as_str() {
        "ping" => {
            return http_response_200("{\"status\":\"ok\"}");
        }
        
        "/internal/montecarlo" => {
            if let Some(p_str) = params.get("points") {
                if let Ok(p) = p_str.parse::<u64>() {
                    let hits = calculate_monte_carlo(p);
                    let body = format!("{{\"hits\":{}}}", hits);
                    return format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                }
            }
            return http_resonse_400("Parametro 'points' invalido o faltante");
        }

        "/fibonacci" => {
            if let Some(n_str) = params.get("num") {
                if let Ok(n) = n_str.parse::<u64>() {
                    let result = fibonacci(n);
                    return http_response_200(&result.to_string());
                }
            }
            return http_resonse_400("Parametro 'num' invalido");
        }

        "/reverse" => {
            if let Some(text) = params.get("text") {
                let result = rerverse_text(text);
                return http_response_200(&result);
            }
            return http_resonse_400("Falta el parametro 'text'");
        }

        "/hash" => {
            if let Some(text) = params.get("text") {
                let result = sha256_hash(text);
                return http_response_200(&result);
            }
            return http_resonse_400("Falta el parametro 'text'");
        }

        "/timestamp" => {
            let result = timestamp_iso();
            return http_response_200(&result);
        }

        "/sleep" => {
            if let Some(n_str) = params.get("seconds") {
                if let Ok(n) = n_str.parse::<u64>() {
                    sleep(Duration::from_secs(n));
                    return http_response_200(&format!("Simulado retraso de {} segundos", n));
                }
            }
            return http_resonse_400("Parámetro 'seconds' inválido o faltante");
        }

        "/random" => {
            let count = params.get("count").and_then(|s| s.parse::<usize>().ok());
            let min = params.get("min").and_then(|s| s.parse::<i32>().ok());
            let max = params.get("max").and_then(|s| s.parse::<i32>().ok());
        
            if let (Some(c), Some(mi), Some(ma)) = (count, min, max) {
                if mi >= ma {
                    return http_resonse_400("El parametro 'min' debe ser menor que 'max'");
                }
                let numbers = generate_random_numbers(c, mi, ma);
                return http_response_200(&format!("{:?}", numbers));
            }
            return http_resonse_400("Faltan parametros (count, min, max) o son invalidos");
        }

        "/createfile" => {
            if let (Some(name), Some(content)) = (params.get("name"), params.get("content")) {
                match create_file(name, content) {
                    Ok(msg) => http_response_200(&format!("{}", msg)),
                    Err(e) => http_response_500(&e)
                }
            } else {
                return http_resonse_400("Faltan parametros 'name' o 'content'");
            }
        }

        "/deletefile" => {
            if let Some(name) = params.get("name") {
                match delete_file(name) {
                    Ok(msg) => http_response_200(&format!("{}", msg)),
                    Err(e) => http_response_500(&e),
                }
            } else {
                return http_resonse_400("Falta el parametro 'name'");
            }
        }

        "/help" => {
            let help_text = format!("\"endpoints\" : [
                {{\"path\" : \"reverse\", 
                \"description\" : \"Invierte el texto recibido\", 
                \"params\" : [\"text: texto que se desea invertir\"], 
                \"example\" : \"/reverse?text=abc\"}},
                {{\"path\" : \"toupper\", \"description\" : \"Convierte el texto a mayúsculas\", \"params\" : [\"text: texto a convertir\"], \"example\" : \"/toupper?text=hola\"}},
                {{\"path\" : \"sha256\", \"description\" : \"Devuelve el hash SHA-256 del texto\", \"params\" : [\"text: texto a hashear\"], \"example\" : \"/sha256?text=hola\"}},
                {{\"path\" : \"fibonacci\", \"description\" : \"Calcula el n-ésimo número de Fibonacci (recursivo)\", \"params\" : [\"num: número a calcular\"], \"example\" : \"/fibonacci?num=10\"}},
                {{\"path\" : \"random\", \"description\" : \"Genera una lista de números aleatorios\", \"params\" : [\"count: cantidad\", \"min: mínimo\", \"max: máximo\"], \"example\" : \"/random?count=5&min=10&max=100\"}},
                {{\"path\" : \"timestamp\", \"description\" : \"Devuelve la hora actual en formato ISO\", \"params\" : [], \"example\" : \"/timestamp\"}},
                {{\"path\" : \"sleep\", \"description\" : \"Simula una espera bloqueante de N segundos\", \"params\" : [\"seconds: segundos a esperar\"], \"example\" : \"/sleep?seconds=3\"}},
                {{\"path\" : \"createfile\", \"description\" : \"Crea un archivo con el contenido indicado\", \"params\" : [\"name: nombre del archivo\", \"content: contenido\"], \"example\" : \"/createfile?name=miarchivo&content=hola\"}},
                {{\"path\" : \"deletefile\", \"description\" : \"Elimina un archivo existente\", \"params\" : [\"name: nombre del archivo\"], \"example\" : \"/deletefile?name=miarchivo\"}},
                {{\"path\" : \"simulate\", \"description\" : \"Simula un endpoint como reverse, toupper, etc., con retardo\", \"params\" : [\"seconds: retardo\", \"task: nombre del endpoint interno\", \"otros: según la tarea\"], \"example\" : \"/simulate?seconds=2&task=reverse&text=hola\"}},
                {{\"path\" : \"loadtest\", \"description\" : \"Encola múltiples tareas para medir carga del sistema\", \"params\" : [\"task: tipo de tarea\", \"count: cuántas tareas\", \"text: valor base si aplica\"], \"example\" : \"/loadtest?task=reverse&count=5&text=hola\"}},
                {{\"path\" : \"help\", \"description\" : \"Devuelve este manual de uso de endpoints\", \"params\" : [], \"example\" : \"/help\"}},
                ]");
                return http_response_200(&help_text);
        }

        _ => http_resonse_404("Ruta no encontrada")
    }
}

/*
Separa la ruta principal de los parametros
*/
pub fn parse_query (path: &str) -> (String, HashMap<String, String>) {
    let mut parts = path.split('?');
    let route = parts.next().unwrap_or("").to_string();
    let mut query_map = HashMap::new();

    if let Some(query) = parts.next() {
        for param in query.split('&') {
            let mut kv = param.split('=');
            let key = kv.next().unwrap_or("").to_string();
            let value = kv.next().unwrap_or("").to_string();
            query_map.insert(key, value);
        }
    }

    (route, query_map)
}