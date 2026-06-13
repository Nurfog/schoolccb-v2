use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub use schoolccb_common::auth::Claims;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub rut: String,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub active: bool,
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub admin_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked: bool,
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_user(
    pool: &PgPool,
    rut: &str,
    name: &str,
    email: &str,
    password: &str,
    role: &str,
    corporation_id: Option<Uuid>,
    school_id: Option<Uuid>,
    admin_type: Option<&str>,
) -> Result<UserRow, sqlx::Error> {
    let hash = hash_password(password);

    let id = Uuid::new_v4();
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type)
        VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8, $9)
        RETURNING id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type
        "#,
    )
    .bind(id)
    .bind(rut)
    .bind(name)
    .bind(email)
    .bind(&hash)
    .bind(role)
    .bind(corporation_id)
    .bind(school_id)
    .bind(admin_type)
    .fetch_one(pool)
    .await
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    duration_days: i64,
) -> Result<(String, RefreshTokenRow), sqlx::Error> {
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let token_hash = hash_token(&token);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(duration_days);
    let id = Uuid::new_v4();

    let row = sqlx::query_as::<_, RefreshTokenRow>(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, revoked)
        VALUES ($1, $2, $3, $4, false)
        RETURNING id, user_id, token_hash, expires_at, revoked
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok((token, row))
}

pub async fn find_refresh_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<RefreshTokenRow>, sqlx::Error> {
    let token_hash = hash_token(token);
    sqlx::query_as::<_, RefreshTokenRow>(
        r#"
        SELECT id, user_id, token_hash, expires_at, revoked
        FROM refresh_tokens
        WHERE token_hash = $1 AND revoked = false AND expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
}

pub async fn revoke_refresh_token(pool: &PgPool, token_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_all_user_tokens(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub async fn seed_admin(pool: &PgPool) -> Result<(), sqlx::Error> {
    let email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@colegio.cl".to_string());
    let password =
        std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let name = std::env::var("ADMIN_NAME").unwrap_or_else(|_| "Administrador".to_string());
    let rut = std::env::var("ADMIN_RUT").unwrap_or_else(|_| "11.111.111-1".to_string());

    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(pool)
        .await?;

    if exists.0 > 0 {
        return Ok(());
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, rut, name, email, password_hash, role, active)
         VALUES ($1, $2, $3, $4, $5, 'Administrador', true)",
    )
    .bind(Uuid::new_v4())
    .bind(&rut)
    .bind(&name)
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await?;

    tracing::info!("Admin user created: {email}");
    Ok(())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

pub async fn update_user_profile(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    email: &str,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "UPDATE users SET name = $1, email = $2, updated_at = NOW() WHERE id = $3
         RETURNING id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type",
    )
    .bind(name)
    .bind(email)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn change_password(
    pool: &PgPool,
    id: Uuid,
    new_password: &str,
) -> Result<(), sqlx::Error> {
    let hash = hash_password(new_password);
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SchoolConfigRow {
    pub id: Uuid,
    pub corporation_id: Option<Uuid>,
    pub school_name: String,
    pub school_logo_url: String,
    pub primary_color: String,
    pub secondary_color: String,
}

pub async fn get_branding(pool: &PgPool, corporation_id: Option<Uuid>) -> Result<Option<SchoolConfigRow>, sqlx::Error> {
    match corporation_id {
        Some(cid) => sqlx::query_as::<_, SchoolConfigRow>(
            "SELECT id, corporation_id, school_name, school_logo_url, primary_color, secondary_color
             FROM school_config WHERE corporation_id = $1 LIMIT 1",
        )
        .bind(cid)
        .fetch_optional(pool)
        .await,
        None => sqlx::query_as::<_, SchoolConfigRow>(
            "SELECT id, corporation_id, school_name, school_logo_url, primary_color, secondary_color
             FROM school_config LIMIT 1",
        )
        .fetch_optional(pool)
        .await,
    }
}

pub async fn upsert_branding(
    pool: &PgPool,
    corporation_id: Option<Uuid>,
    school_name: &str,
    school_logo_url: &str,
    primary_color: &str,
    secondary_color: &str,
) -> Result<SchoolConfigRow, sqlx::Error> {
    let existing = get_branding(pool, corporation_id).await?;
    if let Some(_row) = existing {
        match corporation_id {
            Some(cid) => sqlx::query_as::<_, SchoolConfigRow>(
                "UPDATE school_config SET school_name = $1, school_logo_url = $2, primary_color = $3, secondary_color = $4, updated_at = NOW()
                 WHERE corporation_id = $5
                 RETURNING id, corporation_id, school_name, school_logo_url, primary_color, secondary_color",
            )
            .bind(school_name)
            .bind(school_logo_url)
            .bind(primary_color)
            .bind(secondary_color)
            .bind(cid)
            .fetch_one(pool)
            .await,
            None => sqlx::query_as::<_, SchoolConfigRow>(
                "UPDATE school_config SET school_name = $1, school_logo_url = $2, primary_color = $3, secondary_color = $4, updated_at = NOW()
                 RETURNING id, corporation_id, school_name, school_logo_url, primary_color, secondary_color",
            )
            .bind(school_name)
            .bind(school_logo_url)
            .bind(primary_color)
            .bind(secondary_color)
            .fetch_one(pool)
            .await,
        }
    } else {
        let id = Uuid::new_v4();
        match corporation_id {
            Some(cid) => sqlx::query_as::<_, SchoolConfigRow>(
                "INSERT INTO school_config (id, corporation_id, school_name, school_logo_url, primary_color, secondary_color)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING id, corporation_id, school_name, school_logo_url, primary_color, secondary_color",
            )
            .bind(id)
            .bind(cid)
            .bind(school_name)
            .bind(school_logo_url)
            .bind(primary_color)
            .bind(secondary_color)
            .fetch_one(pool)
            .await,
            None => sqlx::query_as::<_, SchoolConfigRow>(
                "INSERT INTO school_config (id, school_name, school_logo_url, primary_color, secondary_color)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id, corporation_id, school_name, school_logo_url, primary_color, secondary_color",
            )
            .bind(id)
            .bind(school_name)
            .bind(school_logo_url)
            .bind(primary_color)
            .bind(secondary_color)
            .fetch_one(pool)
            .await,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreferenceRow {
    pub user_id: Uuid,
    pub show_module_manager: bool,
}

pub async fn get_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserPreferenceRow, sqlx::Error> {
    sqlx::query_as::<_, UserPreferenceRow>(
        "SELECT user_id, show_module_manager FROM user_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|opt| {
        opt.unwrap_or(UserPreferenceRow {
            user_id,
            show_module_manager: true,
        })
    })
}

pub async fn seed_roles(pool: &PgPool) {
    let default_roles = [
        (
            "Sostenedor",
            "Dueño del colegio, acceso total al sistema",
            true,
        ),
        (
            "Administrador",
            "Administrador del sistema, gestión completa",
            true,
        ),
        ("Director", "Director académico, supervisión general", true),
        ("UTP", "Unidad Técnico Pedagógica, gestión curricular", true),
        ("Profesor", "Docente, gestión de cursos y notas", true),
        (
            "Apoderado",
            "Padre/madre/apoderado, consulta de pupilos",
            true,
        ),
        ("Alumno", "Estudiante, consulta de notas y asistencia", true),
        (
            "Admision",
            "Equipo de admisión, gestión de postulantes",
            true,
        ),
        (
            "GerenteGeneral",
            "Gerente general del área comercial",
            true,
        ),
        (
            "JefeVentas",
            "Jefe del equipo de ventas",
            true,
        ),
        (
            "AgenteVentas",
            "Agente de ventas comercial",
            true,
        ),
        (
            "RepresentanteLegal",
            "Representante legal de la corporación",
            true,
        ),
        (
            "DirectorRRHH",
            "Director de Recursos Humanos",
            true,
        ),
        (
            "ProfesorHonorario",
            "Profesor honorario sin contrato formal",
            true,
        ),
        (
            "Administrativo",
            "Personal administrativo del establecimiento",
            true,
        ),
        (
            "AdminGlobal",
            "Administrador global con acceso a toda la corporación",
            true,
        ),
    ];

    for (name, description, is_system) in &default_roles {
        let existing: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

        if existing.0 == 0 {
            sqlx::query(
                "INSERT INTO roles (id, name, description, is_system) VALUES ($1, $2, $3, $4)",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(name)
            .bind(description)
            .bind(is_system)
            .execute(pool)
            .await
            .unwrap_or_else(|_| {
                tracing::warn!("Could not seed role: {}", name);
                Default::default()
            });
        }
    }
}

pub async fn seed_permission_definitions(pool: &PgPool) {
    use schoolccb_common::roles::Module;
    for (module_name, resources) in Module::all() {
        for resource in resources {
            let existing: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM permission_definitions WHERE module = $1 AND resource = $2",
            )
            .bind(module_name)
            .bind(resource)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

            if existing.0 == 0 {
                let label = format!("{}/{}", module_name, resource);
                sqlx::query(
                    "INSERT INTO permission_definitions (id, module, resource, label) VALUES ($1, $2, $3, $4)",
                )
                .bind(uuid::Uuid::new_v4())
                .bind(module_name)
                .bind(resource)
                .bind(&label)
                .execute(pool)
                .await
                .unwrap_or_else(|_| {
                    tracing::warn!("Could not seed permission: {}", label);
                    Default::default()
                });
            }
        }
    }
}

pub async fn update_preferences(
    pool: &PgPool,
    user_id: Uuid,
    show_module_manager: bool,
) -> Result<UserPreferenceRow, sqlx::Error> {
    sqlx::query_as::<_, UserPreferenceRow>(
        "INSERT INTO user_preferences (user_id, show_module_manager)
         VALUES ($1, $2)
         ON CONFLICT (user_id)
         DO UPDATE SET show_module_manager = $2, updated_at = NOW()
         RETURNING user_id, show_module_manager",
    )
    .bind(user_id)
    .bind(show_module_manager)
    .fetch_one(pool)
    .await
}

pub async fn seed_gerente_general(pool: &PgPool) {
    let email = std::env::var("GERENTE_EMAIL").unwrap_or_else(|_| "juan.allende@gmail.com".into());
    let password = std::env::var("GERENTE_PASSWORD").unwrap_or_else(|_| "admin123".into());
    let name = std::env::var("GERENTE_NAME").unwrap_or_else(|_| "Juan Allende".into());
    let rut = std::env::var("GERENTE_RUT").unwrap_or_else(|_| "22.222.222-2".into());

    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if exists.0 > 0 {
        return;
    }

    let hash = hash_password(&password);
    sqlx::query(
        "INSERT INTO users (id, rut, name, email, password_hash, role, active)
         VALUES ($1, $2, $3, $4, $5, 'GerenteGeneral', true)",
    )
    .bind(Uuid::new_v4())
    .bind(&rut)
    .bind(&name)
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await
    .unwrap_or_else(|_| {
        tracing::warn!("Could not seed gerente general");
        Default::default()
    });

    tracing::info!("Gerente General created: {email}");
}

pub async fn seed_license_plans(pool: &PgPool) {
    let exists: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM license_plans WHERE name = 'Básico'")
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    if exists.0 > 0 {
        return;
    }

fn base_system_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("users", "Usuarios y Perfiles"),
        ("roles", "Roles y Permisos"),
        ("config", "Configuración"),
    ]
}

    let plans = vec![
        (
            "Básico",
            "Gestión escolar esencial: dashboard, alumnos, cursos, asistencia y notas",
            49900.0,
            49900.0 * 12.0,
            false,
            1,
            [vec![
                ("dashboard", "Dashboard"),
                ("students", "Gestión de Alumnos"),
                ("courses", "Cursos"),
                ("enrollments", "Matrículas"),
                ("attendance", "Asistencia"),
                ("grades", "Calificaciones"),
            ], base_system_modules()].concat(),
        ),
        (
            "Profesional",
            "Incluye todo lo del plan Básico más RRHH, finanzas, admisión CRM y reportes avanzados",
            99900.0,
            99900.0 * 12.0,
            true,
            2,
            [vec![
                ("dashboard", "Dashboard"),
                ("students", "Gestión de Alumnos"),
                ("courses", "Cursos"),
                ("enrollments", "Matrículas"),
                ("subjects", "Asignaturas"),
                ("grade-levels", "Niveles"),
                ("academic-years", "Años Académicos"),
                ("classrooms", "Salas"),
                ("attendance", "Asistencia"),
                ("grades", "Calificaciones"),
                ("hr", "Recursos Humanos"),
                ("payroll", "Remuneraciones"),
                ("my-portal", "Portal Auto-consulta"),
                ("finance", "Finanzas"),
                ("admission", "Admisión CRM"),
                ("reports", "Reportes"),
                ("notifications", "Centro de Mensajería"),
                ("agenda", "Agenda Escolar"),
            ], base_system_modules()].concat(),
        ),
        (
            "Corporativo",
            "Solución completa multi-colegio con SIGE, API y todos los módulos",
            199900.0,
            199900.0 * 12.0,
            true,
            3,
            [vec![
                ("dashboard", "Dashboard"),
                ("students", "Gestión de Alumnos"),
                ("courses", "Cursos"),
                ("enrollments", "Matrículas"),
                ("subjects", "Asignaturas"),
                ("grade-levels", "Niveles"),
                ("academic-years", "Años Académicos"),
                ("classrooms", "Salas"),
                ("attendance", "Asistencia"),
                ("grades", "Calificaciones"),
                ("hr", "Recursos Humanos"),
                ("payroll", "Remuneraciones"),
                ("my-portal", "Portal Auto-consulta"),
                ("finance", "Finanzas"),
                ("admission", "Admisión CRM"),
                ("reports", "Reportes"),
                ("notifications", "Centro de Mensajería"),
                ("agenda", "Agenda Escolar"),
                ("sige", "SIGE / MINEDUC"),
                ("corporations", "Multi-colegio"),
                ("complaints", "Ley Karin — Denuncias"),
            ], base_system_modules()].concat(),
        ),
    ];

    for (name, desc, price_monthly, price_yearly, featured, sort_order, modules) in plans {
        let plan_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO license_plans (id, name, description, price_monthly, price_yearly, featured, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(plan_id)
        .bind(name)
        .bind(desc)
        .bind(price_monthly)
        .bind(price_yearly)
        .bind(featured)
        .bind(sort_order)
        .execute(pool)
        .await
        .unwrap_or_else(|_| {
            tracing::warn!("Could not seed plan: {}", name);
            Default::default()
        });

        for (module_key, module_name) in modules {
            sqlx::query(
                "INSERT INTO plan_modules (id, plan_id, module_key, module_name, included)
                 VALUES ($1, $2, $3, $4, true)",
            )
            .bind(Uuid::new_v4())
            .bind(plan_id)
            .bind(module_key)
            .bind(module_name)
            .execute(pool)
            .await
            .unwrap_or_else(|_| {
                tracing::warn!("Could not seed module {module_key} for plan {name}");
                Default::default()
            });
        }
    }

    tracing::info!("License plans seeded: Básico, Profesional, Corporativo");
}

pub async fn seed_default_school(pool: &PgPool) {
    let exists: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM corporations WHERE name = 'Corporación Educativa'")
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    if exists.0 > 0 {
        return;
    }

    let corp_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO corporations (id, name, rut, active) VALUES ($1, 'Corporación Educativa', '99.999.999-9', true)",
    )
    .bind(corp_id)
    .execute(pool)
    .await
    .unwrap_or_else(|_| {
        tracing::warn!("Could not seed default corporation");
        Default::default()
    });

    sqlx::query(
        "INSERT INTO schools (id, corporation_id, name, active) VALUES ($1, $2, 'Colegio Predeterminado', true)",
    )
    .bind(Uuid::new_v4())
    .bind(corp_id)
    .execute(pool)
    .await
    .unwrap_or_else(|_| {
        tracing::warn!("Could not seed default school");
        Default::default()
    });

    tracing::info!("Default corporation and school created");
}
