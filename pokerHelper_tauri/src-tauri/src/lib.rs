use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{State, Manager};

// Estado para mantener el proceso Java
struct JavaService {
    process: Arc<Mutex<Option<Child>>>,
}

fn find_jar_file(service_path: &str) -> Option<String> {
    use std::fs;
    
    let target_dir = format!("{}/target", service_path);
    
    if let Ok(entries) = fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                
                // Misma lógica que el script bash
                if file_name_str.ends_with(".jar") && 
                   !file_name_str.contains("-sources") && 
                   !file_name_str.contains("-javadoc") {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    None
}

// Función para obtener argumentos JVM específicos del SO (consistentes con scripts)
fn get_optimized_jvm_args() -> Vec<String> {
    let mut args = vec![
        "-XX:+UseG1GC".to_string(),
        "-Xms128m".to_string(),  // Consistente con ambos scripts
        "-Xmx512m".to_string(),  // Consistente con ambos scripts
        "-XX:+TieredCompilation".to_string(),
        "-XX:TieredStopAtLevel=1".to_string(),
        "-XX:+UseStringDeduplication".to_string(),
        "-Djavalin.showJavalinBanner=false".to_string(),
    ];

    // Argumentos específicos del sistema operativo
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        args.extend([
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+UseJVMCICompiler".to_string(),
            "-XX:+OptimizeStringConcat".to_string(),
            "-Djava.security.egd=file:/dev/./urandom".to_string(),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        // En Windows omitimos algunos argumentos que pueden causar problemas
        // y añadimos optimizaciones específicas de Windows
        args.push("-Djava.awt.headless=true".to_string());
    }

    args
}

#[tauri::command]
async fn start_java_service(service: State<'_, JavaService>) -> Result<String, String> {
    // Verificar si ya hay un servicio ejecutándose externamente
    if is_service_already_running().await {
        println!("Found existing Java service running on port 8080");
        return Ok("Java service is already running (external)".to_string());
    }

    {
        let process_guard = service.process.lock().unwrap();
        if process_guard.is_some() {
            return Ok("Java service is already running".to_string());
        }
    }

    let java_service_path = "../../services/java";
    
    if !std::path::Path::new(java_service_path).exists() {
        return Err(format!("Java service directory not found: {}", java_service_path));
    }
    
    // Buscar JAR automáticamente (como en start-fast.sh)
    let jar_path = find_jar_file(java_service_path)
        .unwrap_or_else(|| format!("{}/target/poker-probability-service-1.0.0.jar", java_service_path));
    
    // Solo compilar si el JAR no existe o está desactualizado
    if !std::path::Path::new(&jar_path).exists() || should_rebuild_jar(java_service_path, &jar_path) {
        println!("Compiling Java service (JAR not found or outdated)...");
        
        let compile_result = Command::new("mvn")
            .args(&["package", "-DskipTests", "-q", "-T", "1C"])
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
                
                // Buscar el JAR de nuevo después de compilar
                if let Some(new_jar) = find_jar_file(java_service_path) {
                    println!("Found compiled JAR: {}", new_jar);
                } else {
                    return Err("No JAR file found after compilation".to_string());
                }
            }
            Err(e) => {
                return Err(format!("Failed to execute Maven: {}", e));
            }
        }
    } else {
        println!("Using existing JAR: {}", jar_path);
    }
    
    // Intentar usar el script optimizado específico del SO si existe
    if let Some(result) = try_startup_script(&service, java_service_path).await {
        result
    } else {
        start_java_direct(&service, java_service_path, &jar_path).await
    }
}

// Función auxiliar para iniciar Java directamente
async fn start_java_direct(service: &State<'_, JavaService>, java_service_path: &str, jar_path: &str) -> Result<String, String> {
    println!("Starting Java service directly...");
    
    let mut jvm_args = get_optimized_jvm_args();
    jvm_args.push("-jar".to_string());
    
    // Extraer solo el nombre del archivo JAR para el comando
    let jar_file = std::path::Path::new(jar_path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| format!("target/{}", s))
        .unwrap_or_else(|| jar_path.to_string());
    
    jvm_args.push(jar_file);
    
    let process = Command::new("java")
        .args(&jvm_args)
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
            println!("Java process started, waiting for service to be ready...");
            
            if wait_for_service_ready().await {
                println!("Java service is healthy and ready!");
                Ok("Java service started successfully and is healthy".to_string())
            } else {
                println!("Java service started but failed to become ready in time");
                Ok("Java service started but health check timeout".to_string())
            }
        }
        Err(e) => {
            Err(format!("Failed to start Java service: {}", e))
        }
    }
}

#[tauri::command]
async fn check_java_service() -> Result<bool, String> {
    // Verificar si el servicio Java está ejecutándose con timeout corto
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1000)) // Timeout de 1 segundo
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
        
    match client.get("http://localhost:8080/health").send().await {
        Ok(response) => {
            let status = response.status().is_success();
            Ok(status)
        },
        Err(_) => {
            // No mostrar error, simplemente devolver false
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

#[tauri::command]
async fn diagnose_java_service() -> Result<Vec<String>, String> {
    let java_service_path = "../../services/java";
    
    if !std::path::Path::new(java_service_path).exists() {
        return Err(format!("Java service directory not found: {}", java_service_path));
    }
    
    let issues = diagnose_startup_issues(java_service_path);
    Ok(issues)
}

// Función para verificar si necesitamos recompilar el JAR
fn should_rebuild_jar(service_path: &str, jar_path: &str) -> bool {
    use std::fs;
    
    let jar_metadata = match fs::metadata(jar_path) {
        Ok(metadata) => metadata,
        Err(_) => return true, // Si no podemos leer el JAR, recompilamos
    };
    
    let jar_modified = jar_metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
    
    // Verificar si algún archivo fuente es más nuevo que el JAR
    let src_path = format!("{}/src", service_path);
    let pom_path = format!("{}/pom.xml", service_path);
    
    // Verificar pom.xml
    if let Ok(pom_metadata) = fs::metadata(&pom_path) {
        if let Ok(pom_modified) = pom_metadata.modified() {
            if pom_modified > jar_modified {
                return true;
            }
        }
    }
    
    // Verificar archivos fuente (simplificado, solo verificamos la carpeta src en general)
    if let Ok(src_metadata) = fs::metadata(&src_path) {
        if let Ok(src_modified) = src_metadata.modified() {
            if src_modified > jar_modified {
                return true;
            }
        }
    }
    
    false
}

// Función para esperar a que el servicio esté listo con polling optimizado para Javalin
async fn wait_for_service_ready() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(300)) // Timeout aún más corto para Javalin
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    
    let max_attempts = 15; // Máximo 6 segundos para Javalin (más rápido que Spring Boot)
    let mut attempt = 0;
    
    println!("Checking service health...");
    
    while attempt < max_attempts {
        attempt += 1;
        
        match client.get("http://localhost:8080/health").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Javalin service ready after {} attempts ({:.1}s)", attempt, attempt as f64 * 0.4);
                    return true;
                }
            },
            Err(_) => {
                // El servicio aún no está listo, continuar esperando
            }
        }
        
        // Espera más corta para Javalin (inicia más rápido)
        let sleep_duration = if attempt < 4 {
            Duration::from_millis(100) // 100ms para los primeros intentos
        } else if attempt < 8 {
            Duration::from_millis(300) // 300ms para intentos medios
        } else {
            Duration::from_millis(500) // 500ms para intentos finales
        };
        
        tokio::time::sleep(sleep_duration).await;
    }
    
    println!("Javalin service health check timeout after {} attempts", attempt);
    false
}

// Función para verificar si ya hay un servicio ejecutándose en el puerto
async fn is_service_already_running() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(300)) // Timeout corto para verificación rápida
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
        
    match client.get("http://localhost:8080/health").send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

// Función para diagnosticar problemas comunes de inicio
fn diagnose_startup_issues(java_service_path: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Verificar si Maven está instalado
    if Command::new("mvn").arg("-version").output().is_err() {
        issues.push("Maven no está instalado o no está en el PATH".to_string());
    }
    
    // Verificar si Java está instalado
    if Command::new("java").arg("-version").output().is_err() {
        issues.push("Java no está instalado o no está en el PATH".to_string());
    }
    
    // Verificar si pom.xml existe
    if !std::path::Path::new(&format!("{}/pom.xml", java_service_path)).exists() {
        issues.push("pom.xml no encontrado en el directorio del servicio".to_string());
    }
    
    // Verificar puerto específico por SO
    check_port_usage(&mut issues);
    
    issues
}

// Función para verificar uso de puerto específico por SO
fn check_port_usage(issues: &mut Vec<String>) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("netstat").args(&["-an"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains(":8080") {
                issues.push("Puerto 8080 puede estar en uso por otro proceso (netstat)".to_string());
            }
        }
    }
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        if let Ok(output) = Command::new("lsof").args(&["-i", ":8080"]).output() {
            if !output.stdout.is_empty() {
                issues.push("Puerto 8080 puede estar en uso por otro proceso (lsof)".to_string());
            }
        }
    }
}

// Función para intentar usar el script de inicio específico del SO
async fn try_startup_script(service: &State<'_, JavaService>, java_service_path: &str) -> Option<Result<String, String>> {
    let (script_path, command, args) = get_startup_script_info(java_service_path)?;
    
    if !std::path::Path::new(&script_path).exists() {
        return None;
    }
    
    println!("Using startup script: {}", script_path);
    
    let process = Command::new(command)
        .args(args)
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
            println!("Java service started using script, waiting for readiness...");
            
            let result = if wait_for_service_ready().await {
                println!("Java service is healthy and ready!");
                Ok("Java service started using script and is healthy".to_string())
            } else {
                println!("Java service started but failed to become ready in time");
                Ok("Java service started using script but health check timeout".to_string())
            };
            
            Some(result)
        }
        Err(e) => {
            println!("Failed to use script, will fallback to direct Java execution: {}", e);
            None // Retornar None para indicar que debe usar fallback
        }
    }
}

// Función para obtener información del script específico del SO
fn get_startup_script_info(java_service_path: &str) -> Option<(String, String, Vec<String>)> {
    #[cfg(target_os = "windows")]
    {
        let script_path = format!("{}/start-fast.bat", java_service_path);
        Some((script_path.clone(), "cmd".to_string(), vec!["/C".to_string(), script_path]))
    }
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let script_path = format!("{}/start-fast.sh", java_service_path);
        Some((script_path.clone(), "bash".to_string(), vec![script_path]))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        // Para otros sistemas operativos, intentar con bash como fallback
        let script_path = format!("{}/start-fast.sh", java_service_path);
        Some((script_path.clone(), "bash".to_string(), vec![script_path]))
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
        .invoke_handler(tauri::generate_handler![start_java_service, check_java_service, stop_java_service, diagnose_java_service, get_startup_info])
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

#[tauri::command]
async fn get_startup_info() -> Result<serde_json::Value, String> {
    let java_service_path = "../../services/java";
    
    if !std::path::Path::new(java_service_path).exists() {
        return Err("Java service directory not found".to_string());
    }
    
    let mut info = serde_json::Map::new();
    
    // Información del sistema operativo
    info.insert("os".to_string(), serde_json::Value::String(std::env::consts::OS.to_string()));
    
    // Verificar qué scripts están disponibles
    let startup_scripts = check_available_scripts(java_service_path);
    info.insert("available_scripts".to_string(), serde_json::Value::Array(startup_scripts));
    
    // Verificar si el JAR existe
    if let Some(jar_path) = find_jar_file(java_service_path) {
        info.insert("jar_found".to_string(), serde_json::Value::String(jar_path));
    } else {
        info.insert("jar_found".to_string(), serde_json::Value::Null);
    }
    
    // Estado del servicio
    let service_running = is_service_already_running().await;
    info.insert("service_running".to_string(), serde_json::Value::Bool(service_running));
    
    Ok(serde_json::Value::Object(info))
}

// Función para verificar qué scripts de inicio están disponibles
fn check_available_scripts(java_service_path: &str) -> Vec<serde_json::Value> {
    let mut scripts = Vec::new();
    
    // Script de Windows
    let bat_path = format!("{}/start-fast.bat", java_service_path);
    if std::path::Path::new(&bat_path).exists() {
        scripts.push(serde_json::json!({
            "name": "start-fast.bat",
            "type": "windows",
            "path": bat_path
        }));
    }
    
    // Script de Unix
    let sh_path = format!("{}/start-fast.sh", java_service_path);
    if std::path::Path::new(&sh_path).exists() {
        scripts.push(serde_json::json!({
            "name": "start-fast.sh",
            "type": "unix",
            "path": sh_path
        }));
    }
    
    scripts
}