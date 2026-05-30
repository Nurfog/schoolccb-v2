use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rut::Rut;

/// Tipo de administrador: global (todos los colegios) o por colegio específico.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdminType {
    #[serde(rename = "global")]
    Global,
    #[serde(rename = "school")]
    School,
}

impl AdminType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdminType::Global => "global",
            AdminType::School => "school",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "global" => Some(AdminType::Global),
            "school" => Some(AdminType::School),
            _ => None,
        }
    }
}

/// Rol funcional de un usuario en el sistema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum UserRole {
    /// Gerente General — superadministrador (reemplaza Root).
    GerenteGeneral,
    /// Jefe de Ventas — supervisa equipo comercial.
    JefeVentas,
    /// Agente de Ventas — ejecutivo comercial.
    AgenteVentas,
    /// Sostenedor o administrador de corporación.
    Sostenedor,
    /// Director del establecimiento.
    Director,
    /// Jefe de Unidad Técnico Pedagógica.
    UTP,
    /// Administrador del establecimiento.
    Administrador,
    /// Profesor o docente.
    Profesor,
    /// Profesor honorario (sin contrato formal).
    ProfesorHonorario,
    /// Apoderado o tutor de un estudiante.
    Apoderado,
    /// Alumno o estudiante.
    Alumno,
    /// Usuario del módulo de admisión.
    Admision,
    /// Representante legal de la corporación.
    RepresentanteLegal,
    /// Director de Recursos Humanos.
    DirectorRRHH,
    /// Administrativo del establecimiento.
    Administrativo,
    /// Administrador global con acceso a toda la corporación.
    AdminGlobal,
}

impl UserRole {
    /// Retorna `true` si el rol tiene permisos administrativos.
    pub fn es_admin(&self) -> bool {
        matches!(
            self,
            UserRole::Administrador
                | UserRole::Sostenedor
                | UserRole::Director
                | UserRole::AdminGlobal
                | UserRole::RepresentanteLegal
                | UserRole::DirectorRRHH
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::GerenteGeneral => "GerenteGeneral",
            UserRole::JefeVentas => "JefeVentas",
            UserRole::AgenteVentas => "AgenteVentas",
            UserRole::Sostenedor => "Sostenedor",
            UserRole::Director => "Director",
            UserRole::UTP => "UTP",
            UserRole::Administrador => "Administrador",
            UserRole::Profesor => "Profesor",
            UserRole::ProfesorHonorario => "ProfesorHonorario",
            UserRole::Apoderado => "Apoderado",
            UserRole::Alumno => "Alumno",
            UserRole::Admision => "Admision",
            UserRole::RepresentanteLegal => "RepresentanteLegal",
            UserRole::DirectorRRHH => "DirectorRRHH",
            UserRole::Administrativo => "Administrativo",
            UserRole::AdminGlobal => "AdminGlobal",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GerenteGeneral" => Some(UserRole::GerenteGeneral),
            "JefeVentas" => Some(UserRole::JefeVentas),
            "AgenteVentas" => Some(UserRole::AgenteVentas),
            "Sostenedor" => Some(UserRole::Sostenedor),
            "Director" => Some(UserRole::Director),
            "UTP" => Some(UserRole::UTP),
            "Administrador" => Some(UserRole::Administrador),
            "Profesor" => Some(UserRole::Profesor),
            "ProfesorHonorario" => Some(UserRole::ProfesorHonorario),
            "Apoderado" => Some(UserRole::Apoderado),
            "Alumno" => Some(UserRole::Alumno),
            "Admision" => Some(UserRole::Admision),
            "RepresentanteLegal" => Some(UserRole::RepresentanteLegal),
            "DirectorRRHH" => Some(UserRole::DirectorRRHH),
            "Administrativo" => Some(UserRole::Administrativo),
            "AdminGlobal" => Some(UserRole::AdminGlobal),
            _ => None,
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UserRole::from_str(s).ok_or_else(|| format!("Invalid role: {s}"))
    }
}

/// Usuario del sistema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub rut: Rut,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub active: bool,
    pub admin_type: Option<AdminType>,
    pub managed_school_id: Option<Uuid>,
}

/// Payload de autenticación (email + contraseña).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

/// Payload para registrar un nuevo usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub rut: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub corporation_id: Option<String>,
    pub school_id: Option<String>,
    pub admin_type: Option<String>,
}

/// Payload para refrescar un token JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshPayload {
    pub refresh_token: String,
}

/// Respuesta de autenticación exitosa con tokens y datos del usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: User,
}

/// Resumen del dashboard principal del establecimiento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_students: i64,
    pub total_teachers: i64,
    pub attendance_today_percentage: f64,
    pub pending_alerts: i64,
    pub today_events: Vec<AgendaEvent>,
}

/// Evento de la agenda escolar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date: String,
    pub event_type: EventType,
}

/// Tipo de evento en la agenda.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventType {
    /// Bloque de clases.
    Clase,
    /// Reunión de apoderados o del equipo.
    Reunion,
    /// Evaluación o prueba.
    Evaluacion,
    /// Evento general (acto, celebración, etc.).
    Evento,
}

/// Widget de resumen de asistencia del día.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceTodayWidget {
    pub date: String,
    pub total_students: i64,
    pub present: i64,
    pub absent: i64,
    pub late: i64,
    pub justified: i64,
}

/// Widget de alertas de asistencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertWidget {
    pub alerts: Vec<AttendanceAlert>,
}

use crate::attendance::AttendanceAlert;

/// Widget de eventos de la agenda.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaWidget {
    pub events: Vec<AgendaEvent>,
}
