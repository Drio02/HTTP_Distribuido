use rand::Rng;
use sha2::{Sha256, Digest};
use chrono::{self, DateTime, Utc};
use std::fs::{File, create_dir_all, remove_file};
use std::io::Write;
use std::path::Path;


// Este archivo va a ser un mòdulo que va a contener la lògica de todos los endpoints

// /fibonacci
pub fn fibonacci(n: u64) -> u64{
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
// / createfile?name=filename&content=text&repeat=X
pub fn create_file (name : &str, content: &str) -> Result<String, String> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Nombre del archivo invàlido (Solo se permiten alfanùmericos)".to_string());
    }

    let folder = "archivos";
    if let Err(_) = create_dir_all(folder) {
        return Err("No se pudo crear el directorio".to_string());
    }

    let path = format!("{}/{}.txt", folder, name);
    let path_original = Path::new(&path);

    if path_original.exists() {
        return Err(format!("El archivo '{}' ya existe", path));
    }

    match File::create(&path_original) {
        Ok(mut file) => {
            if let Err(_) = file.write_all(content.as_bytes()) {
                return Err("Error escribiendo en el archivo".to_string());
            }
            Ok(format!("Archivo '{}' creado exitosamente", path))
        }
        Err(_) => return Err("No se pudo crear el archivo".to_string()),
    }
}

// /deletefile?name=filename
pub fn delete_file (name: &str) -> Result<String, String> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Nombre del archivo invàlido (Solo se permiten alfanùmericos)".to_string());
    }

    let folder = "archivos";
    let path = format!("{}/{}.txt", folder, name);
    let path_original = Path::new(&path);

    if !path_original.exists() {
        return Err(format!("El archivo '{}' no existe", path));
    }

    match remove_file(&path_original) {
        Ok(_) => Ok(format!("Archivo '{}' eliminado exitosamente", path)),
        Err(_) => Err(format!("No se pudo eliminar el archivo '{}'", path)),
    }
}

// / reverse?text=abc
pub fn rerverse_text(input: &str) -> String{
    input.chars().rev().collect()
}
// /toupper?text=abc
pub fn to_uppercase(input: &str) -> String {
    input.to_uppercase()
}

// /random?count=n&min=a&max=b

pub fn generate_random_numbers(count : usize, min: i32, max: i32) -> Vec<i32>{
    let mut rng = rand::rng();
    (0..count).map(|_| rng.random_range(min..=max)).collect()
}

// /timestamp
pub fn timestamp_iso() -> String {
    let now = std::time::SystemTime::now();

    //Convertimos en formato ISO
    let datetime: DateTime<Utc> = now.into();
    let iso = datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    format!("{}", iso)
}

// /hash?text=abc
pub fn sha256_hash(input: &str) -> String{
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

//Parte de calculo de pi con MonteCarlo
pub fn calculate_monte_carlo(points: u64) -> u64 {
    let mut rng = rand::rng();
    let mut hits = 0;

    for _ in 0..points {
        let x: f64 = rng.random();
        let y: f64 = rng.random();

        //Comprueba si el punto (x, y) esta dentro del circulo
        if x * x + y * y <= 1.0 {
            hits += 1;
        }
    }
    return hits;
}
