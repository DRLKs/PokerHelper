# Probar CI

Usar act 

```bash
sudo pacman -S act 
act -j test    # Solo job de tests
act -j clippy  # Solo job de clippy
act -j fmt     # Solo job de formato
act            # Todos los jobs
```

# Arreglar formato

```bash
cargo fmt 
```

# Ejecutar Tests
```bash
cd src-tauri
cargo test --verbose
```

# Usar clipppy
```bash
cd src-tauri
cargo clippy -- -D warnings
```