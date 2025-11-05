# PokerHelper - Estructura del Proyecto

```
PokerHelper/
├── src-tauri/                    # Backend Rust (Tauri)
│   ├── src/
│   │   └── main.rs               # Punto de entrada (stub)
│   ├── crates/
│   │   └── poker_agent/          # Crate "Brain" 
│   │       ├── src/
│   │       │   ├── lib.rs        # Módulo principal
│   │       │   ├── game_state.rs # Estructuras de datos
│   │       │   ├── probability.rs # Cálculos de probabilidad
│   │       │   └── decision.rs   # Lógica de decisión
│   │       └── Cargo.toml
│   ├── binaries/
│   │   ├── cv_sidecar.py         # Agente CV Python (stub)
│   │   └── requirements.txt      # Dependencias Python
│   ├── Cargo.toml                # Configuración Rust
│   ├── tauri.conf.json           # Configuración Tauri
│   └── build.rs
├── src/
│   └── main.js                   # Frontend (stub)
├── config/
│   └── cv_config.json            # Coordenadas de pantalla
├── tests/                        # Carpeta para tests
├── index.html                    # UI principal
├── vite.config.js                # Configuración Vite
└── .gitignore
```
