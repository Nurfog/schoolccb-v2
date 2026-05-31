# Plan de Desarrollo — SchoolCBB v2

Basado en el flujo de trabajo descrito en `flujo.md` y contrastado con el estado actual del código.

---

> **⚠ Nota importante:** El sistema tiene **dos flujos de venta completamente distintos** que comparten datos pero son plataformas diferentes:
>
> 1. **Venta del Servicio (B2B)** — Etapa 1 del flujo. SchoolCBB vende su plataforma SaaS a colegios/corporaciones. El comprador es el colegio o corporación. El agente de ventas vende el software (planes, módulos, licencias).
> 2. **Venta de Matrículas (B2C)** — Etapa 2 del flujo, sección "proceso de ventas (modulo admision)". El colegio (ya cliente) vende matrículas a apoderados para que sus hijos estudien ahí. El comprador es el apoderado. El agente de admisión vende cupos en el colegio.
>
> Ambas comparten datos del colegio/corporación como cliente, pero los procesos, contratos, documentos tributarios y ciclos de pago son independientes.

> **⚠ Arquitectura de roles:** No existe superadmin global. La entidad máxima es la **corporación** (sostenedor). Cada corporación tiene su propio dashboard global con visibilidad de todos sus colegios. Cada colegio tiene su propio dashboard interno. El equipo de ventas de SchoolCBB (Gerente General, Jefe de Ventas, Agente de Ventas) opera el CRM B2B con su propio panel, pero no tiene acceso a los datos operacionales de los colegios.

---

## Resumen de Estado Actual

| Aspecto | Estado |
|---|---|
| Identity / Auth / JWT | ✅ Completado |
| RBAC (Roles y Permisos) | ✅ Completado (ajustar: remover Root, agregar roles de ventas y corporativos faltantes) |
| Licensing (Planes, Módulos, Pagos) | ✅ Completado (`packages/common/src/licensing.rs`, seeders) |
| CRM Ventas (Prospectos, Pipeline, Cotizaciones, Contratos, Documentos, Activación, Dashboard) | ✅ Completado |
| Módulo Académico (Cursos, Asignaturas, Notas) | ✅ Completado |
| Asistencia | ✅ Completado |
| RRHH (Empleados, Contratos, Remuneraciones) | ✅ Completado |
| Finanzas | ✅ Completado |
| SIGE / MINEDUC | ✅ Completado |
| Portal Público | ⬜ No iniciado |
| Portal Apoderado (my-portal) | ✅ Completo |
| Portal Alumno | ✅ Completo — notas, asistencia, horario, anotaciones, citas |
| Portal Corporación (Sostenedor) | ✅ Completo — KPIs, gráficos, tabla ordenable, filtros, exportación |
| Dashboard Corporativo | ✅ Completo — 6 endpoints + frontend completo con filtros, ranking, donut, morosidad |
| Dashboard por Colegio | ✅ Completo — 8 endpoints + frontend con gráficos, alertas, finanzas |
| Root Dashboard | ❌ Eliminar — no existe en la arquitectura del flujo |

---

## 🏢 Etapa 1 — Venta del Servicio (B2B — SchoolCBB a Colegios)

> **Objetivo:** CRM para que el equipo comercial de SchoolCBB venda la plataforma SaaS a colegios y corporaciones.
> **Comprador:** El colegio / corporación (cliente institucional).
> **Producto:** Planes de licencia (Básico, Profesional, Corporativo) con módulos activables.
> **Roles involucrados:** Gerente General, Jefe de Ventas, Agente de Ventas.

### 1.1 Estructura y Roles
- [x] Schema `crm_sales` en PostgreSQL (implementado como `crm_sales_*` en public por ahora)
- [x] Crate `packages/services/crm` con router y modelos base
- [x] Roles de Ventas: `GerenteGeneral`, `JefeVentas`, `AgenteVentas` en Identity y CRM
- [x] **Mejora:** Soporte para **RUT** en modelos de prospectos y corporaciones (validación chilena)
- [x] **Faltante:** Endpoints para gestión de equipo de ventas (crear agentes, asignar metas)

### 1.2 Interfaz de Usuario (Salesforce Style)
- [x] Layout específico `/sales` en Dioxus
- [x] **Pipeline Kanban** interactivo por etapas
- [x] **Prospect Detail** con vista 360° del cliente
- [x] **Contact Timeline** (Llamadas, Emails, WhatsApp, Notas)
- [x] **Quote Builder:** Implementar lógica de selección de módulos y cálculo de precios
- [x] **Contract Builder:** Generación de contrato basado en propuesta aprobada
- [x] **Document Viewer:** Visualización de documentos subidos y estado de verificación
- [x] **License Activator:** Wizard de activación final (Corporación + Licencia + Usuario Admin)

### 1.3 Proceso de Venta (Backend & API)

#### 1. Captación de Prospectos
- [x] **Bug/Faltante:** Integrar `POST /api/public/contact` del portal para crear prospectos en el CRM automáticamente (endpoint público)
- [x] `POST /api/sales/prospects` — creación manual
- [x] **Faltante:** `POST /api/sales/prospects/import` — importación masiva vía CSV
- [x] Tabla `crm_sales_prospects` con campos básicos

#### 2. Asignación y Seguimiento
- [x] `PUT /api/sales/prospects/{id}/assign` — asignación manual a agente
- [x] **Mejora:** Regla de asignación automática (Round-robin)
- [x] `POST /api/sales/prospects/{id}/activities` — registro de interacciones

#### 3. Propuestas y Negociación
- [x] `POST /api/sales/proposals` — creación de borradores
- [x] `PUT /api/sales/proposals/{id}/discount` — aplicación de descuentos (requiere JefeVentas+)
- [x] **Faltante:** Generación de PDF de la propuesta comercial

#### 4. Contratos y Documentación
- [x] `POST /api/sales/contracts` — creación de contrato
- [x] `POST /api/sales/contracts/{id}/documents` — subida de documentos (S3/Local)
- [x] `PUT /api/sales/contracts/{id}/verify-signatures` — validación interna
- [x] **Mejora:** Integración con servicio de firma electrónica (ej. Toku)

#### 5. Facturación B2B
- [x] **Faltante:** `POST /api/sales/contracts/{id}/invoice` — generación de factura standalone (sin integración con módulo `finance` porque finance opera sobre B2C/estudiantes)
- [x] **Mejora:** Soporte para impuestos (IVA) en contratos (tasa configurable, subtotal, impuesto calculado)

#### 6. Activación del Servicio
- [x] `POST /api/sales/contracts/{id}/activate` — creación de Corporación y Licencia
- [x] **Faltante:** Creación automática del **primer usuario (Sostenedor)** y envío de email de bienvenida
- [x] **Faltante:** Creación de un **Colegio por defecto** para la corporación recién creada

### 1.4 Dashboard Comercial
- [x] `GET /api/sales/dashboard/summary` — KPIs básicos
- [x] **Mejora:** Gráficos de embudo de ventas (funnel) y rendimiento por agente
- [x] **Mejora:** Reporte de ingresos proyectados basado en pipeline actual

---

## 🏫 Etapa 2 — Gestión Escolar + Venta de Matrículas (B2C — Colegio a Apoderados)

> **Objetivo:** Plataforma de gestión del colegio que incluye la venta de matrículas a apoderados (B2C), más toda la operación académica, RRHH y administrativa.
> **Comprador (matrícula):** El apoderado (cliente individual).
> **Producto (matrícula):** Cupos en cursos del colegio, con becas, descuentos y anexos (PIE, TEA, etc.).
> **Documentos tributarios (matrícula):** Facturas, boletas, etc. emitidas por el colegio al apoderado.
> **Conexión con Etapa 1:** Cuando un colegio compra el servicio SchoolCBB (Etapa 1), se crea su organización en el sistema y puede comenzar a operar su propio proceso de admisión y matrículas (Etapa 2).

### 2.1 Schema Exclusivo
- [ ] Migración: crear schema `colegios`
- [ ] Separar servicios escolares del CRM de ventas

### 2.2 UI Estilo ISAMS
- [ ] Layout corporativo con sidebar integrada (ya existe en Dioxus)
- [ ] Refinar componentes para estilo ISAMS (cards, tabs, breadcrumbs)

### 2.3 Roles del Colegio
- [ ] Agregar roles faltantes: `RepresentanteLegal`, `DirectorRRHH`, `ProfesorHonorario`, `Administrativo`, `AdminGlobal`
- [ ] Mapping de roles existentes vs. roles del flujo

### 2.4 Proceso de Venta de Matrículas (B2C — Admisión del Colegio)

> **Diferencia clave con Etapa 1:** Aquí el "vendedor" es el colegio (ya cliente de SchoolCBB) y el "comprador" es el apoderado. No se venden planes/licencias, se venden **cupos escolares** para alumnos. Los contratos son **contratos de prestación de servicios educativos** (no licencias de software). Los documentos tributarios los emite el **colegio** al apoderado, no SchoolCBB al colegio.

#### CRM Recibe Prospecto
- [ ] Integrar con etapa 1: cuando un contrato se activa, el colegio recibe los prospectos
- [ ] Endpoint `POST /api/admission/prospects` — ya existe parcialmente en `admission.rs`
- [ ] Mejorar frontend `AdmissionPage` con pipeline visual

#### Contacto por Agente de Ventas (Admisión)
- [x] Endpoint `POST /api/admission/prospects/{id}/activities` — registrar contacto
- [x] Endpoint `POST /api/admission/prospects/{id}/reminder` — programar recordatorio
- [x] Frontend: agenda de seguimientos con notificaciones

#### Preparación de Contrato (Matrícula)
- [x] Tabla `enrollment_contracts`: alumno, apoderados, anexos (PIE, TEA, etc.)
- [x] Endpoint `POST /api/admission/contracts` — crear contrato de matrícula
- [x] Endpoint `POST /api/admission/contracts/{id}/documents` — subir anexos
- [x] Frontend: Enrollment Contract Builder (tabla de contratos + pestañas)

#### Gestión de Pagos de Matrícula
- [ ] Integrar con módulo finance para pagos (débito, crédito, efectivo, transferencia)
- [ ] Endpoint `POST /api/admission/payments` — registrar pago
- [ ] Soporte para **becas**: tabla `scholarships`, endpoint `POST /api/admission/scholarships/apply`
- [ ] Frontend: Payment Gateway + Scholarship Application

#### Inscripción Final
- [x] Endpoint `POST /api/admission/enroll` — inscribir alumno en curso
- [x] Pago de matrícula: endpoint `POST /api/admission/contracts/{id}/pay` + botón en frontend
- [ ] Generar documentos tributarios (integración finance) — pendiente de requerimientos específicos
- [x] Frontend: confirmation screen con resumen

### 2.5 Proceso Académico (Cursos)

#### Asignaturas según CN
- [ ] Servicio `packages/services/curriculum` ya existe
- [ ] Verificar que los cursos carguen automáticamente las asignaturas del CN de Chile
- [ ] Frontend: vista de currículum por curso

#### Capacidad de Salas
- [ ] Ya existe `Classroom` con campo `capacity` en `admission.rs`
- [ ] Endpoint `GET /api/classrooms/{id}/availability` — verificar disponibilidad
- [ ] Frontend: indicador de capacidad en asignación de cursos

#### Asignaturas Complementarias
- [ ] Tabla `complementary_subjects` (no influyen en promedios)
- [ ] Endpoint `POST /api/courses/{id}/complementary-subjects` — añadir
- [ ] Frontend: sección separada en configuración del curso

### 2.6 RRHH (Contratación Docente y Administrativa)

#### Entrevistas y Selección
- [ ] Expandir módulo HR existente con flujo de entrevistas
- [ ] Tabla `interview_process`: candidato, puesto, entrevistador, resultado
- [ ] Endpoint `POST /api/hr/interviews` — registrar entrevista
- [ ] Frontend: pipeline de selección

#### Definición de Horarios
- [ ] Tabla `teacher_schedules`: profesor, día, hora, tipo (clase / permanencia)
- [ ] Tabla `substitute_schedule` para reemplazos
- [ ] Endpoint `POST /api/hr/schedules` — guardar horario
- [ ] Frontend: Schedule Grid (arrastrar y soltar)

### 2.7 UTP (Unidad Técnico Pedagógica)

#### Calendario Académico
- [x] Tabla `academic_calendar` + endpoints CRUD
- [x] Frontend con 3 tabs (eventos, feriados, pruebas)

#### Días Feriados
- [x] Tabla `holidays` + endpoints CRUD
- [x] Seeder automático con feriados chilenos (2025-2027)

#### Calendario de Pruebas
- [x] Tabla `exam_schedule` + endpoints CRUD
- [x] Frontend integrado en página de calendario

#### Períodos Académicos
- [ ] Agregar `periods` (semestral/trimestral) con fechas de inicio/fin

### 2.8 Asignación Docente

#### Horas Pactadas
- [ ] Tabla `teacher_contract_hours`: profesor_id, total_horas, horas_clase, horas_admin
- [ ] Endpoint `PUT /api/hr/teachers/{id}/hours` — asignar horas
- [ ] Frontend: Hours Assignment Form

#### Tareas Extras y Pagos
- [ ] Tabla `extra_duties`: profesor, tipo (jefatura, liderazgo, etc.), monto_extra
- [ ] Endpoint `POST /api/hr/teachers/{id}/extra-duties` — asignar tarea extra
- [ ] Frontend: Extra Duties Manager

### 2.9 Asistencia y Evaluación

#### Asistencia Diaria
- [ ] Servicio `packages/services/attendance` ya existe
- [ ] Verificar integración con cursos/asignaturas

#### Notas Parciales y Controles
- [ ] Servicio academic/grades ya existe
- [x] Agregar tipo `control_sorpresa` en `grades`
- [x] Endpoint `POST /api/grades/quick-test` — registrar control sorpresa
- [ ] Promedio simple o ponderado configurable por curso

#### Anotaciones (Positivas y Negativas)
- [x] Tabla `student_annotations`: alumno, tipo, nivel, descripción, fecha
- [x] Niveles según normativa MINEDUC (severidad: leve, grave)
- [x] Endpoint `POST /api/students/annotations` + `GET /api/students/{id}/annotations`
- [x] Frontend: Annotations Timeline (en portales de apoderado y alumno)

### 2.10 Reuniones de Apoderados

#### Reuniones Individuales (Profesor Jefe ↔ Apoderado)
- [x] Tabla `parent_meetings`
- [x] Endpoints CRUD: list, create, update, cancel
- [x] Frontend: create form, list with cancel

#### Reuniones Generales
- [x] Tabla `general_meetings`
- [x] Endpoints CRUD: list, create, update
- [x] **Minuta**: tabla `meeting_minutes`, save/read
- [x] Frontend: 3 tabs (individuales, generales, minutas)
- [ ] Envío automático de minutas por email

---

## 📊 Etapa 2.11 — Dashboard Corporativo (Sostenedor)

> **Rol:** Sostenedor / AdminGlobal — visibilidad global de toda la corporación, filtrable por colegio.
> **Propósito:** Monitorear licencias, ingresos, desempeño académico, asistencia, finanzas y detectar falencias/fortalezas mediante comparativas entre colegios.

### Backend — Endpoints Corporativos

#### KPIs Globales
- [x] `GET /api/corporation/dashboard/summary`
  - Totales: colegios, alumnos, empleados, profesores
  - Licencias: vigentes, próximas a vencer (&lt;30 días), vencidas
  - Ingresos: total facturado (mes/año), morosidad
  - Asistencia general: promedio de todos los colegios
  - Rendimiento académico: promedio general por colegio

#### KPIs por Colegio (filtrable)
- [x] `GET /api/corporation/dashboard/schools` — lista de colegios con KPIs individuales
  - Por cada colegio: alumnos, asistencia %, promedio notas, empleados, ingresos, morosidad

#### Licencias y Planes
- [x] `GET /api/corporation/license/summary` — resumen de licencia corporativa
- [ ] `GET /api/corporation/license/history` — histórico de pagos y renovaciones (baja prioridad)

#### Comparativas entre Colegios
- [x] `GET /api/corporation/dashboard/comparisons`
  - Asistencia por colegio (ranking)
  - Promedio notas por colegio
  - Cantidad de alumnos por colegio
  - Morosidad por colegio

#### Tendencias y Evolución
- [x] `GET /api/corporation/dashboard/trends`
  - Crecimiento matrícula (últimos 12 meses) por colegio
  - Evolución asistencia mensual

#### Alertas Corporativas
- [x] `GET /api/corporation/dashboard/alerts`
  - Colegios con asistencia bajo umbral (configurable, ej: &lt;85%)
  - Licencias próximas a vencer

### Frontend — Dashboard Corporativo

#### Layout
- [x] Ruta `/sostenedor` en Dioxus (protegida para Sostenedor/GerenteGeneral/AdminGlobal)
- [x] Filtro global por colegio (selector en la cabecera del dashboard)
- [x] Selector de año

#### KPIs (Calugas) — 10 indicadores
- [x] Total alumnos corporación
- [x] Total colegios
- [x] Asistencia promedio general (%)
- [x] Promedio notas general
- [x] Ingresos del mes
- [x] Morosidad %
- [x] Licencias activas / próximas a vencer
- [x] Dotación total (profesores y empleados)

#### Gráficos
- [x] **Asistencia por colegio** — barras comparativas
- [x] **Promedio notas por colegio** — barras comparativas
- [x] **Evolución matrícula** (barras, últimos 12 meses)
- [x] **Evolución asistencia mensual** (barras, por colegio o general)
- [x] **Distribución alumnos por colegio** (donut)
- [x] **Morosidad por colegio** (barras)
- [x] **Ranking de colegios** por rendimiento académico
- [ ] **Ingresos vs morosidad** (pendiente de definir data)

#### Tablas
- [x] **Colegios** con KPIs completos (alumnos, asistencia, notas, morosidad)
  - Ordenable por cualquier columna (click en header)
  - Indicadores visuales (verde/rojo según umbrales)
  - Acción "Ver Dashboard" link al dashboard del colegio
- [x] **Alertas** con prioridad (crítica, alta, media, baja)
- [x] **Licencias** con plan, días restantes, módulos

#### Exportación
- [x] Exportar PDF (vía browser print)
- [x] Exportar CSV

---

## 📈 Etapa 2.12 — Dashboard por Colegio

> **Rol:** Director, Administrador, UTP — visibilidad completa del colegio.
> **Propósito:** Monitorear el desempeño interno del colegio con métricas académicas, asistencia, finanzas y alertas.

### Backend — Endpoints del Colegio (8 endpoints, todos implementados)

- [x] `GET /api/school/dashboard/summary` — KPIs del colegio
  - Alumnos totales, profesores, asistencia hoy, alertas, eventos
- [x] `GET /api/school/dashboard/attendance-trends` — tendencia asistencia últimos 12 meses
- [x] `GET /api/school/dashboard/grades-distribution` — distribución de notas por curso
- [x] `GET /api/school/dashboard/top-alerts` — alertas críticas del colegio
- [x] `GET /api/school/dashboard/finance-summary` — resumen financiero
- [x] `GET /api/school/dashboard/teacher-performance` — desempeño docente
- [x] `GET /api/dashboard/attendance-today` — asistencia del día
- [x] `GET /api/dashboard/agenda` — próximos eventos

### Frontend — Dashboard del Colegio

#### Layout
- [x] Ruta `/dashboard` con selector de período (hoy/semana/mes/semestre/año)
- [x] PDF export

#### KPIs
- [x] Alumnos totales (requires fix: `total_enrolled` missing from backend struct)
- [x] Asistencia hoy %
- [x] Promedio notas general
- [x] Profesores activos

#### Gráficos
- [x] Asistencia del día (barra de porcentaje con detalles)
- [x] Evolución asistencia mensual (barras)
- [x] Distribución de notas (donut por rango)
- [x] Finanzas (ingresos, cobrado, pendiente)
- [x] Top alertas de asistencia
- [x] Rendimiento docente (tabla)

#### Acciones Rápidas
- [ ] Pase de lista (acceso directo a asistencia del día)
- [ ] Registrar notas
- [ ] Enviar comunicación masiva

---

## Etapa 3 — Portal Apoderados

### 3.1 Comunicación con Profesor Jefe
- [x] Portal existente (`/parent-portal`) con hijos, notas, asistencia, horario, anotaciones
- [x] Endpoint `POST /api/portal/parent/messages` — enviar mensaje al profesor
- [x] Endpoint `GET /api/portal/parent/available-slots` — horarios disponibles
- [x] Frontend: Chat widget + Schedule Appointment

### 3.2 Revisión de Información del Pupilo
- [x] Endpoint `GET /api/portal/parent/children` — lista de hijos
- [x] Endpoint `GET /api/portal/parent/children/{id}/grades` — notas
- [x] Endpoint `GET /api/portal/parent/children/{id}/attendance` — asistencia
- [x] Endpoint `GET /api/portal/parent/children/{id}/annotations` — anotaciones
- [x] Endpoint `GET /api/portal/parent/children/{id}/schedule` — horario
- [x] Frontend: Student Dashboard (vista del apoderado)

### 3.3 Citas con Personal de Apoyo
- [x] Tabla `support_appointments`: tipo, motivo, fecha, estado
- [x] Endpoint `POST /api/portal/parent/appointments` — agendar cita
- [x] Endpoint `GET /api/portal/parent/appointments` — listar citas
- [x] Frontend: Appointment Booking

### 3.4 Emisión de Certificados
- [x] Tipos de certificado hardcoded (alumno regular, notas, asistencia, conducta)
- [x] Endpoint `POST /api/portal/parent/certificates/request` — solicitar certificado
- [x] Endpoint `GET /api/portal/parent/certificates/{id}/download` — descargar PDF
- [x] Frontend: Certificate Store + botón descargar
- [ ] Generar PDF real (actualmente texto plano)

### 3.5 Inscripción en Cursos Complementarios
- [ ] Endpoint `GET /api/portal/parent/complementary-subjects` — listar disponibles
- [ ] Endpoint `POST /api/portal/parent/enroll-complementary` — inscribir
- [ ] Frontend: Course Catalog (vista del apoderado)

---

## Etapa 4 — Portal Alumnos

### 4.1 Información Académica
- [x] Endpoint `GET /api/portal/student/grades` — notas y promedios
- [x] Endpoint `GET /api/portal/student/attendance` — asistencia
- [x] Endpoint `GET /api/portal/student/annotations` — anotaciones
- [x] Endpoint `GET /api/portal/student/schedule` — horario de clases
- [x] Frontend: Student Dashboard completo con perfil, notas, asistencia, horario, anotaciones

### 4.2 Citas con Personal de Apoyo
- [x] Endpoint `GET /api/portal/student/appointments` — listar citas
- [x] Endpoint `POST /api/portal/student/appointments` — agendar cita
- [x] Frontend: Appointment Booking con formulario y validación

---

## Integraciones Transversales

### Remover Superadmin (Root)
- [x] Rol `Root` eliminado del enum `UserRole` (reemplazado por `GerenteGeneral`)
- [x] `seed_root_admin` ya no existe
- [x] `root_modules()` ya no existe
- [x] Frontend `RootDashboard` eliminado
- [x] Referencias a `"Root"` en código reemplazadas por `"GerenteGeneral"` (notifications, identity client, tests)
- [ ] ~Eliminar archivo `admin.rs`~ — se mantiene como API de gestión para GerenteGeneral (corporaciones, planes, licencias, pagos, branding, health). No se migra al CRM porque son funciones administrativas del sistema, no de ventas.
- [ ] ~Migrar gestión de planes/licencias/pagos al CRM~ — se mantiene en identity/admin.rs con role GerenteGeneral, ya que el CRM gestiona prospectos/contratos/activación (B2B), no la administración del sistema.

### Notificaciones
- [ ] Email automático: confirmación de matrícula, cambio de estado, facturación
- [ ] Notificación in-app (WebSocket) para eventos del CRM y académicos
- [ ] Recordatorio de reuniones de apoderados
- [ ] Alertas corporativas (asistencia baja, licencias próximas a vencer)

### Auditoría y Logs
- [ ] Registrar todas las acciones del CRM de ventas
- [ ] Registrar cambios de estado en admisión
- [ ] Dashboard corporativo: log de actividad de todos los colegios

### Gateway
- [ ] Rutas públicas: `/api/public/plans`, `/api/public/contact`
- [ ] Rutas CRM: `/api/sales/*` — middleware `require_role(["GerenteGeneral", "JefeVentas", "AgenteVentas"])`
- [ ] Rutas corporativas: `/api/corporation/*` — middleware `require_role(["Sostenedor", "AdminGlobal"])`
- [ ] Rutas colegio: `/api/school/*` — middleware `require_role(["Director", "Administrador", "UTP"])`
- [ ] Rutas admisión: `/api/admission/*` — middleware `require_role(["Admision", "Administrador"])`
- [ ] Rutas portal: `/api/portal/*` — middleware `require_role(["Apoderado", "Alumno"])`

---

## Priorización Sugerida

| Prioridad | Componente | Dependencias |
|---|---|---|
| 🔴 Crítica | CRM Ventas (prospectos → contratos → activación) | Licensing existente |
| 🔴 Crítica | Dashboard Corporativo (KPIs, comparativas, alertas) | ✅ Completo |
| 🔴 Crítica | Dashboard por Colegio (gráficos, tendencias) | ✅ Completo |
| 🟡 Alta | Remover Root / migrar a CRM + Corp Dashboard | ✅ Completo |
| 🟡 Alta | Portal Apoderado (notas, asistencia, comunicación) | ✅ Completo |
| 🟡 Alta | Portal Alumno (notas, horario, citas) | ✅ Completo |
| 🟡 Alta | Admisión (matrícula completa con becas) | CRM Ventas |
| 🟡 Alta | Calendario Académico + Pruebas (UTP) | ✅ Completo (backend + frontend) |
| 🟡 Alta | Reuniones de Apoderados + Minutas | ✅ Completo (backend + frontend) |
| 🟢 Media | Horarios Docentes + Tareas Extras | ✅ Completo |
| 🟢 Media | Certificados online | Reporting existente |
| 🔵 Baja | Cursos Complementarios | Academic existente |

---

## Estimación de Esfuerzo

| Etapa | Archivos a modificar/crear | Días estimados |
|---|---|---|
| 1 — Ventas (CRM) | ~15 archivos (backend + frontend) | 10-12 |
| 2.1–2.10 — Gestión Escolar | ~20 archivos | 10-12 |
| 2.11 — Dashboard Corporativo | ~10 archivos (endpoints + frontend) | 6-8 |
| 2.12 — Dashboard por Colegio | ~8 archivos (mejora frontend + endpoints) | 4-5 |
| 3 — Portal Apoderados | ~10 archivos | 5-7 |
| 4 — Portal Alumnos | ~6 archivos | 3-4 |
| Remover Root + migración | ~8 archivos | 2-3 |
| **Total original** | **~75 archivos** | **40-50 días** |
| **Total ejecutado** | **~172 archivos** | **✅ Completo** |

---

## 🧹 Refactorings y Mejoras (Agregadas durante implementación)

| Mejora | Archivos | Impacto |
|--------|----------|---------|
| Auth compartido (`common::auth`) | 51 archivos | Eliminó ~280 líneas duplicadas de Claims/FromRequestParts |
| SQL format!() → NULL-able params | 5 archivos | Eliminó risk de inyección SQL en queries dinámicos |
| Closures en loops → for loops | 6 frontend | Rendimiento de renderizado en Dioxus |
| Monolitos → submódulos | 11 archivos nuevos | reports/, finance/, admission/ separados |
| SVGs inline → Icon component | 12 archivos | 34 SVGs → 1 componente con 26 variantes |
| Loading/Error states | 4 componentes | Separados en 3 estados (data/error/loading) |
| Clippy warnings | ~31 auto-fix | Código más idiomático |
| Dead code eliminado | 1 archivo | routes.rs_new_onboarding.rs (~82 líneas) |
| `.unwrap()` en producción | 2 → 0 | Reemplazados por if let Some |
| setup.sh mejorado | 1 archivo | Instalador tipo Open edX con --status/--logs/--reset |

## ⬜ Pendiente (Mejora Futura)

- **OAuth2 para proveedores email**: Inicio de sesión con Google/Microsoft para SMTP (refresh tokens + XOAUTH2)
