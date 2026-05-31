//! Tipos y estructuras compartidas del sistema SchoolCBB.
//!
//! Este crate define los tipos de datos comunes utilizados por todos los
//! módulos de la aplicación: estudiantes, asistencia, calificaciones, RRHH,
//! finanzas, roles, RUT chileno, licencias, y más.

pub mod academic;
pub mod admission;
pub mod attendance;
pub mod audit;
pub mod communication;
pub mod email;
pub mod event_bus;
pub mod finance;
pub mod grades;
pub mod hr;
pub mod licensing;
pub mod modules;
pub mod reporting;
pub mod roles;
pub mod rut;
pub mod school;
pub mod student;
pub mod user;

#[cfg(feature = "db")]
pub mod db_schema;
