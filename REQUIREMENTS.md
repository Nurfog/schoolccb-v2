# Requisitos del Sistema — SchoolCBB v2

## Requisitos Mínimos

| Componente | Versión | Propósito |
|-----------|---------|-----------|
| Rust | 1.88+ (`rust-toolchain.toml`) | Compilación de todos los servicios backend |
| Rust WASM target | `wasm32-unknown-unknown` | Compilación del frontend Dioxus |
| PostgreSQL | 16+ | Base de datos principal |
| Docker | 24+ | Contenedores (entorno recomendado) |
| Docker Compose | v2+ | Orquestación de servicios |

---

## Instalación Rápida (Docker — Recomendado)

### 1. Dependencias del sistema

```bash
# Debian / Ubuntu
sudo apt-get update
sudo apt-get install -y curl git build-essential pkg-config libssl-dev protobuf-compiler

# Arch Linux
sudo pacman -S curl git base-devel pkgconf openssl protobuf

# macOS
brew install curl git pkg-config openssl protobuf
```

### 2. Docker

```bash
# Linux
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER  # cerrar sesión y volver a entrar

# macOS / Windows
# Descargar Docker Desktop: https://docs.docker.com/get-docker/
```

### 3. Clonar y configurar

```bash
git clone <repo-url> schoolccb-v2
cd schoolccb-v2
cp .env.example .env
# Editar .env si es necesario (JWT_SECRET, passwords, etc.)
```

### 4. Iniciar

```bash
docker compose up -d --build
```

Esto construye e inicia 12 servicios:
- `db` — PostgreSQL 16
- `identity` — Auth, usuarios, roles, corporaciones
- `sis` — Student Information System (alumnos, cursos, RRHH)
- `academic` — Notas, periodos académicos, calendario
- `attendance` — Asistencia diaria
- `notifications` — WebSocket, notificaciones, comunicaciones
- `finance` — Cobros, pagos, gateway de pago
- `reporting` — Certificados, informes, exportación SIGE
- `portal` — Sitio web público (minijinja)
- `curriculum` — Currículum Nacional de Chile (KB)
- `crm` — CRM de ventas B2B (prospectos → contratos → activación)
- `gateway` — API Gateway + GraphQL (punto de entrada)
- `frontend` — SPA Dioxus/WASM servida por Nginx

### 5. Acceder

| URL | Descripción |
|-----|-------------|
| `http://localhost:8080` | Frontend (SPA Dioxus) |
| `http://localhost:3000` | API Gateway |
| `http://localhost:3010` | Portal público |

---

## Instalación Manual (Desarrollo — Sin Docker)

### 1. Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.88
rustup target add wasm32-unknown-unknown
```

### 2. Dioxus CLI

```bash
cargo install dioxus-cli --version 0.6.3
```

### 3. PostgreSQL

```bash
# Debian / Ubuntu
sudo apt-get install -y postgresql postgresql-contrib
sudo systemctl start postgresql

# Crear usuario y base de datos
sudo -u postgres psql -c "CREATE USER schoolccb WITH PASSWORD 'schoolccb';"
sudo -u postgres psql -c "CREATE DATABASE schoolccb OWNER schoolccb;"
```

### 4. Dependencias de compilación

```bash
# Debian / Ubuntu
sudo apt-get install -y pkg-config libssl-dev protobuf-compiler

# Arch Linux
sudo pacman -S pkgconf openssl protobuf

# macOS
brew install pkg-config openssl protobuf
```

### 5. Configurar entorno

```bash
cp .env.example .env
# Editar DATABASE_URL si usas credenciales distintas
```

### 6. Compilar servicios

```bash
# Todos los servicios backend (menos frontend)
cargo build --release --workspace --exclude schoolccb-frontend

# Frontend (Dioxus WASM)
cd packages/frontend
dx build --release --platform web
```

### 7. Iniciar servicios

Cada servicio es un binario independiente. Iniciar en terminales separadas:

```bash
# Terminal 1: Identity (puerto 3001)
cargo run --release -p schoolccb-identity

# Terminal 2: SIS (puerto 3002)
cargo run --release -p schoolccb-sis

# Terminal 3: Academic (puerto 3003)
cargo run --release -p schoolccb-academic

# Terminal 4: Attendance (puerto 3004)
cargo run --release -p schoolccb-attendance

# Terminal 5: Notifications (puerto 3005)
cargo run --release -p schoolccb-notifications

# Terminal 6: Finance (puerto 3006)
cargo run --release -p schoolccb-finance

# Terminal 7: Reporting (puerto 3007)
cargo run --release -p schoolccb-reporting

# Terminal 8: Portal (puerto 3010)
cargo run --release -p schoolccb-portal

# Terminal 9: Curriculum (puerto 3011)
cargo run --release -p schoolccb-curriculum

# Terminal 10: CRM (puerto 3012)
cargo run --release -p schoolccb-crm

# Terminal 11: Gateway (puerto 3000)
cargo run --release -p schoolccb-gateway
```

O usar `cargo run --release -p schoolccb-gateway` (depende de los demás).

Servir frontend:
```bash
cd packages/frontend
dx serve --platform web --port 8080
```

---

## Puertos

| Servicio | Puerto interno | Puerto externo (Docker) |
|----------|---------------|------------------------|
| Gateway | 3000 | 3000 |
| Identity | 3001 | — |
| SIS | 3002 | — |
| Academic | 3003 | — |
| Attendance | 3004 | — |
| Notifications | 3005 | — |
| Finance | 3006 | — |
| Reporting | 3007 | — |
| Portal | 3010 | 3010 |
| Curriculum | 3011 | — |
| CRM | 3012 | — |
| Frontend (Nginx) | 80 | 8080 |
| PostgreSQL | 5432 | 5432 |

---

## Variables de Entorno (`.env`)

| Variable | Default | Descripción |
|----------|---------|-------------|
| `COMPANY_NAME` | `SchoolCBB` | Nombre de la empresa |
| `DOMAIN` | `localhost` | Dominio principal |
| `APP_URL` | `http://localhost:8080` | URL del frontend |
| `GERENTE_EMAIL` | — | Email del Gerente General (superadmin) |
| `GERENTE_PASSWORD` | — | Contraseña del Gerente General |
| `GERENTE_NAME` | `Juan Allende` | Nombre del Gerente General |
| `DATABASE_URL` | — | URL de conexión PostgreSQL |
| `JWT_SECRET` | — | Secreto para firmar JWT (generar con `openssl rand -hex 32`) |
| `CURRICULUM_KB_DIR` | `.agents/skills/curriculo-chile` | Ruta a la KB del Currículum Nacional |
| `RUST_LOG` | `info,schoolccb=debug` | Nivel de logging |

---

## Verificar Instalación

```bash
# Health checks
curl http://localhost:3000/health           # Gateway
curl http://localhost:3001/health           # Identity
curl http://localhost:3011/health           # Curriculum

# Curriculum KB debe cargar chunks
curl http://localhost:3011/api/curriculum/info
# → {"total_chunks": N, "status": "ok"}

# Frontend debe servir
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080
# → 200
```
