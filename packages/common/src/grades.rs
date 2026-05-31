use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Valor de una calificación en escala chilena (1.0 a 7.0, redondeado a 1 decimal).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grade(f64);

/// Error al construir una [`Grade`] con un valor fuera del rango válido.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GradeError {
    /// El valor debe estar entre 1.0 y 7.0.
    #[error("Nota fuera de rango: debe estar entre 1.0 y 7.0")]
    OutOfRange,
}

impl Grade {
    /// Crea una nueva calificación validando que esté entre 1.0 y 7.0.
    /// Redondea al décimo más cercano.
    pub fn new(value: f64) -> Result<Self, GradeError> {
        if !(1.0..=7.0).contains(&value) {
            return Err(GradeError::OutOfRange);
        }
        let rounded = (value * 10.0).round() / 10.0;
        Ok(Self(rounded))
    }

    /// Retorna el valor numérico de la calificación.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Retorna `true` si la nota es igual o superior a 4.0 (aprobado).
    pub fn is_passing(&self) -> bool {
        self.0 >= 4.0
    }

    /// Retorna `true` si la nota es igual o superior a 6.0 (excelente).
    pub fn is_excellent(&self) -> bool {
        self.0 >= 6.0
    }
}

/// Tipo de calificación.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum GradeType {
    /// Calificación que pondera para el promedio final.
    Sumativa,
    /// Calificación formativa (no pondera).
    Formativa,
    /// Control sorpresa (no pondera, solo informativo).
    ControlSorpresa,
}

/// Semestre académico.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Semester {
    /// Primer semestre.
    First = 1,
    /// Segundo semestre.
    Second = 2,
}

/// Mínimo de calificaciones por asignatura por semestre exigido por normativa.
pub const MIN_CALIFICACIONES_SEMESTRE: usize = 2;

/// Calificación de un estudiante en una asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectGrade {
    pub id: Uuid,
    pub student_id: Uuid,
    pub subject: String,
    pub grade: Grade,
    pub grade_type: GradeType,
    pub semester: Semester,
    pub year: i32,
    pub date: NaiveDate,
    pub teacher_id: Uuid,
    pub observation: Option<String>,
}

/// Promedio y estadísticas de un estudiante en una asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectAverage {
    pub subject: String,
    pub average: f64,
    pub grades_count: i32,
    pub min_grade: f64,
    pub max_grade: f64,
}

/// Resultado de la evaluación de promoción de un estudiante.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum PromotionResult {
    /// El estudiante cumple los requisitos para ser promovido.
    Promovido,
    /// El estudiante no cumple los requisitos y reprueba el año.
    Reprobado,
}

/// Reporte de calificaciones de un estudiante por semestre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentGradeReport {
    pub student_id: Uuid,
    pub student_name: String,
    pub semester: Semester,
    pub year: i32,
    pub subjects: Vec<SubjectAverage>,
    pub global_average: f64,
    pub is_promoted: bool,
}

impl StudentGradeReport {
    /// Calcula el promedio global a partir de los promedios por asignatura.
    pub fn calculate(promedios: &[SubjectAverage]) -> f64 {
        if promedios.is_empty() {
            return 0.0;
        }
        let sum: f64 = promedios.iter().map(|s| s.average).sum();
        sum / promedios.len() as f64
    }

    /// Evalúa si el estudiante es promovido según la normativa MINEDUC:
    ///
    /// - 0 asignaturas reprobadas: promovido.
    /// - 1 reprobada con promedio >= 3.5: promovido.
    /// - 2 reprobadas con promedio >= 3.0 en ambas: promovido.
    /// - 3 o más reprobadas: reprobado.
    pub fn evaluate_promotion(&self) -> PromotionResult {
        let failed: Vec<&SubjectAverage> =
            self.subjects.iter().filter(|s| s.average < 4.0).collect();

        match failed.len() {
            0 => PromotionResult::Promovido,
            1 => {
                let min_failed = failed[0].average;
                if min_failed >= 3.5 {
                    PromotionResult::Promovido
                } else {
                    PromotionResult::Reprobado
                }
            }
            2 => {
                let all_above_30 = failed.iter().all(|s| s.average >= 3.0);
                if all_above_30 {
                    PromotionResult::Promovido
                } else {
                    PromotionResult::Reprobado
                }
            }
            _ => PromotionResult::Reprobado,
        }
    }

    /// Verifica que todas las asignaturas tengan al menos
    /// [`MIN_CALIFICACIONES_SEMESTRE`] calificaciones registradas.
    pub fn has_minimum_grades(&self) -> bool {
        self.subjects
            .iter()
            .all(|s| s.grades_count >= MIN_CALIFICACIONES_SEMESTRE as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passing_grade() {
        let g = Grade::new(4.0).unwrap();
        assert!(g.is_passing());
    }

    #[test]
    fn test_failing_grade() {
        let g = Grade::new(3.9).unwrap();
        assert!(!g.is_passing());
    }

    #[test]
    fn test_out_of_range() {
        assert!(Grade::new(0.9).is_err());
        assert!(Grade::new(7.1).is_err());
    }

    #[test]
    fn test_rounding() {
        let g = Grade::new(4.55).unwrap();
        assert_eq!(g.value(), 4.6);
    }

    #[test]
    fn test_promotion_all_pass() {
        let report = StudentGradeReport {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            semester: Semester::First,
            year: 2025,
            subjects: vec![
                SubjectAverage {
                    subject: "MAT".into(),
                    average: 5.0,
                    grades_count: 3,
                    min_grade: 4.0,
                    max_grade: 6.0,
                },
                SubjectAverage {
                    subject: "LEN".into(),
                    average: 4.5,
                    grades_count: 3,
                    min_grade: 4.0,
                    max_grade: 5.0,
                },
            ],
            global_average: 4.75,
            is_promoted: true,
        };
        assert_eq!(report.evaluate_promotion(), PromotionResult::Promovido);
    }

    #[test]
    fn test_promotion_one_fail_above_35() {
        let report = StudentGradeReport {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            semester: Semester::First,
            year: 2025,
            subjects: vec![
                SubjectAverage {
                    subject: "MAT".into(),
                    average: 3.6,
                    grades_count: 2,
                    min_grade: 3.0,
                    max_grade: 4.0,
                },
                SubjectAverage {
                    subject: "LEN".into(),
                    average: 5.0,
                    grades_count: 3,
                    min_grade: 4.0,
                    max_grade: 6.0,
                },
            ],
            global_average: 4.3,
            is_promoted: false,
        };
        assert_eq!(report.evaluate_promotion(), PromotionResult::Promovido);
    }

    #[test]
    fn test_promotion_one_fail_below_35() {
        let report = StudentGradeReport {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            semester: Semester::First,
            year: 2025,
            subjects: vec![
                SubjectAverage {
                    subject: "MAT".into(),
                    average: 3.4,
                    grades_count: 2,
                    min_grade: 3.0,
                    max_grade: 4.0,
                },
                SubjectAverage {
                    subject: "LEN".into(),
                    average: 5.0,
                    grades_count: 3,
                    min_grade: 4.0,
                    max_grade: 6.0,
                },
            ],
            global_average: 4.2,
            is_promoted: false,
        };
        assert_eq!(report.evaluate_promotion(), PromotionResult::Reprobado);
    }

    #[test]
    fn test_promotion_two_fails_both_above_30() {
        let report = StudentGradeReport {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            semester: Semester::First,
            year: 2025,
            subjects: vec![
                SubjectAverage {
                    subject: "MAT".into(),
                    average: 3.5,
                    grades_count: 2,
                    min_grade: 3.0,
                    max_grade: 4.0,
                },
                SubjectAverage {
                    subject: "LEN".into(),
                    average: 3.2,
                    grades_count: 2,
                    min_grade: 3.0,
                    max_grade: 5.0,
                },
                SubjectAverage {
                    subject: "HIS".into(),
                    average: 5.0,
                    grades_count: 3,
                    min_grade: 4.0,
                    max_grade: 6.0,
                },
            ],
            global_average: 3.9,
            is_promoted: false,
        };
        assert_eq!(report.evaluate_promotion(), PromotionResult::Promovido);
    }

    #[test]
    fn test_promotion_three_fails() {
        let report = StudentGradeReport {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            semester: Semester::First,
            year: 2025,
            subjects: vec![
                SubjectAverage {
                    subject: "MAT".into(),
                    average: 3.0,
                    grades_count: 2,
                    min_grade: 2.0,
                    max_grade: 4.0,
                },
                SubjectAverage {
                    subject: "LEN".into(),
                    average: 3.5,
                    grades_count: 2,
                    min_grade: 3.0,
                    max_grade: 5.0,
                },
                SubjectAverage {
                    subject: "HIS".into(),
                    average: 3.0,
                    grades_count: 2,
                    min_grade: 3.0,
                    max_grade: 4.0,
                },
            ],
            global_average: 3.17,
            is_promoted: false,
        };
        assert_eq!(report.evaluate_promotion(), PromotionResult::Reprobado);
    }
}
