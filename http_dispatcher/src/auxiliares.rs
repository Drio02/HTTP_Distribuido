use std::collections::HashMap;
// Funciones que necesita el dispatcher para funcionar
use std::sync::{Arc, Mutex, MutexGuard};
use std::net::{TcpStream};
use std::time::Duration;
use std::io::{Read, Write};
use std::env;

use futures::future::join_all;
use serde::Deserialize;

use crate::responses::{http_resonse_400, http_response_200, http_response_500_json};

//Estructura que define el estado de un Worker
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerStatus {
    Active,
    Inactive,
}

//Worker en el sistema
#[derive(Debug, Clone)]
pub struct Worker {
    pub id: String,
    pub address: String,
    pub status: WorkerStatus,
    pub task_completed: u64,
    pub tasks_failed: u64,
}

//Tiene todo el estado del dispatcher
#[derive(Debug)]
pub struct DispatcherState {
    pub workers: Vec<Worker>,
    pub next_worker_index: usize, //Index para estrategia de RR
}

#[derive(Deserialize)]
struct WorkerResponse {
    hits: u64,
}

pub fn initialize_workers() -> Vec<Worker> {
    println!("Buscando variable de entorno WORKER_ADDRESSE...");
    // Hay quie implementar toda la logica para leer el .env
    // La variable tendra las direcciones de los workers
    let addresses = env::var("WORKER_ADDRESSES").expect("Variable de entorno no esta definida");

    let workers: Vec<Worker> = addresses.split(',')
    .enumerate().map(|(i, address)| {
        let worker_id = format!("worker{}", i + 1);
        println!("[Init] Configurando worker: {} con direccion {}", worker_id, address);
        Worker{
            id: worker_id,
            address: address.trim().to_string(),
            status: WorkerStatus::Inactive,
            task_completed:0,
            tasks_failed:0
        }
    }).collect();

    if workers.is_empty(){
        panic!("La variable WORKER_ADDRESSES esta vacia. No se pueden configurar los workers.")
    }

    workers
}

pub async fn health_check(state_dispatcher: Arc<Mutex<DispatcherState>>) {
    let client = reqwest::Client::new();
    loop {
        //Bloquear el mutex, iterar sobre los workers y hacerles ping
        //Vamos actualizando el status segun la respuesta
        println!("(Healthcheck) Verificando estado de workers...");

        let workers_to_check = {
            let state = state_dispatcher.lock().unwrap();
            state.workers.iter().map(|w| (w.id.clone(), w.address.clone())).collect::<Vec<_>>()
        };

        for (id, address) in workers_to_check {
            let ping_url = format!("{}/ping", address);
            let mut new_status = WorkerStatus::Inactive; // Vamos a asumir que esta inactivo

            // Hacemos la peticion de ping con un timeout
            let request = client.get(&ping_url).timeout(Duration::from_secs(2));

            match request.send().await {
                Ok(response) if response.status().is_success() => {
                    println!("(Healthcheck) Worker {} en {} respondio correctamente.", id, address);
                    new_status = WorkerStatus::Active;
                }
                Ok(response) => { // Worker respondio, con codigo de error
                    println!("(Healthcheck) Worker {} en {} respondio con error: {}", id, address, response.status());
                }
                Err(e) => { //La peticion fallo (timeout, no se pudo conectar)
                    println!("(Healthcheck) Fallo al contactar worker {} en {}: {}", id, address, e);
                }
            }

            //Actualizamos el estado del worker
            {
                let mut state = state_dispatcher.lock().unwrap();
                if let Some(worker) = state.workers.iter_mut().find(|w| w.id == id) {
                    worker.status = new_status;
                }
            }
        }

        // Dormimos para la proxima verificacion
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}

//Funcio que va a parsear la linea de la peticion
fn parse_request_line(request: &str) -> (&str, &str) {
    if let Some(request_line) = request.lines().next() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return (parts[0], parts[1]);
        }
    }

    ("", "")
}

//Separa la ruta de los parametros
fn parse_query(path_query: &str) -> (String, HashMap<String, String>) {
    let mut parts = path_query.splitn(2, '?');
    let route = parts.next().unwrap_or("").to_string();
    let mut query_map = HashMap::new();

    if let Some(query) = parts.next() {
        for param in query.split('&') {
            let mut kv = param.splitn(2, '=');
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                query_map.insert(key.to_string(), value.to_string());
            }
        }
    }

    (route, query_map)
}

pub fn handle_cliente(mut stream: TcpStream, state_dispatcher: Arc<Mutex<DispatcherState>>) {
    let mut buffer = [0; 1024];
    if let Err(e) = stream.read(&mut buffer) {
        eprintln!("Error al leer: {}", e);
        return;
    }

    let request_str = String::from_utf8_lossy(&buffer[..]);

    let (_method, path_query) = parse_request_line(&request_str);

    let (path, params) = parse_query(path_query);

    let respose = match path.as_str() {
        "/workers" => handle_workers_status_request(state_dispatcher),
        "/montecarlo" => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let client = reqwest::Client::new();

            rt.block_on(handle_montecarlo_request(&params, &state_dispatcher, &client))
        }
        _ => handle_task_forwarding(path_query, state_dispatcher) //Cualquier otra ruta se considera para reenvio
    };

    if let Err(e) = stream.write_all(respose.as_bytes()) {
        eprintln!("Error al escribir respuesta: {}", e);
    }
    stream.flush().unwrap_or_default();
}

fn handle_workers_status_request(state_dispatcher: Arc<Mutex<DispatcherState>>) -> String {
    println!("Generando reporte de estado de workers ...");
    let state = state_dispatcher.lock().unwrap();

    let workers_json: Vec<String> = state.workers.iter().map(|w| {
        format!(
            "{{\"id\":\"{}\",\"address\":\"{}\",\"status\":\"{:?}\",\"tasks_completed\":{},\"tasks_failed\":{}}}",
            w.id, w.address, w.status, w.task_completed, w.tasks_failed
        )
    }).collect();

    let body = format!("[{}]", workers_json.join(","));
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

//Se encargar de elegier el siguiente worker ACTIVO con RR
pub fn select_next_worker(state: &mut MutexGuard<DispatcherState>) -> Option<usize>{
    println!("Next worker");
    let num_workers = state.workers.len();
    if num_workers == 0 {
        return None;
    }
    let mut index = state.next_worker_index;
    for i in 0..num_workers {
        if index ==  num_workers - 1 {
            state.next_worker_index = 0;
        } else {
            state.next_worker_index += 1;
        }
        println!("Worker que se va a evaluar: {}", index);
        println!("{}", state.next_worker_index);
        println!("Numero total de worker: {}", num_workers);
        println!("{:?}", state.workers[index].status);
        if state.workers[index].status == WorkerStatus::Active {
            println!("Entra para retornar el index");
            return Some(index);
        }
        index += 1;
    }
    None
}

pub fn handle_task_forwarding(path_and_query: &str, state_dispatcher: Arc<Mutex<DispatcherState>>) -> String{
    let client = reqwest::Client::new();

    
    let mut worker_info: Option<(String, String)> = Some(("".to_string(),"".to_string()));
    let max_retries = {state_dispatcher.lock().unwrap().workers.len()}; //Numero maximo de reintentos
    
    if max_retries == 0 {
        return "HTTP/1.1 503 Service Unavailable\r\n\r\nNo workers configured".to_string()
    }
    
    for attempt in 0..max_retries {

        worker_info = {
            let mut state = state_dispatcher.lock().unwrap();

            select_next_worker(&mut state).map(|index| {
                let worker = &state.workers[index];
                (worker.id.clone(), worker.address.clone())
            })
        };

        // Si el worker fue seleccionado procedemos
        if let Some((worker_id, worker_address)) =  worker_info{
            //Enviamos la tarea
            let target_url = format!("{}{}", worker_address, path_and_query);
            println!("Reenviado tarea '{}' al worker '{}' en '{}'", path_and_query, worker_id, target_url);
    
            // Reenviar la peticion y esperar respuesta
            // Usamos un runtime de Tokio
            let rt = tokio::runtime::Runtime::new().unwrap();
            let response_result = rt.block_on(client.get(&target_url).send());
    
            // Procesamos respuesta o el fallo
            match response_result {
                Ok(response) => {
                    let status = response.status();
                    let body = rt.block_on(response.text()).unwrap_or_else(|_| "".to_string());
        
                    //Incrementos el contador de tareas completadas para el worker
                    let mut state = state_dispatcher.lock().unwrap();
                    if let Some(worker) = state.workers.iter_mut().find(|w| w.id == worker_id) {
                        worker.task_completed += 1;
                    }
        
                    println!("Respuesta recibida del worker '{}'. Status: '{}'", worker_id, status);
                    //Falta la funcion que le da formato a la respuesta
                    return format_forwarded_response(status, &body)
                }
                Err(e) => {
                    eprintln!("Fallo al reenviar la tarea al worker '{}': '{}'", worker_id, e);
        
                    //Falla el worker, entonces lo marcamos como inactivo y registramos fallo
                    let mut state = state_dispatcher.lock().unwrap();
                    if let Some(worker) = state.workers.iter_mut().find(|w| w.id == worker_id) {
                        worker.status = WorkerStatus::Inactive;
                        worker.tasks_failed += 1;
                    }
    
                }
            }
    
        } else {
            //Si entra aqui es que el select_next_worker devolvio None
            //Significa que, no hay workers como tal o no hay activos
            println!("No se encontraron más workers activos. Abortando tarea.");
            return "HTTP/1.1 503 Service Unavailable\r\n\r\nNo active workers available".to_string();
        }
    }

        "HTTP/1.1 502 Bad Gateway\r\n\r\nCould not complete the task after all workers failed".to_string()
}

// Formata la respuesta recibida del worker
fn format_forwarded_response(status: reqwest::StatusCode, body:&str) -> String {
    format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        body.len(),
        body
    )
}   

//Funcion que maneja el calculo de pi
async fn handle_montecarlo_request(
    params: &HashMap<String, String>,
    state_dispatcher: &Arc<Mutex<DispatcherState>>,
    client: &reqwest::Client
) -> String {
    //Parseamos el request

    let total_points = match params.get("points").and_then(|s| s.parse::<u64>().ok()) {
        Some(p) if p > 0 => p,
        _ => return http_resonse_400("Parametro 'points' debe ser un numero entero positivo"),
    };

    //Obtenemos los workers activos
    let active_workers = {
        state_dispatcher.lock().unwrap().workers.iter()
        .filter(|w| w.status == WorkerStatus::Active)
        .map(|w| (w.id.clone(), w.address.clone()))
        .collect::<Vec<_>>()
    };

    if active_workers.is_empty() {
        return http_response_500_json("No hay workers disponibles");
    }

    //Dividmos el trabajo
    let points_per_worker = total_points / active_workers.len() as u64;
    let mut futures = vec![];

    println!("[Dispatcher] Dividiendo {} puntos entre {} workers ({} c/u)", total_points, active_workers.len(), points_per_worker);

    //Generamos las tareas para la peticion concurrente
    for (workerId, address) in active_workers {
        let url = format!("{}/internal/montecarlo?points={}", address, points_per_worker);
        let client_clone = client.clone();

        futures.push(tokio::spawn(async move {
            (workerId, client_clone.get(&url).send().await)
        }));
    }
        //Ejecutamos las peticiones en paralelo y esperamos los resultados
        let results = join_all(futures).await;
        let mut total_hits = 0;
        let mut succesful_workers = 0;

        for result in results {
            println!("Entra en la parte de resultados");

            if let Ok((workerId, Ok(response))) = result {
                if let Ok(worker_response) = response.json::<WorkerResponse>().await {
                    total_hits += worker_response.hits;
                    succesful_workers += 1;

                    let mut state = state_dispatcher.lock().unwrap();
                    if let Some(w) = state.workers.iter_mut().find(|w| w.id == workerId) {
                        w.task_completed += 1;
                    } 
                }
            }
        }
        if succesful_workers == 0 {
            return http_response_500_json("Ningun worker pudo completar la tarea de Montecarlo");
        }

        let pi_estimate = 4.0 * (total_hits as f64) / (points_per_worker * succesful_workers) as f64;

        let response_body = format!(
            "{{\"pi_estimate\":{}, \"total_points_simulated\":{}, \"total_hits\":{}}}",
            pi_estimate,
            points_per_worker * succesful_workers,
            total_hits
        );

        println!("Response body: {}", response_body);

        return http_response_200(&response_body);
}