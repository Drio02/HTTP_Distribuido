use std::net::TcpListener;

mod handle_connection;
mod endpoints;
mod responses;

use crate::handle_connection::handle_connection;
fn main() {
    let listener = match TcpListener::bind("0.0.0.0:7878") {
        Ok(listener) => {
            println!("Servidor simple iniciado y escuchando en 0.0.0.0:7878");
            listener
        },
        Err(e) => {
            eprintln!("ERROR CRÍTICO: No se pudo enlazar al puerto 7878. Error: {}", e);
            std::process::exit(1);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("[Worker] Conexión entrante aceptada.");
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexion.");
            }
        }
    }
}