use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rut::Rut;

/// Condición de matrícula del estudiante según clasificación SIGE.
///
/// | Código | Significado      |
/// |--------|------------------|
/// | AL     | Alumno Regular   |
/// | RE     | Repitente        |
/// | TR     | Trasladado       |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum CondicionMatricula {
    #[serde(rename = "AL")]
    AlumnoRegular,
    #[serde(rename = "RE")]
    Repitente,
    #[serde(rename = "TR")]
    Trasladado,
}

/// Clasificación de prioridad del estudiante (SEP / PIE).
///
/// | Código | Significado              |
/// |--------|--------------------------|
/// | 1      | Prioritario (SEP)        |
/// | 2      | Preferente               |
/// | 0      | No prioritario           |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Prioritario {
    #[serde(rename = "1")]
    Si,
    #[serde(rename = "2")]
    Preferente,
    #[serde(rename = "0")]
    No,
}

/// Necesidades Educativas Especiales (NEE).
///
/// | Código | Significado         |
/// |--------|---------------------|
/// | T      | Transitoria         |
/// | P      | Permanente          |
/// | N      | Sin NEE             |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum NEE {
    #[serde(rename = "T")]
    Transitoria,
    #[serde(rename = "P")]
    Permanente,
    #[serde(rename = "N")]
    No,
}

/// Datos completos de un estudiante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: Uuid,
    pub rut: Rut,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    /// Nivel de enseñanza (ej: `"1° Básico"`).
    pub grade_level: String,
    /// Letra o identificación de la sección (ej: `"A"`).
    pub section: String,
    /// Código de nivel para exportación SIGE (ej: `"1"`, `"2"`, `"7"`).
    pub cod_nivel: Option<String>,
    pub condicion: CondicionMatricula,
    pub prioritario: Prioritario,
    pub nee: NEE,
    /// Indica si el estudiante se encuentra matriculado activamente.
    pub enrolled: bool,
}

impl Student {
    /// Retorna el nombre completo del estudiante: `"{first_name} {last_name}"`.
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Retorna el umbral de asistencia aplicable según si el estudiante
    /// presenta NEE. General: 85%, NEE: 75%.
    pub fn attendance_threshold(&self) -> f64 {
        match self.nee {
            NEE::No => crate::attendance::THRESHOLD_ASISTENCIA_GENERAL,
            _ => crate::attendance::THRESHOLD_ASISTENCIA_NEE,
        }
    }
}

/// Curso o asignatura impartida por un profesor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub grade_level: String,
    pub section: String,
    pub teacher_id: Uuid,
    /// Plan de estudios (`"FG"`, `"HC"`, `"TP"`).
    pub plan: Option<String>,
    pub classroom_id: Option<Uuid>,
}

/// Relación de matrícula entre un estudiante y un curso en un año académico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: Uuid,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub year: i32,
    pub active: bool,
}

/// Información médica y de contacto de emergencia del estudiante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalInfo {
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
}

/// Relación entre un estudiante y su apoderado o tutor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianRelationship {
    pub id: Uuid,
    pub student_id: Uuid,
    pub guardian_user_id: Uuid,
    pub guardian_name: String,
    pub guardian_rut: String,
    pub relationship: String,
    /// Autorizado para retirar al estudiante del establecimiento.
    pub authorized_pickup: bool,
    /// Recibe notificaciones del sistema.
    pub receives_notifications: bool,
}

/// Payload para crear un nuevo estudiante con datos personales e información médica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStudentPayload {
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub grade_level: String,
    pub section: String,
    pub cod_nivel: Option<String>,
    pub condicion: Option<String>,
    pub prioritario: Option<String>,
    pub nee: Option<String>,
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
}

/// Payload para actualizar los datos de un estudiante existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStudentPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub grade_level: Option<String>,
    pub section: Option<String>,
    pub cod_nivel: Option<String>,
    pub condicion: Option<String>,
    pub prioritario: Option<String>,
    pub nee: Option<String>,
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
}

/// Anotación positiva o negativa de un estudiante, según normativa MINEDUC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentAnnotation {
    pub id: Uuid,
    pub student_id: Uuid,
    pub annotation_type: String,
    pub description: String,
    pub severity: String,
    pub created_by: Option<Uuid>,
    pub teacher_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear una anotación en un estudiante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnotationPayload {
    pub student_id: Uuid,
    pub annotation_type: String,
    pub description: String,
    pub severity: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_student(nee: NEE) -> Student {
        Student {
            id: Uuid::nil(),
            rut: Rut::new_unchecked("111111111"),
            first_name: "Juan".into(),
            last_name: "Perez".into(),
            email: Some("juan@test.cl".into()),
            phone: None,
            grade_level: "1° Básico".into(),
            section: "A".into(),
            cod_nivel: None,
            condicion: CondicionMatricula::AlumnoRegular,
            prioritario: Prioritario::No,
            nee,
            enrolled: true,
        }
    }

    #[test]
    fn test_full_name() {
        let s = make_student(NEE::No);
        assert_eq!(s.full_name(), "Juan Perez");
    }

    #[test]
    fn test_attendance_threshold_general() {
        let s = make_student(NEE::No);
        assert!((s.attendance_threshold() - 85.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_attendance_threshold_nee_transitoria() {
        let s = make_student(NEE::Transitoria);
        assert!((s.attendance_threshold() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_attendance_threshold_nee_permanente() {
        let s = make_student(NEE::Permanente);
        assert!((s.attendance_threshold() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_condicion_matricula_serialization() {
        assert_eq!(
            serde_json::to_value(&CondicionMatricula::AlumnoRegular).unwrap(),
            "AL"
        );
        assert_eq!(
            serde_json::to_value(&CondicionMatricula::Repitente).unwrap(),
            "RE"
        );
        assert_eq!(
            serde_json::to_value(&CondicionMatricula::Trasladado).unwrap(),
            "TR"
        );
    }

    #[test]
    fn test_prioritario_serialization() {
        assert_eq!(serde_json::to_value(&Prioritario::Si).unwrap(), "1");
        assert_eq!(serde_json::to_value(&Prioritario::Preferente).unwrap(), "2");
        assert_eq!(serde_json::to_value(&Prioritario::No).unwrap(), "0");
    }

    #[test]
    fn test_nee_serialization() {
        assert_eq!(serde_json::to_value(&NEE::Transitoria).unwrap(), "T");
        assert_eq!(serde_json::to_value(&NEE::Permanente).unwrap(), "P");
        assert_eq!(serde_json::to_value(&NEE::No).unwrap(), "N");
    }
}
