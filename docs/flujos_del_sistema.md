# SchoolCCB v2 — Documentacion de Flujos del Sistema

> Plataforma de gestion escolar SaaS construida en Rust (Axum + SQLx + PostgreSQL)
> con arquitectura de microservicios. Implementa curriculo MINEDUC chileno,
> reportes SIGE, Ley Karin, y legislacion laboral chilena.

---

## Indice

1. [Arquitectura General](#1-arquitectura-general)
2. [Flujo de Identidad y Autenticacion](#2-flujo-de-identidad-y-autenticacion)
3. [Flujo de Ventas B2B (CRM)](#3-flujo-de-ventas-b2b-crm)
4. [Flujo de Admision de Alumnos](#4-flujo-de-admision-de-alumnos)
5. [Flujo Academico (Notas)](#5-flujo-academico-notas)
6. [Flujo de Asistencia Estudiantil](#6-flujo-de-asistencia-estudiantil)
7. [Flujo de Recursos Humanos](#7-flujo-de-recursos-humanos)
8. [Flujo de Remuneraciones (Payroll)](#8-flujo-de-remuneraciones-payroll)
9. [Flujo de Finanzas](#9-flujo-de-finanzas)
10. [Flujo de Reportes y SIGE](#10-flujo-de-reportes-y-sige)
11. [Matriz de Roles y Permisos](#11-matriz-de-roles-y-permisos)
12. [Diagramas UML](#12-diagramas-uml)
13. [Resumen de Arquitectura](#13-resumen-de-la-arquitectura)

---

## 1. Arquitectura General

### 1.1 Microservicios

```
                    ┌──────────────────────────────────────────────────────────┐
                    │               NGINX (puerto 8080)                        │
                    │  Frontend SPA (Dioxus WASM) + Proxy reverso /api/*      │
                    └──────────────────────┬───────────────────────────────────┘
                                           │
                                     ┌─────▼──────┐
                                     │   Gateway   │  (Axum, puerto 3000)
                                     │  Proxy +    │
                                     │  GraphQL    │
                                     └──┬──┬──┬──┬─┘
                                        │  │  │  │
               ┌────────────────────────┘  │  └──────────────┐
               │            ┌──────────────┘                 │
               ▼            ▼                               ▼
       ┌───────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐
       │  Identity  │  │   SIS    │  │ Academic │  │     CRM      │
       │  (3001)    │  │  (3002)  │  │  (3003)  │  │    (3012)    │
       │ Auth,Users │  │Students  │  │ Grades   │  │ Sales B2B    │
       │ Roles,Corps│  │ Courses  │  │ Subjects │  │ Proposals    │
       │ Schools    │  │ HR       │  │ Periods  │  │ Contracts    │
       │ Licenses   │  │ Admission│  │ Reports  │  │ Activation   │
       └───────────┘  │ Finance  │  └──────────┘  └──────────────┘
                      │ Attend.  │
                      └──────────┘
                           │
                           ▼
                    ┌─────────────────────────────────┐
                    │      PostgreSQL (compartida)     │
                    │      ~70 tablas                 │
                    └─────────────────────────────────┘
```

## 2. Flujo de Identidad y Autenticacion

### 2.1 Login

```
[Usuario]           [Gateway]           [Identity Service]       [PostgreSQL]
    │                    │                      │                    │
    │ POST /api/auth/    │                      │                    │
    │ login {email,pass} │                      │                    │
    │───────────────────>│  proxy_identity()     │                    │
    │                    │─────────────────────>│                    │
    │                    │                      │ find_by_email()    │
    │                    │                      │───────────────────>│
    │                    │                      │<───────────────────│
    │                    │                      │                    │
    │                    │                      │ verify_password()  │
    │                    │                      │ (Argon2id)         │
    │                    │                      │                    │
    │                    │                      │ generate JWT       │
    │                    │                      │ (12h expiry)       │
    │                    │                      │                    │
    │                    │                      │ create_refresh_    │
    │                    │                      │ token (7d)         │
    │                    │                      │───────────────────>│
    │                    │                      │<───────────────────│
    │                    │                      │                    │
    │  Set-Cookie:       │<─────────────────────│                    │
    │  jwt_token=...     │  { token,            │                    │
    │  (Path=/, Max-Age  │    refresh_token,    │                    │
    │   =43200, SameSite │    user }            │                    │
    │  =Lax)             │                      │                    │
    │<───────────────────│                      │                    │
    │                    │                      │                    │
    │ Almacena JWT en    │                      │                    │
    │ memoria WASM       │                      │                    │
    │ (static TOKEN)     │                      │                    │
```

### 2.2 Request Autenticado

```
[Frontend WASM]          [Gateway]              [Service]
    │                        │                      │
    │ GET /api/students      │                      │
    │ Authorization: Bearer  │                      │
    │ <JWT de memoria>       │                      │
    │───────────────────────>│                      │
    │                        │ Verifica JWT en      │
    │                        │ Authorization header │
    │                        │                      │
    │                        │ Si no hay Auth,      │
    │                        │ busca cookie          │
    │                        │ jwt_token=...         │
    │                        │                      │
    │                        │ proxy_request()      │
    │                        │──────────────────────>│
    │                        │                      │
    │                        │<──────────────────────│
    │<───────────────────────│                      │
```

### 2.3 JWT Claims

```json
{
  "sub": "uuid-usuario",
  "role": "Director",
  "name": "Daniela Soto Pizarro",
  "email": "daniela.soto@colegio.cl",
  "school_id": "uuid-colegio",
  "corporation_id": "uuid-corporacion",
  "admin_type": "school",
  "exp": 1716000000,
  "iat": 1715956800
}
```

### 2.4 Roles

| Rol | Descripcion |
|-----|-------------|
| **GerenteGeneral** | Superadmin corporativo, acceso total al sistema |
| **Sostenedor** | Dueno de la corporacion, acceso corporativo |
| **Administrador** | Administrador del colegio, gestion completa |
| **Director** | Director academico, supervision general |
| **UTP** | Unidad Tecnico Pedagogica, gestion curricular |
| **Profesor** | Docente, gestion de cursos y notas |
| **Apoderado** | Padre/madre/apoderado, consulta de pupilos |
| **Alumno** | Estudiante, consulta de notas y asistencia |
| **Admision** | Equipo de admision, gestion de postulantes |
| **JefeVentas** | Jefe del equipo de ventas CRM |
| **AgenteVentas** | Agente de ventas comercial |

---

## 3. Flujo de Ventas B2B (CRM)

### 3.1 Pipeline de 8 Etapas

```
NUEVO ──> CONTACTADO ──> REQUISITOS ──> PROPUESTA ──> NEGOCIACION ──> CONTRATO ──> CERRADO GANADO
  (1)        (2)            (3)            (4)             (5)            (6)            (7)
                                                                                         │
                                                                                         └──> CERRADO PERDIDO (8)
```

### 3.2 Flujo Completo

**Paso 1 - Captura del Lead (Nuevo):**
- **Web publico** (sin auth): `POST /api/public/sales/prospects` → source="web", sin agente asignado
- **Agente** (auth): `POST /api/sales/prospects` → assigned_to = creador
- **CSV masivo**: `POST /api/sales/prospects/import` → validacion por fila

**Paso 2 - Asignacion y Contacto (→ Contactado):**
- `PUT /api/sales/prospects/{id}/assign` solo manager
- `PUT /api/sales/prospects/{id}/move` para avanzar etapa

**Paso 3 - Seguimiento (→ Requisitos):**
- `POST /api/sales/prospects/{id}/activities` registrar llamadas, emails, reuniones
- `PUT /api/sales/prospects/{id}` actualizar requirements (JSONB)

**Paso 4 - Propuesta (→ Propuesta):**
- `POST /api/sales/proposals` crear propuesta con plan, modulos, valor
- `PUT /api/sales/proposals/{id}/discount` (manager) auto-incrementa version

**Paso 5 - Negociacion (→ Negociacion):**
- Iterar propuestas con descuentos

**Paso 6 - Contrato (→ Contrato):**
- `POST /api/sales/contracts` (manager) calcula subtotal + IVA 19%
- Status: "draft"
- `POST /api/sales/contracts/{id}/documents` subir PDF firmado

**Paso 7 - Verificacion:**
- `PUT /api/sales/contracts/{id}/verify-signatures` (manager)
- Status: "verified"

**Paso 8 - Activacion (→ Cerrado Ganado):**
- `POST /api/sales/contracts/{id}/activate` (manager)
- Llama a Identity para onboarding (crea corporation, school, admin)
- Crea corporation_licenses (365 dias, grace 30)
- Contrato → "active", prospecto → "Cerrado Ganado"
- Envia credenciales al cliente

**Paso 9 - Facturacion:**
- `POST /api/sales/contracts/{id}/invoice` genera INV-YYYYMM-XXXXXX

---

## 4. Flujo de Admision de Alumnos

### 4.1 Pipeline de 6 Etapas

```
PRIMER CONTACTO ──> TOUR ESCOLAR ──> EVALUACION ──> DOCUMENTACION ──> ACEPTADO ──> MATRICULADO
     (0)               (1)              (2)              (3)             (4)           (5)
```

### 4.2 Flujo Completo

1. **Crear Prospecto**: `POST /api/admission/prospects` → etapa "Primer Contacto"
2. **Gestionar**: actividades (llamadas, notas, emails), mover entre etapas
3. **Documentos**: subir via multipart, verificar por admin/director
4. **Aceptado**: workflow notifica a finanzas
5. **Matriculado**: validacion RUT obligatorio → `promote_to_student()`:
   - Crea registro en `students`
   - Crea `enrollment` en el curso correspondiente
   - Crea contrato de matricula

### 4.3 Endpoints Clave

| Method | Path | Descripcion |
|--------|------|-------------|
| GET | `/api/admission/prospects` | Listar (filtros: etapa, busqueda, asignado) |
| POST | `/api/admission/prospects` | Crear prospecto |
| PUT | `/api/admission/prospects/{id}/stage` | Mover de etapa |
| POST | `/api/admission/documents/upload` | Subir documento (multipart) |
| POST | `/api/admission/documents/{id}/verify` | Verificar documento |
| GET | `/api/admission/vacancy-check` | Verificar vacantes por nivel |
| GET | `/api/admission/metrics` | Dashboard metricas |

---

## 5. Flujo Academico (Notas)

### 5.1 Ciclo Anual

```
1. Activar ano academico
2. Crear periodos (Semestre 1, Semestre 2)
3. Asignar materias a cursos con profesores (course_subjects)
4. Crear categorias de evaluacion con pesos (Pruebas 50%, Tareas 30%, etc.)
5. Profesores ingresan notas (individual o masivo)
6. Fin de semestre: calcular promedios ponderados
7. Evaluar promocion segun Decreto 67
```

### 5.2 Ingreso de Notas

```
Individual:  POST /api/grades { student_id, course_subject_id, grade: 5.5, ... }
Masivo:      POST /api/grades/bulk { course_subject_id, grades: [...] }
```

**Validaciones**: rango 1.0-7.0, redondeo a 1 decimal, tipo Sumativa/Formativa.

### 5.3 Calculo de Promedio

```
Por asignatura + alumno + semestre:
  Promedio_categoria = suma(notas) / cantidad
  Aporte_ponderado = promedio_categoria * (weight/100)
  Promedio_final = suma de aportes_ponderados
```

### 5.4 Promocion (Decreto 67)

| Reprobadas (<4.0) | Condicion | Resultado |
|---|---|---|
| 0 | — | Promovido |
| 1 | Prom >= 3.5 | Promovido |
| 1 | Prom < 3.5 | Reprobado |
| 2 | Ambas >= 3.0 | Promovido |
| 2 | Alguna < 3.0 | Reprobado |
| 3+ | — | Reprobado |

---

## 6. Flujo de Recursos Humanos

### 6.1 Ciclo del Empleado

```
Contratacion:
  1. POST /api/hr/employees (crear ficha)
  2. POST /api/hr/employees/{id}/contracts (contrato con sueldo y horas)
  3. POST /api/hr/employees/{id}/pension-fund (AFP y salud)

Durante:
  - Marcacion diaria: POST /api/hr/attendance/sync
  - Solicitudes de permiso: POST /api/hr/me/leave-requests
  - Liquidacion mensual: POST /api/hr/payroll

Desvinculacion:
  - DELETE /api/hr/employees/{id} (desactivar)
```

### 6.2 Calculo de Liquidacion

```
Sueldo Base + Gratificacion (25%, tope $500K) = Renta Imponible

Descuentos:
  AFP: 10% + comision (10.58% a 11.45% segun AFP)
  Salud: 7% Fonasa / monto fijo Isapre
  Cesantia: 0.6%
  Impuesto: tramos progresivos 4%-35% (anualizado/12)

Sueldo Liquido = Renta Imponible + No Imponible - Descuentos
```

### 6.3 Denuncias Ley Karin

```
POST /api/hr/complaints/submit  →  Estado: recibida
                               →  investigando
                               →  resuelta
                               →  cerrada
```

---

## 7. Flujo de Finanzas

### 7.1 Aranceles y Pagos

```
Crear arancel:  POST /api/finance/fees
Pago manual:    POST /api/finance/payments (auto-marca fee como pagado)
Pago Webpay:    GET  /api/finance/payment/init/{fee_id}
                → Callback: /api/finance/payment/return?token_ws=...
```

### 7.2 Becas

```
Crear:     POST /api/finance/scholarships
Aprobar:   PUT /api/finance/scholarships/{id}
```

---

## 8. Matriz de Permisos (Resumen)

| Modulo | Gerente | Sostenedor | Admin | Director | UTP | Profesor | Apoderado | Alumno | Admision | JefeVtas | AgenteVtas |
|--------|:-------:|:----------:|:-----:|:--------:|:---:|:--------:|:---------:|:------:|:--------:|:--------:|:----------:|
| Students | CRUD | CRUD | CRUD | CRUD | CRUD | R | — | — | R | — | — |
| Attendance | CRUD | CRUD | CRUD | CRUD | CRUD | CR | R* | R* | — | — | — |
| Grades | CRUD | CRUD | CRUD | CRUD | CRUD | CR | R* | R* | — | — | — |
| HR | CRUD | R | CRUD | CRUD | R | — | — | — | — | — | — |
| Payroll | CRUD | R | CRUD | CRUD | R | — | — | — | — | — | — |
| Admission | CRUD | CRUD | CRUD | CRUD | CRUD | — | — | — | CRUD | — | — |
| Sales | CRUD | — | — | — | — | — | — | — | — | CRUD | CR** |
| MyPortal | — | — | — | — | — | — | R | R | — | — | — |

*R* = solo lectura de sus datos  
** = solo sus prospectos asignados

---

## 9. Resumen de Arquitectura

### Stack

| Componente | Tecnologia |
|------------|------------|
| Backend | Rust, Axum 0.8, SQLx, Tokio |
| Frontend | Rust, Dioxus 0.6 (WASM) |
| Base de datos | PostgreSQL 16 |
| Auth | JWT + Argon2id + Refresh Tokens |
| Comunicacion | Event Bus local + gRPC + WebSocket |
| Proxy | NGINX |
| Contenedores | Docker Compose |

### Base de Datos (~70 tablas)

- **Core**: users, roles, permissions
- **Multi-tenant**: corporations, schools, licenses
- **SIS**: students, courses, enrollments, guardian_relationships
- **Academic**: subjects, grades, periods, categories
- **HR**: employees, contracts, payroll, leave_requests, complaints
- **Finance**: fees, payments, scholarships
- **Admission**: prospects, stages, documents, contracts
- **CRM Sales**: crm_sales_prospects, contracts, proposals, agents, goals
- **Audit**: audit_log, event_log, admin_activity_log

---

> Documentacion generada el 2026-05-20 — SchoolCCB v2
