#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# SchoolCBB v2 — Instalador Docker
#   ./setup.sh              → instalar o actualizar
#   ./setup.sh --dev        → modo desarrollo (con hot-reload)
#   ./setup.sh --status     → estado de los servicios
#   ./setup.sh --logs       → logs en tiempo real
#   ./setup.sh --stop       → detener todo
#   ./setup.sh --reset      → reinicio completo (destructivo)
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; MAGENTA='\033[0;35m'; NC='\033[0m'
BOLD='\033[1m'; DIM='\033[2m'

info()  { echo -e "  ${CYAN}▸${NC} $1"; }
ok()    { echo -e "  ${GREEN}✔${NC} $1"; }
warn()  { echo -e "  ${YELLOW}⚠${NC} $1"; }
err()   { echo -e "  ${RED}✘${NC} $1"; }
hr()    { echo -e "  ${DIM}────────────────────────────────────────${NC}"; }

# ─── Banner ──────────────────────────────────────────────────
echo ""
echo -e "${CYAN}${BOLD}"
echo "   ╔═══════════════════════════════════════════════╗"
echo "   ║         SchoolCBB v2 — Instalador             ║"
echo "   ║   Plataforma Escolar Multi-Tenant en Rust     ║"
echo "   ╚═══════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "   ${DIM}Documentación: https://schoolccb.cl${NC}"
echo ""

# ─── Comandos rápidos ────────────────────────────────────────
case "${1:-install}" in
  --status|status)
    echo -e "  ${BOLD}Estado de servicios${NC}\n"
    docker compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || warn "No ejecutando"
    exit 0
    ;;
  --logs|logs)
    shift; docker compose logs -f "$@"
    exit 0
    ;;
  --stop|stop)
    info "Deteniendo servicios..."
    docker compose down
    ok "Servicios detenidos"
    exit 0
    ;;
  --reset|reset)
    echo -e "  ${RED}${BOLD}⚠  ¡RESET COMPLETO!${NC}"
    warn "Esto eliminará TODOS los datos (volúmenes incluidos)."
    read -r -p "$(echo -e "  ${RED}¿Continuar? (escribe 'reset' para confirmar): ${NC}")" confirm
    if [ "$confirm" != "reset" ]; then
      info "Cancelado."
      exit 0
    fi
    info "Deteniendo y eliminando volúmenes..."
    docker compose down -v
    rm -f .env
    ok "Reset completado. Vuelve a ejecutar ./setup.sh"
    exit 0
    ;;
  --dev|dev)
    MODE="dev"
    ;;
  --install|install)
    MODE="install"
    ;;
  *)
    MODE="install"
    ;;
esac

# ═══════════════════════════════════════════════════════════════
# 1. Verificar dependencias del sistema
# ═══════════════════════════════════════════════════════════════
echo -e "  ${BOLD}1. Requisitos del sistema${NC}\n"

check_req() {
  local cmd="$1" name="$2" url="$3"
  if command -v "$cmd" &>/dev/null; then
    ok "$name — $(command -v $cmd)"
  else
    err "$name no encontrado."
    echo -e "     ${DIM}Instalar: $url${NC}"
    FAIL=1
  fi
}

check_req "docker" "Docker" "https://docs.docker.com/engine/install/"
check_req "curl" "curl" "apt install curl"
check_req "openssl" "OpenSSL" "apt install openssl"

if [ "${FAIL:-0}" = "1" ]; then
  echo ""; err "Instala los requisitos faltantes y vuelve a ejecutar."
  exit 1
fi

# Verificar Docker Compose
if docker compose version &>/dev/null; then
  ok "Docker Compose — $(docker compose version | cut -d' ' -f4)"
else
  err "Docker Compose plugin no disponible."
  info "Actualiza Docker Desktop o instalá docker-compose-plugin"
  exit 1
fi

# Verificar que Docker esté corriendo
if ! docker info &>/dev/null 2>&1; then
  err "Docker no está corriendo."
  info "Inicia el servicio Docker y vuelve a intentar."
  exit 1
fi

# ─── Hardware mínimo ──────────────────────────────────────────
MEM_MB=$(awk '/MemTotal/ {printf "%d", $2/1024}' /proc/meminfo 2>/dev/null || echo 0)
if [ "$MEM_MB" -gt 0 ] && [ "$MEM_MB" -lt 2000 ]; then
  warn "Memoria RAM: ${MEM_MB}MB (mínimo recomendado: 4GB)"
fi
DISK_KB=$(df / | awk 'NR==2 {print $4}')
if [ "$DISK_KB" -lt 5000000 ]; then
  warn "Espacio en disco: $((DISK_KB/1024))MB (mínimo recomendado: 5GB)"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# 2. Configuración interactiva
# ═══════════════════════════════════════════════════════════════
echo -e "  ${BOLD}2. Configuración de la plataforma${NC}\n"

prompt() {
  local var="$1" msg="$2" default="${3:-}"
  read -r -p "$(echo -e "  ${CYAN}${msg}${NC} [${default}]: ")" VAL
  eval "$var=\"${VAL:-$default}\""
}

prompt COMPANY_NAME "Nombre de la empresa" "SchoolCBB"
prompt DOMAIN "Dominio principal (sin http)" "localhost"

if [ "$DOMAIN" != "localhost" ]; then
  prompt APP_SUBDOMAIN "Subdominio de la app" "app"
  APP_URL="https://${APP_SUBDOMAIN}.${DOMAIN}"
  PORTAL_URL="https://${DOMAIN}"
else
  APP_URL="http://localhost:8080"
  PORTAL_URL="http://localhost:3010"
fi

echo ""
info "Gerente General (superadmin con acceso al CRM y licencias)"
prompt GERENTE_EMAIL "Email del Gerente General" "admin@${DOMAIN}"
read -r -s -p "$(echo -e "  ${CYAN}Contraseña${NC} [admin123]: ")" GERENTE_PASSWORD
GERENTE_PASSWORD="${GERENTE_PASSWORD:-admin123}"
echo ""

prompt DB_USER "Usuario PostgreSQL" "schoolccb"
read -r -s -p "$(echo -e "  ${CYAN}Contraseña PostgreSQL${NC} [schoolccb]: ")" DB_PASS
DB_PASS="${DB_PASS:-schoolccb}"
echo ""
prompt DB_NAME "Base de datos PostgreSQL" "schoolccb"

# ─── Generar secretos ─────────────────────────────────────────
JWT_SECRET=$(openssl rand -hex 32 2>/dev/null || echo "cambio-en-produccion-por-favor")
INTERNAL_API_SECRET=$(openssl rand -hex 32 2>/dev/null || echo "dev-secret")
DB_URL="postgres://${DB_USER}:${DB_PASS}@db:5432/${DB_NAME}"

echo ""

# ═══════════════════════════════════════════════════════════════
# 3. Generar .env
# ═══════════════════════════════════════════════════════════════
echo -e "  ${BOLD}3. Generando configuración${NC}\n"

if [ -f .env ] && [ "${MODE}" != "dev" ]; then
  warn "Archivo .env ya existe. Haz backup: cp .env .env.backup"
fi

cat > .env << ENVEOF
# ═══════════════════════════════════════════════════════════════
# SchoolCBB v2 — Generado por setup.sh el $(date "+%Y-%m-%d %H:%M")
# ═══════════════════════════════════════════════════════════════

# ─── Generales ────────────────────────────────────────────────
COMPANY_NAME=${COMPANY_NAME}
DOMAIN=${DOMAIN}
APP_URL=${APP_URL}
PORTAL_URL=${PORTAL_URL}

# ─── Gerente General (Superadmin) ─────────────────────────────
GERENTE_EMAIL=${GERENTE_EMAIL}
GERENTE_PASSWORD=${GERENTE_PASSWORD}
GERENTE_NAME=Admin

# ─── Base de Datos ────────────────────────────────────────────
DATABASE_URL=${DB_URL}
DB_USER=${DB_USER}
DB_PASSWORD=${DB_PASS}
DB_NAME=${DB_NAME}

# ─── JWT ──────────────────────────────────────────────────────
JWT_SECRET=${JWT_SECRET}
INTERNAL_API_SECRET=${INTERNAL_API_SECRET}

# ─── Gateway ──────────────────────────────────────────────────
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=3000
FRONTEND_URL=${APP_URL}
IDENTITY_URL=http://identity:3001
SIS_URL=http://sis:3002
ACADEMIC_URL=http://academic:3003
ATTENDANCE_URL=http://attendance:3004
NOTIFICATIONS_URL=http://notifications:3005
FINANCE_URL=http://finance:3006
REPORTING_URL=http://reporting:3007
PORTAL_URL_INTERNAL=http://portal:3010
CURRICULUM_URL=http://curriculum:3011
CRM_URL=http://crm:3012

# ─── Servicios ────────────────────────────────────────────────
IDENTITY_HOST=0.0.0.0; IDENTITY_PORT=3001
SIS_HOST=0.0.0.0; SIS_PORT=3002
ACADEMIC_HOST=0.0.0.0; ACADEMIC_PORT=3003
ATTENDANCE_HOST=0.0.0.0; ATTENDANCE_PORT=3004
NOTIFICATIONS_HOST=0.0.0.0; NOTIFICATIONS_PORT=3005
FINANCE_HOST=0.0.0.0; FINANCE_PORT=3006; FINANCE_GRPC_URL=http://finance:4006
REPORTING_HOST=0.0.0.0; REPORTING_PORT=3007
PORTAL_HOST=0.0.0.0; PORTAL_PORT=3010
CURRICULUM_HOST=0.0.0.0; CURRICULUM_PORT=3011; CURRICULUM_KB_DIR=.agents/skills/curriculo-chile
CRM_HOST=0.0.0.0; CRM_PORT=3012

# ─── Curriculum (CN Chile) ────────────────────────────────────
CURRICULUM_KB_DIR=.agents/skills/curriculo-chile

# ─── SMTP (Correo — opcional) ─────────────────────────────────
# SMTP_HOST=smtp.gmail.com
# SMTP_PORT=587
# SMTP_USERNAME=
# SMTP_PASSWORD=
# SMTP_FROM=SchoolCBB <noreply@schoolccb.cl>
# PDF_OUTPUT_DIR=/tmp/proposals

# ─── Logging ──────────────────────────────────────────────────
RUST_LOG=info,schoolccb=debug
ENVEOF

ok ".env generado"

# ═══════════════════════════════════════════════════════════════
# 4. Construir imágenes
# ═══════════════════════════════════════════════════════════════
echo ""
echo -e "  ${BOLD}4. Construyendo imágenes Docker${NC}\n"

BUILD_FLAGS=""
if [ "${MODE}" = "dev" ]; then
  BUILD_FLAGS="--build-arg BUILD_MODE=dev"
  info "Modo desarrollo — construcción con caché optimizada"
fi

info "Construyendo servicios backend (11 microservicios)..."
docker compose build --parallel $BUILD_FLAGS 2>&1 | while IFS= read -r line; do
  echo -e "  ${DIM}${line}${NC}"
done
ok "Imágenes construidas"

# ═══════════════════════════════════════════════════════════════
# 5. Iniciar servicios
# ═══════════════════════════════════════════════════════════════
echo ""
echo -e "  ${BOLD}5. Iniciando servicios${NC}\n"

if [ "${MODE}" = "dev" ]; then
  info "Iniciando base de datos para desarrollo local..."
  docker compose up -d db
  ok "Base de datos lista en localhost:5432"
  echo ""
  info "Para desarrollo local, ejecuta los servicios individualmente:"
  echo -e "  ${DIM}  cargo run --release -p schoolccb-identity${NC}"
  echo -e "  ${DIM}  cargo run --release -p schoolccb-gateway${NC}"
  echo -e "  ${DIM}  cd packages/frontend && dx serve${NC}"
  echo ""
  info "O inicia todo con: docker compose up -d"
else
  info "Iniciando todos los servicios..."
  docker compose up -d
  echo ""

  # ─── Health check polling ─────────────────────────────────
  info "Esperando health checks..."
  for i in $(seq 1 60); do
    HEALTHY=$(docker compose ps --format json 2>/dev/null | python3 -c "
import json, sys
try:
    services = [json.loads(l) for l in sys.stdin]
    healthy = [s for s in services if s.get('Health') == 'healthy']
    print(len(healthy))
except: print(0)
" 2>/dev/null || echo "0")
    if [ "$HEALTHY" -ge 12 ] 2>/dev/null; then
      ok "Todos los servicios saludables"
      break
    fi
    if [ "$i" -eq 60 ]; then
      warn "Tiempo de espera agotado. Revisa: docker compose logs -f"
    fi
    sleep 2
  done
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# 6. Resumen Final
# ═══════════════════════════════════════════════════════════════
echo -e "${GREEN}${BOLD}"
echo "   ╔═══════════════════════════════════════════════╗"
echo "   ║       Instalación completada                  ║"
echo "   ╚═══════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "  ${CYAN}┃${NC}  Empresa:        ${BOLD}${COMPANY_NAME}${NC}"
echo -e "  ${CYAN}┃${NC}  Frontend:       ${BOLD}${APP_URL}${NC}"
echo -e "  ${CYAN}┃${NC}  API Gateway:    ${BOLD}http://localhost:3000${NC}"
echo -e "  ${CYAN}┃${NC}  Portal Público: ${BOLD}${PORTAL_URL}${NC}"
echo ""
echo -e "  ${CYAN}┃${NC}  Gerente General: ${BOLD}${GERENTE_EMAIL}${NC}"
echo ""
echo -e "  ${DIM}  Comandos útiles:${NC}"
echo -e "  ${DIM}  ./setup.sh --status   → estado de servicios${NC}"
echo -e "  ${DIM}  ./setup.sh --logs     → logs en tiempo real${NC}"
echo -e "  ${DIM}  ./setup.sh --stop     → detener todo${NC}"
echo -e "  ${DIM}  ./setup.sh --reset    → reinicio completo${NC}"
echo -e "  ${DIM}  docker compose logs -f → logs detallados${NC}"
echo ""
