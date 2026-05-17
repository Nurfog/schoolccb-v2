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
| CRM Admisión (Prospects, Pipeline) | ✅ Parcial (`packages/common/src/admission.rs`, frontend `AdmissionPage`) |
| Módulo Académico (Cursos, Asignaturas, Notas) | ✅ Completado |
| Asistencia | ✅ Completado |
| RRHH (Empleados, Contratos, Remuneraciones) | ✅ Completado |
| Finanzas | ✅ Completado |
| SIGE / MINEDUC | ✅ Completado |
| Portal Público | ⬜ No iniciado |
| Portal Apoderado (my-portal) | ✅ Parcial |
| Portal Alumno | ⬜ No iniciado |
| Portal Corporación (Sostenedor) | ⬜ Básico — falta dashboard completo con KPIs, comparativas, gráficos |
| Dashboard Corporativo | ⬜ No existe — actualmente usa solo `/api/dashboard/summary` (escolar, no corporativo) |
| Dashboard por Colegio | ✅ Parcial (mosaicos simples) — faltan gráficos, comparativas, tendencias |
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
- [ ] **Mejora:** Soporte para **RUT** en modelos de prospectos y corporaciones (validación chilena)
- [ ] **Faltante:** Endpoints para gestión de equipo de ventas (crear agentes, asignar metas)

### 1.2 Interfaz de Usuario (Salesforce Style)
- [x] Layout específico `/sales` en Dioxus
- [x] **Pipeline Kanban** interactivo por etapas
- [x] **Prospect Detail** con vista 360° del cliente
- [x] **Contact Timeline** (Llamadas, Emails, WhatsApp, Notas)
- [ ] **Quote Builder:** Implementar lógica de selección de módulos y cálculo de precios
- [ ] **Contract Builder:** Generación de contrato basado en propuesta aprobada
- [ ] **Document Viewer:** Visualización de documentos subidos y estado de verificación
- [ ] **License Activator:** Wizard de activación final (Corporación + Licencia + Usuario Admin)

### 1.3 Proceso de Venta (Backend & API)

#### 1. Captación de Prospectos
- [ ] **Bug/Faltante:** Integrar `POST /api/public/contact` del portal para crear prospectos en el CRM automáticamente
- [x] `POST /api/sales/prospects` — creación manual
- [ ] **Faltante:** `POST /api/sales/prospects/import` — importación masiva vía CSV
- [x] Tabla `crm_sales_prospects` con campos básicos

#### 2. Asignación y Seguimiento
- [x] `PUT /api/sales/prospects/{id}/assign` — asignación manual a agente
- [ ] **Mejora:** Regla de asignación automática (Round-robin)
- [x] `POST /api/sales/prospects/{id}/activities` — registro de interacciones

#### 3. Propuestas y Negociación
- [x] `POST /api/sales/proposals` — creación de borradores
- [x] `PUT /api/sales/proposals/{id}/discount` — aplicación de descuentos (requiere JefeVentas+)
- [ ] **Faltante:** Generación de PDF de la propuesta comercial

#### 4. Contratos y Documentación
- [x] `POST /api/sales/contracts` — creación de contrato
- [x] `POST /api/sales/contracts/{id}/documents` — subida de documentos (S3/Local)
- [x] `PUT /api/sales/contracts/{id}/verify-signatures` — validación interna
- [ ] **Mejora:** Integración con servicio de firma electrónica (ej. Toku)

#### 5. Facturación B2B
- [ ] **Faltante:** `POST /api/sales/contracts/{id}/invoice` — integración con módulo `finance` para generar la primera factura/NC
- [ ] **Mejora:** Soporte para impuestos (IVA) y exenciones según normativa chilena

#### 6. Activación del Servicio
- [x] `POST /api/sales/contracts/{id}/activate` — creación de Corporación y Licencia
- [ ] **Faltante:** Creación automática del **primer usuario (Sostenedor)** y envío de email de bienvenida
- [ ] **Faltante:** Creación de un **Colegio por defecto** para la corporación recién creada

### 1.4 Dashboard Comercial
- [x] `GET /api/sales/dashboard/summary` — KPIs básicos
- [ ] **Mejora:** Gráficos de embudo de ventas (funnel) y rendimiento por agente
- [ ] **Mejora:** Reporte de ingresos proyectados basado en pipeline actual

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
- [ ] Endpoint `POST /api/admission/prospects/{id}/activities` — registrar contacto
- [ ] Endpoint `POST /api/admission/prospects/{id}/reminder` — programar recordatorio
- [ ] Frontend: agenda de seguimientos con notificaciones

#### Preparación de Contrato (Matrícula)
- [ ] Tabla `enrollment_contracts`: alumno, apoderados, anexos (PIE, TEA, etc.)
- [ ] Endpoint `POST /api/admission/contracts` — crear contrato de matrícula
- [ ] Endpoint `POST /api/admission/contracts/{id}/documents` — subir anexos
- [ ] Frontend: Enrollment Contract Builder

#### Gestión de Pagos de Matrícula
- [ ] Integrar con módulo finance para pagos (débito, crédito, efectivo, transferencia)
- [ ] Endpoint `POST /api/admission/payments` — registrar pago
- [ ] Soporte para **becas**: tabla `scholarships`, endpoint `POST /api/admission/scholarships/apply`
- [ ] Frontend: Payment Gateway + Scholarship Application

#### Inscripción Final
- [ ] Endpoint `POST /api/admission/enroll` — inscribir alumno en curso
- [ ] Generar documentos tributarios (integración finance)
- [ ] Frontend: confirmation screen con resumen

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
- [ ] Tabla `academic_calendar`: eventos configurables con alertas
- [ ] Endpoint `POST /api/academic/calendar` — crear evento
- [ ] Frontend: calendario con vista mensual/semanal

#### Días Feriados
- [ ] Tabla `holidays`: fecha, nombre, tipo (legal / escolar)
- [ ] Endpoint `POST /api/academic/holidays` — cargar feriados
- [ ] Seeder automático con feriados chilenos del año

#### Calendario de Pruebas
- [ ] Tabla `exam_schedule`: asignatura, curso, fecha, período, responsable
- [ ] Endpoint `POST /api/academic/exams` — crear prueba
- [ ] Frontend: Exam Calendar con alertas

#### Períodos Académicos
- [ ] Ya existe `AcademicYears` en frontend
- [ ] Agregar `periods` (semestral/trimestral) con fechas de inicio/fin
- [ ] Endpoint `POST /api/academic/periods` — definir período

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
- [ ] Agregar tipo `control_sorpresa` en `grades`
- [ ] Endpoint `POST /api/grades/quick-test` — registrar control sorpresa
- [ ] Promedio simple o ponderado configurable por curso

#### Anotaciones (Positivas y Negativas)
- [ ] Tabla `student_annotations`: alumno, tipo, nivel, descripción, fecha
- [ ] Niveles según normativa MINEDUC
- [ ] Endpoint `POST /api/students/{id}/annotations`
- [ ] Frontend: Annotations Timeline

### 2.10 Reuniones de Apoderados

#### Reuniones Individuales (Profesor Jefe ↔ Apoderado)
- [ ] Tabla `parent_meetings`: profesor_id, apoderado_id, fecha, hora, estado
- [ ] Endpoint `POST /api/meetings/schedule` — agendar
- [ ] Frontend: Meeting Scheduler con slots disponibles

#### Reuniones Generales
- [ ] Tabla `general_meetings`: curso, fecha, hora, lugar, orden_del_día
- [ ] Endpoint `POST /api/meetings/general` — crear reunión
- [ ] **Minuta**: tabla `meeting_minutes`, envío automático por email
- [ ] Frontend: Meeting Manager + Minutes Editor

---

## 📊 Etapa 2.11 — Dashboard Corporativo (Sostenedor)

> **Rol:** Sostenedor / AdminGlobal — visibilidad global de toda la corporación, filtrable por colegio.
> **Propósito:** Monitorear licencias, ingresos, desempeño académico, asistencia, finanzas y detectar falencias/fortalezas mediante comparativas entre colegios.

### Backend — Endpoints Corporativos

#### KPIs Globales
- [ ] `GET /api/corporation/dashboard/summary`
  - Totales: colegios, alumnos, empleados, profesores
  - Licencias: vigentes, próximas a vencer (&lt;30 días), vencidas
  - Ingresos: total facturado (mes/año), morosidad
  - Asistencia general: promedio de todos los colegios
  - Rendimiento académico: promedio general por colegio

#### KPIs por Colegio (filtrable)
- [ ] `GET /api/corporation/dashboard/schools` — lista de colegios con KPIs individuales
  - Por cada colegio: alumnos, asistencia %, promedio notas, empleados, ingresos, morosidad
  - Curso con mejor/peor rendimiento
  - Curso con mejor/peor asistencia

#### Licencias y Planes
- [ ] `GET /api/corporation/license/summary` — resumen de licencia corporativa
- [ ] `GET /api/corporation/license/history` — histórico de pagos y renovaciones
- [ ] `GET /api/corporation/modules` — módulos activos y overrides

#### Comparativas entre Colegios
- [ ] `GET /api/corporation/dashboard/comparisons`
  - Asistencia por colegio (ranking)
  - Promedio notas por colegio
  - Evolución mensual de asistencia/notas por colegio
  - Cantidad de alumnos por colegio
  - Dotación docente vs alumnos por colegio
  - Ingresos vs morosidad por colegio

#### Tendencias y Evolución
- [ ] `GET /api/corporation/dashboard/trends`
  - Crecimiento matrícula (últimos 12 meses) por colegio
  - Evolución asistencia mensual
  - Evolución promedios generales por período
  - Rotación docente

#### Alertas Corporativas
- [ ] `GET /api/corporation/dashboard/alerts`
  - Colegios con asistencia bajo umbral (configurable, ej: &lt;85%)
  - Colegios con rendimiento bajo umbral (configurable)
  - Licencias próximas a vencer
  - Morosidad crítica por colegio
  - Dotación insuficiente (profesores vs alumnos)

### Frontend — Dashboard Corporativo

#### Layout
- [ ] Ruta `/corporation/dashboard` en Dioxus (protegida para Sostenedor/AdminGlobal)
- [ ] Filtro global por colegio (selector en la cabecera del dashboard)
- [ ] Selector de período (mes actual, trimestre, semestre, año)

#### KPIs (Calugas)
- [ ] Total alumnos corporación
- [ ] Total colegios
- [ ] Asistencia promedio general (%)
- [ ] Promedio notas general
- [ ] Ingresos del mes
- [ ] Morosidad %
- [ ] Licencias activas / próximas a vencer
- [ ] Dotación total (profesores)

#### Gráficos
- [ ] **Asistencia por colegio** — barras comparativas horizontales
- [ ] **Promedio notas por colegio** — barras comparativas
- [ ] **Evolución matrícula** (línea, últimos 12 meses)
- [ ] **Evolución asistencia mensual** (línea, por colegio o general)
- [ ] **Distribución alumnos por colegio** (donut/pie)
- [ ] **Ingresos vs morosidad por colegio** (barras agrupadas)
- [ ] **Ranking de colegios** por rendimiento académico
- [ ] **Mapa de calor** (asistencia x curso x mes)

#### Tablas
- [ ] **Colegios** con KPIs completos (alumnos, asistencia, notas, ingresos, morosidad)
  - Ordenable por cualquier columna
  - Indicadores visuales (verde = sobre promedio, rojo = bajo umbral)
  - Acción para ir al dashboard del colegio
- [ ] **Alertas** con prioridad (crítica, alta, media, baja)
- [ ] **Licencias** con fechas y estados

#### Exportación
- [ ] Botón "Exportar PDF" — reporte ejecutivo corporativo
- [ ] Botón "Exportar CSV" — datos crudos de comparativa

---

## 📈 Etapa 2.12 — Dashboard por Colegio

> **Rol:** Director, Administrador, UTP — visibilidad completa del colegio.
> **Propósito:** Monitorear el desempeño interno del colegio con métricas académicas, asistencia, finanzas y alertas.

### Backend — Endpoints del Colegio

- [ ] `GET /api/school/dashboard/summary` — KPIs del colegio
  - Alumnos totales, por curso, por nivel
  - Profesores totales, por asignatura
  - Asistencia del día/semana/mes
  - Promedio notas general y por curso
  - Ingresos del período
  - Morosidad
- [ ] `GET /api/school/dashboard/attendance-trends` — tendencia asistencia últimos 12 meses
- [ ] `GET /api/school/dashboard/grades-distribution` — distribución de notas por curso
- [ ] `GET /api/school/dashboard/top-alerts` — alertas críticas del colegio
- [ ] `GET /api/school/dashboard/finance-summary` — resumen financiero
- [ ] `GET /api/school/dashboard/teacher-performance` — desempeño docente (notas promedio por profesor, asistencia por curso)

### Frontend — Dashboard del Colegio

#### Layout
- [ ] Mejorar `/dashboard` existente con gráficos y tablas (no solo mosaicos)
- [ ] Selector de período (día, semana, mes, semestre, año)

#### KPIs
- [ ] Alumnos totales (vs capacidad)
- [ ] Asistencia hoy/semana/mes
- [ ] Promedio notas general
- [ ] Profesores activos
- [ ] Ingresos del mes
- [ ] Morosidad
- [ ] Alertas activas

#### Gráficos
- [ ] Asistencia del día por curso (barras)
- [ ] Evolución asistencia mensual (línea)
- [ ] Distribución de notas (histograma por rango)
- [ ] Promedio por curso (barras)
- [ ] Ingresos vs gastos (barras agrupadas)
- [ ] Top alertas por curso

#### Tablas
- [ ] Cursos con KPIs (alumnos, asistencia, promedio, alertas)
- [ ] Profesores con carga horaria y cursos asignados
- [ ] Alertas activas con acción

#### Acciones Rápidas
- [ ] Pase de lista (acceso directo a asistencia del día)
- [ ] Registrar notas
- [ ] Ver agenda del día
- [ ] Enviar comunicación masiva

---

## Etapa 3 — Portal Apoderados

### 3.1 Comunicación con Profesor Jefe
- [ ] Expandir portal existente (`/my-portal`) para apoderados
- [ ] Endpoint `POST /api/portal/parent/messages` — enviar mensaje al profesor
- [ ] Endpoint `GET /api/portal/parent/available-slots` — horarios disponibles
- [ ] Frontend: Chat widget + Schedule Appointment

### 3.2 Revisión de Información del Pupilo
- [ ] Endpoint `GET /api/portal/parent/children` — lista de hijos
- [ ] Endpoint `GET /api/portal/parent/children/{id}/grades` — notas
- [ ] Endpoint `GET /api/portal/parent/children/{id}/attendance` — asistencia
- [ ] Endpoint `GET /api/portal/parent/children/{id}/annotations` — anotaciones
- [ ] Endpoint `GET /api/portal/parent/children/{id}/schedule` — horario
- [ ] Frontend: Student Dashboard (vista del apoderado)

### 3.3 Citas con Personal de Apoyo
- [ ] Tabla `support_appointments`: alumno_id, tipo (enfermería, psicólogo, etc.), fecha, estado
- [ ] Endpoint `POST /api/portal/parent/appointments` — agendar cita
- [ ] Endpoint `GET /api/portal/parent/appointments` — listar citas
- [ ] Frontend: Appointment Booking

### 3.4 Emisión de Certificados
- [ ] Tabla `certificate_types`: nombre, costo, gratuito, descripción
- [ ] Endpoint `POST /api/portal/parent/certificates/request` — solicitar certificado
- [ ] Endpoint `GET /api/portal/parent/certificates/{id}/download` — descargar PDF
- [ ] Integración con módulo finance si tiene costo
- [ ] Frontend: Certificate Store

### 3.5 Inscripción en Cursos Complementarios
- [ ] Endpoint `GET /api/portal/parent/complementary-subjects` — listar disponibles
- [ ] Endpoint `POST /api/portal/parent/enroll-complementary` — inscribir
- [ ] Frontend: Course Catalog (vista del apoderado)

---

## Etapa 4 — Portal Alumnos

### 4.1 Información Académica
- [ ] Endpoint `GET /api/portal/student/grades` — notas y promedios
- [ ] Endpoint `GET /api/portal/student/attendance` — asistencia
- [ ] Endpoint `GET /api/portal/student/annotations` — anotaciones
- [ ] Endpoint `GET /api/portal/student/schedule` — horario de clases
- [ ] Frontend: Student Dashboard (vista del alumno)

### 4.2 Citas con Personal de Apoyo
- [ ] Endpoint `GET /api/portal/student/appointments` — listar citas
- [ ] Endpoint `POST /api/portal/student/appointments` — agendar cita
- [ ] Frontend: Appointment Booking (vista alumno)

---

## Integraciones Transversales

### Remover Superadmin (Root)
- [ ] Eliminar rol `Root` del enum `UserRole` en `packages/common/src/user.rs`
- [ ] Eliminar `seed_root_admin` de `packages/services/identity/src/models.rs`
- [ ] Eliminar `root_modules()` de `packages/services/identity/src/routes.rs`
- [ ] Eliminar archivo `admin.rs` completo de identity (o migrar lógica de administración de licencias/planes al CRM de ventas)
- [ ] Las funciones de gestión de planes, licencias y pagos pasan al servicio CRM (`packages/services/crm`)
- [ ] Eliminar frontend `RootDashboard` y rutas `/admin/*` (las reemplaza el dashboard corporativo y el CRM)
- [ ] Endpoints de stats globales pasan a ser corporativos (`/api/corporation/dashboard/*`)

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
| 🔴 Crítica | Dashboard Corporativo (KPIs, comparativas, alertas) | SIS, Academic, Finance existentes |
| 🔴 Crítica | Dashboard por Colegio (gráficos, tendencias) | SIS, Academic, Attendance existentes |
| 🟡 Alta | Remover Root / migrar a CRM + Corp Dashboard | Todo lo anterior |
| 🟡 Alta | Portal Apoderado (notas, asistencia, comunicación) | Auth, Academic existentes |
| 🟡 Alta | Portal Alumno (notas, horario, citas) | Auth, Academic existentes |
| 🟡 Alta | Admisión (matrícula completa con becas) | CRM Ventas |
| 🟡 Alta | Calendario Académico + Pruebas (UTP) | Academic existente |
| 🟡 Alta | Reuniones de Apoderados + Minutas | Notifications existente |
| 🟢 Media | Horarios Docentes + Tareas Extras | HR existente |
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
