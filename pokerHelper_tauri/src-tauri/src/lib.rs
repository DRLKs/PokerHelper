use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{State, Manager};

 // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Estado para mantener el proceso Java
struct JavaService {
    process: Arc<Mutex<Option<Child>>>,
}

#[tauri::command]
async fn start_java_service(service: State<'_, JavaService>) -> Result<String, String> {
    {
        let process_guard = service.process.lock().unwrap();
        
        // Si ya hay un proceso ejecutándose, no iniciarlo de nuevo
        if process_guard.is_some() {
            return Ok("Java service is already running".to_string());
        }
    }

    let java_service_path = "../../services/java";
    
    // Debug: mostrar directorio actual
    if let Ok(current_dir) = std::env::current_dir() {
        println!("Current directory: {:?}", current_dir);
    }
    
    let full_path = std::path::Path::new(java_service_path).canonicalize();
    println!("Looking for Java service at: {:?}", full_path);
    
    // Verificar si el directorio existe
    if !std::path::Path::new(java_service_path).exists() {
        return Err(format!("Java service directory not found: {}", java_service_path));
    }
    
    // Compilar el JAR si no existe
    println!("Compiling Java service...");
    let compile_result = Command::new("mvn")
        .args(&["clean", "package", "-DskipTests", "-q"])
        .current_dir(java_service_path)
        .output();

    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(format!("Maven compilation failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr));
            }
            println!("Java service compiled successfully");
        }
        Err(e) => {
            return Err(format!("Failed to execute Maven: {}", e));
        }
    }
    
    // Intentar ejecutar el JAR compilado
    let jar_path = format!("{}/target/poker-probability-service-1.0.0.jar", java_service_path);
    println!("Attempting to start Java service using JAR: {}", jar_path);
    
    let process = Command::new("java")
        .args(&["-jar", "target/poker-probability-service-1.0.0.jar"])
        .current_dir(java_service_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match process {
        Ok(child) => {
            {
                let mut process_guard = service.process.lock().unwrap();
                *process_guard = Some(child);
            }
            println!("Java service started successfully");
            
            // Esperar un poco para que el servicio se inicie completamente
            println!("Waiting for service to be ready...");
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            // Verificar si el servicio está realmente funcionando
            let client = reqwest::Client::new();
            match client.get("http://localhost:8080/health").send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Java service is healthy and ready!");
                        Ok("Java service started successfully and is healthy".to_string())
                    } else {
                        println!("Java service started but health check failed with status: {}", response.status());
                        Ok("Java service started but health check failed".to_string())
                    }
                },
                Err(e) => {
                    println!("Java service started but health check failed: {}", e);
                    Ok("Java service started but health check failed".to_string())
                }
            }
        }
        Err(e) => {
            println!("Failed to start JAR, trying Maven exec:java - Error: {}", e);
            // Si falla el JAR, intentar con Maven directamente
            let maven_process = Command::new("mvn")
                .args(&["exec:java", "-Dexec.mainClass=com.pokerhelper.PokerHelperApplication", "-q"])
                .current_dir(java_service_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match maven_process {
                Ok(child) => {
                    {
                        let mut process_guard = service.process.lock().unwrap();
                        *process_guard = Some(child);
                    }
                    println!("Java service started using Maven");
                    println!("Waiting for Maven service to be ready...");
                    tokio::time::sleep(Duration::from_secs(8)).await;
                    
                    // Verificar si el servicio está funcionando
                    let client = reqwest::Client::new();
                    match client.get("http://localhost:8080/health").send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                println!("Maven Java service is healthy and ready!");
                                Ok("Java service started using Maven and is healthy".to_string())
                            } else {
                                println!("Maven Java service started but health check failed with status: {}", response.status());
                                Ok("Java service started using Maven but health check failed".to_string())
                            }
                        },
                        Err(e) => {
                            println!("Maven Java service started but health check failed: {}", e);
                            Ok("Java service started using Maven but health check failed".to_string())
                        }
                    }
                }
                Err(maven_error) => {
                    Err(format!("Failed to start Java service: JAR error: {}, Maven error: {}", e, maven_error))
                }
            }
        }
    }
}

#[tauri::command]
async fn check_java_service() -> Result<bool, String> {
    // Verificar si el servicio Java está ejecutándose
    let client = reqwest::Client::new();
    match client.get("http://localhost:8080/health").send().await {
        Ok(response) => {
            let status = response.status().is_success();
            println!("Health check response: {} - {}", response.status(), status);
            Ok(status)
        },
        Err(e) => {
            println!("Health check failed: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
async fn stop_java_service(service: State<'_, JavaService>) -> Result<String, String> {
    let mut process_guard = service.process.lock().unwrap();
    
    if let Some(mut process) = process_guard.take() {
        println!("Stopping Java service...");
        
        // Intentar terminar el proceso graciosamente
        if let Err(e) = process.kill() {
            eprintln!("Error killing Java process: {}", e);
        }
        
        // Esperar a que el proceso termine
        if let Err(e) = process.wait() {
            eprintln!("Error waiting for Java process to end: {}", e);
        }
        
        println!("Java service stopped");
        Ok("Java service stopped".to_string())
    } else {
        Ok("Java service was not running".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let java_service = JavaService {
        process: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(java_service)
        .invoke_handler(tauri::generate_handler![greet, start_java_service, check_java_service, stop_java_service])
        .setup(|app| {
            // Iniciar el servicio Java automáticamente al arrancar la aplicación
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let service = app_handle.state::<JavaService>();
                println!("Starting Java service...");
                match start_java_service(service).await {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("Failed to auto-start Java service: {}", e),
                }
            });
            Ok(())
        })
        .on_window_event(|_window, event| {
            // Manejar el cierre de la aplicación
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app_handle = _window.app_handle();
                let service = app_handle.state::<JavaService>();
                
                // Parar el servicio Java al cerrar la aplicación
                tauri::async_runtime::block_on(async {
                    match stop_java_service(service).await {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => eprintln!("Error stopping Java service: {}", e),
                    }
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}