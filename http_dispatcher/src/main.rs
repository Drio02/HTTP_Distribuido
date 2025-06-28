use std::sync::{Arc, Mutex};
use std::net::{TcpListener};
use std::thread;

use tokio::runtime::Runtime;

use crate::auxiliares::{handle_cliente, health_check, initialize_workers, DispatcherState};

mod auxiliares;
mod responses;

fn main() {
    println!("Iniciado Dispatcher...");

    //Creamos un runtime de Tokio para ejecutar las tareas asincronicos
    let rt = Runtime::new().unwrap();

    // Incializa el estados de los workers
    let workers = initialize_workers();

    let initial_state = DispatcherState {
        workers,
        next_worker_index: 0,
    };

    //Inicializamos el estado del dispatcher
    let dispatcher_state = Arc::new(Mutex::new(initial_state));

    //Iniciamos el hilo en segundo plano para el healthcheck
    let healthcheck_state = dispatcher_state.clone();
    rt.spawn(async move {
        health_check(healthcheck_state).await;
    });
    println!("Hilo de healthcheck iniciado.");

    //Abrimos el TCP para escuchar las solicituides de los clientes
    let listener = TcpListener::bind("0.0.0.0:8080").expect("No se pudo iniciar el servidor en el puerto 8080");
    println!("Dispatcher escuchando en http://0.0.0.0:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = dispatcher_state.clone();
                
                thread::spawn(move || {
                    handle_cliente(stream, state_clone);
                });
            }
            Err(e) => {
                eprintln!("Error al aceptar conexion: {}", e);
            }
        }
    }
}