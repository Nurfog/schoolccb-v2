#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
pub async fn run(pool: &PgPool) {
    // Set search_path to include both public (CRM/sales) and colegios (school data) schemas
    sqlx::query("SET search_path TO public, colegios")
        .execute(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Could not set search_path: {e}");
            Default::default()
        });

    let statements = vec![
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            rut VARCHAR(12) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            role VARCHAR(20) NOT NULL DEFAULT 'Profesor',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS students (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            rut VARCHAR(12) UNIQUE NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            phone VARCHAR(20),
            grade_level VARCHAR(20) NOT NULL,
            section VARCHAR(10) NOT NULL,
            cod_nivel VARCHAR(10),
            condicion VARCHAR(2) NOT NULL DEFAULT 'AL',
            prioritario VARCHAR(1) NOT NULL DEFAULT '0',
            nee VARCHAR(1) NOT NULL DEFAULT 'N',
            diseases TEXT,
            allergies TEXT,
            emergency_contact_name VARCHAR(255),
            emergency_contact_phone VARCHAR(20),
            emergency_contact_relation VARCHAR(100),
            enrolled BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS diseases TEXT",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS allergies TEXT",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS emergency_contact_name VARCHAR(255)",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS emergency_contact_phone VARCHAR(20)",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS emergency_contact_relation VARCHAR(100)",
        "CREATE TABLE IF NOT EXISTS courses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            subject VARCHAR(255) NOT NULL,
            grade_level VARCHAR(20) NOT NULL,
            section VARCHAR(10) NOT NULL,
            teacher_id UUID REFERENCES users(id),
            plan VARCHAR(2),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE courses ADD COLUMN IF NOT EXISTS plan VARCHAR(2)",
        "ALTER TABLE courses ALTER COLUMN subject DROP NOT NULL",
        "CREATE TABLE IF NOT EXISTS enrollments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            course_id UUID NOT NULL REFERENCES courses(id),
            year INTEGER NOT NULL,
            active BOOLEAN NOT NULL DEFAULT true,
            UNIQUE(student_id, course_id, year)
        )",
        "CREATE TABLE IF NOT EXISTS attendance (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            course_id UUID NOT NULL REFERENCES courses(id),
            date DATE NOT NULL,
            time TIME,
            status VARCHAR(20) NOT NULL DEFAULT 'Presente',
            subject VARCHAR(255) NOT NULL,
            teacher_id UUID NOT NULL REFERENCES users(id),
            observation TEXT,
            school_id UUID REFERENCES schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(student_id, course_id, date, subject)
        )",
        "CREATE TABLE IF NOT EXISTS grades (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            subject VARCHAR(255) NOT NULL,
            grade DOUBLE PRECISION NOT NULL,
            grade_type VARCHAR(20) NOT NULL DEFAULT 'Sumativa',
            semester INTEGER NOT NULL DEFAULT 1,
            year INTEGER NOT NULL,
            date DATE NOT NULL,
            teacher_id UUID NOT NULL REFERENCES users(id),
            observation TEXT,
            school_id UUID REFERENCES schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS agenda_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            description TEXT,
            event_date DATE NOT NULL,
            event_type VARCHAR(20) NOT NULL DEFAULT 'Evento',
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_attendance_date ON attendance(date)",
        "CREATE INDEX IF NOT EXISTS idx_attendance_student ON attendance(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_grades_student ON grades(student_id)",
        "CREATE TABLE IF NOT EXISTS refresh_tokens (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id),
            token_hash VARCHAR(255) NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            revoked BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS guardian_relationships (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            guardian_user_id UUID NOT NULL REFERENCES users(id),
            relationship VARCHAR(50) NOT NULL,
            authorized_pickup BOOLEAN NOT NULL DEFAULT false,
            receives_notifications BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(student_id, guardian_user_id)
        )",
        "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON refresh_tokens(token_hash)",
        "CREATE TABLE IF NOT EXISTS subjects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            code VARCHAR(20) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            level VARCHAR(20),
            hours_per_week INTEGER NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS academic_years (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            year INTEGER NOT NULL UNIQUE,
            name VARCHAR(255) NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS academic_periods (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            year INTEGER NOT NULL,
            semester INTEGER NOT NULL DEFAULT 1,
            start_date DATE NOT NULL,
            end_date DATE NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS course_subjects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            course_id UUID NOT NULL REFERENCES courses(id),
            subject_id UUID NOT NULL REFERENCES subjects(id),
            teacher_id UUID NOT NULL REFERENCES users(id),
            academic_year INTEGER NOT NULL,
            hours_per_week INTEGER NOT NULL DEFAULT 0,
            UNIQUE(course_id, subject_id, academic_year),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS grade_categories (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            course_subject_id UUID NOT NULL REFERENCES course_subjects(id),
            name VARCHAR(255) NOT NULL,
            weight_percentage DOUBLE PRECISION NOT NULL DEFAULT 100.0,
            evaluation_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE grades ADD COLUMN IF NOT EXISTS course_subject_id UUID REFERENCES course_subjects(id)",
        "ALTER TABLE grades ADD COLUMN IF NOT EXISTS category_id UUID REFERENCES grade_categories(id)",
        "CREATE INDEX IF NOT EXISTS idx_grades_subject ON grades(subject)",
        "CREATE INDEX IF NOT EXISTS idx_grades_course_subject ON grades(course_subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_guardian_relationships_student ON guardian_relationships(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_guardian_relationships_guardian ON guardian_relationships(guardian_user_id)",
        "CREATE INDEX IF NOT EXISTS idx_subjects_active ON subjects(active)",
        "CREATE INDEX IF NOT EXISTS idx_academic_periods_year ON academic_periods(year)",
        "CREATE INDEX IF NOT EXISTS idx_course_subjects_course ON course_subjects(course_id)",
        "CREATE INDEX IF NOT EXISTS idx_course_subjects_teacher ON course_subjects(teacher_id)",
        "CREATE INDEX IF NOT EXISTS idx_grade_categories_subject ON grade_categories(course_subject_id)",
        "CREATE TABLE IF NOT EXISTS messages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            sender_id UUID NOT NULL REFERENCES users(id),
            receiver_id UUID NOT NULL REFERENCES users(id),
            subject VARCHAR(255) NOT NULL,
            body TEXT NOT NULL,
            read BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS interview_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            teacher_id UUID NOT NULL REFERENCES users(id),
            date DATE NOT NULL DEFAULT CURRENT_DATE,
            reason TEXT NOT NULL,
            notes TEXT NOT NULL,
            follow_up TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS fees (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            description VARCHAR(255) NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            due_date DATE NOT NULL,
            paid BOOLEAN NOT NULL DEFAULT false,
            paid_date DATE,
            paid_amount DOUBLE PRECISION,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS payments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            fee_id UUID NOT NULL REFERENCES fees(id),
            student_id UUID NOT NULL REFERENCES students(id),
            amount DOUBLE PRECISION NOT NULL,
            payment_date DATE NOT NULL DEFAULT CURRENT_DATE,
            payment_method VARCHAR(50) NOT NULL DEFAULT 'Efectivo',
            reference VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS scholarships (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            name VARCHAR(255) NOT NULL,
            discount_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
            approved BOOLEAN NOT NULL DEFAULT false,
            approved_by UUID REFERENCES users(id),
            valid_from DATE NOT NULL,
            valid_until DATE NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_messages_receiver ON messages(receiver_id)",
        "CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id)",
        "CREATE INDEX IF NOT EXISTS idx_interview_logs_student ON interview_logs(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_fees_student ON fees(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_payments_fee ON payments(fee_id)",
        "CREATE INDEX IF NOT EXISTS idx_scholarships_student ON scholarships(student_id)",
        "CREATE TABLE IF NOT EXISTS user_favorites (
            user_id UUID NOT NULL REFERENCES users(id),
            module_id VARCHAR(50) NOT NULL,
            PRIMARY KEY (user_id, module_id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS school_config (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID REFERENCES corporations(id),
            school_name VARCHAR(255) NOT NULL DEFAULT '',
            school_logo_url VARCHAR(500) NOT NULL DEFAULT '',
            primary_color VARCHAR(7) NOT NULL DEFAULT '#1A2B3C',
            secondary_color VARCHAR(7) NOT NULL DEFAULT '#243B4F',
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS user_preferences (
            user_id UUID PRIMARY KEY REFERENCES users(id),
            show_module_manager BOOLEAN NOT NULL DEFAULT true,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS subject_hours (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            subject_id UUID NOT NULL REFERENCES subjects(id),
            level VARCHAR(20) NOT NULL,
            hours_per_week INTEGER NOT NULL DEFAULT 0,
            UNIQUE(subject_id, level)
        )",
        "CREATE INDEX IF NOT EXISTS idx_subject_hours_subject ON subject_hours(subject_id)",
        "CREATE TABLE IF NOT EXISTS classrooms (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            capacity INTEGER NOT NULL DEFAULT 30,
            location VARCHAR(255),
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE courses ADD COLUMN IF NOT EXISTS classroom_id UUID REFERENCES classrooms(id)",
        "CREATE TABLE IF NOT EXISTS pipeline_stages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_final BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS prospects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            rut VARCHAR(12),
            email VARCHAR(255),
            phone VARCHAR(20),
            current_stage_id UUID REFERENCES pipeline_stages(id),
            assigned_user_id UUID REFERENCES users(id),
            source VARCHAR(50),
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_prospects_stage ON prospects(current_stage_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospects_assigned ON prospects(assigned_user_id)",
        "CREATE TABLE IF NOT EXISTS prospect_activities (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID NOT NULL REFERENCES prospects(id) ON DELETE CASCADE,
            activity_type VARCHAR(20) NOT NULL DEFAULT 'note',
            subject VARCHAR(255) NOT NULL,
            description TEXT,
            scheduled_at TIMESTAMPTZ,
            is_completed BOOLEAN NOT NULL DEFAULT false,
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_prospect_activities_prospect ON prospect_activities(prospect_id)",
        "CREATE TABLE IF NOT EXISTS prospect_documents (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID NOT NULL REFERENCES prospects(id) ON DELETE CASCADE,
            file_name VARCHAR(255) NOT NULL,
            s3_url VARCHAR(500),
            doc_type VARCHAR(50) NOT NULL DEFAULT 'other',
            is_verified BOOLEAN NOT NULL DEFAULT false,
            uploaded_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_prospect_documents_prospect ON prospect_documents(prospect_id)",
        "CREATE TABLE IF NOT EXISTS prospect_reminders (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID NOT NULL REFERENCES prospects(id) ON DELETE CASCADE,
            reminder_type VARCHAR(30) NOT NULL DEFAULT 'follow_up',
            title VARCHAR(255) NOT NULL,
            description TEXT,
            remind_at TIMESTAMPTZ NOT NULL,
            is_sent BOOLEAN NOT NULL DEFAULT false,
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_prospect_reminders_prospect ON prospect_reminders(prospect_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospect_reminders_unsent ON prospect_reminders(remind_at) WHERE is_sent = false",
        "CREATE TABLE IF NOT EXISTS student_annotations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id) ON DELETE CASCADE,
            annotation_type VARCHAR(30) NOT NULL DEFAULT 'observacion',
            description TEXT NOT NULL,
            severity VARCHAR(20) NOT NULL DEFAULT 'leve',
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_student_annotations_student ON student_annotations(student_id)",
        "CREATE TABLE IF NOT EXISTS enrollment_contracts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL,
            school_id UUID NOT NULL,
            grade_level VARCHAR(50) NOT NULL,
            guardian_user_id UUID,
            scholarship_id UUID,
            annexes JSONB DEFAULT '[]',
            total_fee DOUBLE PRECISION NOT NULL DEFAULT 0,
            discount_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            final_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            payment_plan VARCHAR(20) DEFAULT 'monthly',
            status VARCHAR(20) NOT NULL DEFAULT 'draft',
            signed_at TIMESTAMPTZ,
            enrolled_at TIMESTAMPTZ,
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS holidays (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            school_id UUID,
            date DATE NOT NULL,
            name VARCHAR(255) NOT NULL,
            holiday_type VARCHAR(20) NOT NULL DEFAULT 'legal',
            is_recurring BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_holidays_date ON holidays(date)",
        "CREATE TABLE IF NOT EXISTS complementary_subjects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            school_id UUID NOT NULL,
            course_id UUID,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            max_students INT DEFAULT 0,
            teacher_id UUID,
            schedule_info JSONB DEFAULT '{}',
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_complementary_subjects_course ON complementary_subjects(course_id)",
        "CREATE TABLE IF NOT EXISTS complementary_subject_enrollments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            subject_id UUID NOT NULL REFERENCES complementary_subjects(id) ON DELETE CASCADE,
            student_id UUID NOT NULL,
            enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_complementary_enrollments_subject ON complementary_subject_enrollments(subject_id)",
        "CREATE TABLE IF NOT EXISTS grade_levels (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            code VARCHAR(20) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            plan VARCHAR(20),
            sort_order INTEGER NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS password_reset_tokens (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id),
            token_hash VARCHAR(255) NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            used BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_hash ON password_reset_tokens(token_hash)",
        "CREATE TABLE IF NOT EXISTS audit_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            entity_type VARCHAR(50) NOT NULL,
            entity_id UUID NOT NULL,
            action VARCHAR(20) NOT NULL,
            user_id UUID,
            changes JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at)",
        "CREATE TABLE IF NOT EXISTS event_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            event_type VARCHAR(50) NOT NULL,
            payload JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_event_log_type ON event_log(event_type)",
        "CREATE INDEX IF NOT EXISTS idx_event_log_created ON event_log(created_at)",
        "CREATE TABLE IF NOT EXISTS payment_transactions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            fee_id UUID NOT NULL REFERENCES fees(id),
            token VARCHAR(255) NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            status VARCHAR(20) NOT NULL DEFAULT 'INITIALIZED',
            authorization_code VARCHAR(50),
            payment_type VARCHAR(20),
            gateway_url TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_payment_transactions_token ON payment_transactions(token)",
        "CREATE INDEX IF NOT EXISTS idx_payment_transactions_fee ON payment_transactions(fee_id)",
        "CREATE TABLE IF NOT EXISTS custom_field_definitions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            entity_type VARCHAR(50) NOT NULL,
            field_name VARCHAR(100) NOT NULL,
            field_type VARCHAR(20) NOT NULL DEFAULT 'text',
            is_required BOOLEAN NOT NULL DEFAULT false,
            options JSONB,
            sort_order INTEGER NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_custom_fields_entity ON custom_field_definitions(entity_type)",
        "CREATE TABLE IF NOT EXISTS custom_field_values (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            field_definition_id UUID NOT NULL REFERENCES custom_field_definitions(id) ON DELETE CASCADE,
            entity_id UUID NOT NULL,
            value TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(field_definition_id, entity_id)
        )",
        "CREATE INDEX IF NOT EXISTS idx_custom_field_values_entity ON custom_field_values(entity_id)",
        "CREATE TABLE IF NOT EXISTS roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(50) UNIQUE NOT NULL,
            description TEXT,
            is_system BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS permission_definitions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            module VARCHAR(50) NOT NULL,
            resource VARCHAR(50) NOT NULL,
            label VARCHAR(100) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(module, resource)
        )",
        "CREATE TABLE IF NOT EXISTS role_permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            permission_id UUID NOT NULL REFERENCES permission_definitions(id) ON DELETE CASCADE,
            can_create BOOLEAN NOT NULL DEFAULT false,
            can_read BOOLEAN NOT NULL DEFAULT false,
            can_update BOOLEAN NOT NULL DEFAULT false,
            can_delete BOOLEAN NOT NULL DEFAULT false,
            UNIQUE(role_id, permission_id)
        )",
        "CREATE TABLE IF NOT EXISTS user_roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            UNIQUE(user_id, role_id)
        )",
        "CREATE TABLE IF NOT EXISTS corporations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            rut VARCHAR(12) UNIQUE,
            logo_url VARCHAR(500),
            legal_representative_name VARCHAR(255),
            legal_representative_rut VARCHAR(12),
            legal_representative_email VARCHAR(255),
            settings JSONB DEFAULT '{}',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS schools (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID NOT NULL REFERENCES corporations(id),
            name VARCHAR(255) NOT NULL,
            address VARCHAR(500),
            phone VARCHAR(20),
            logo_url VARCHAR(500),
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS corporation_id UUID REFERENCES corporations(id)",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS admin_type VARCHAR(20)",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS managed_school_id UUID REFERENCES schools(id)",
        "ALTER TABLE corporations ADD COLUMN IF NOT EXISTS legal_representative_name VARCHAR(255)",
        "ALTER TABLE corporations ADD COLUMN IF NOT EXISTS legal_representative_rut VARCHAR(12)",
        "ALTER TABLE corporations ADD COLUMN IF NOT EXISTS legal_representative_email VARCHAR(255)",
        "ALTER TABLE license_plans ADD COLUMN IF NOT EXISTS is_custom BOOLEAN NOT NULL DEFAULT false",
        "ALTER TABLE license_plans ADD COLUMN IF NOT EXISTS show_in_portal BOOLEAN NOT NULL DEFAULT true",
        "ALTER TABLE plan_modules ADD COLUMN IF NOT EXISTS sub_modules JSONB DEFAULT '[]'",
        "ALTER TABLE students ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE courses ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE enrollments ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE fees ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE scholarships ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE grades ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE attendance ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id)",
        "ALTER TABLE school_config ADD COLUMN IF NOT EXISTS corporation_id UUID REFERENCES corporations(id)",
        "CREATE TABLE IF NOT EXISTS employees (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            school_id UUID REFERENCES schools(id),
            rut VARCHAR(12) UNIQUE NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            phone VARCHAR(20),
            position VARCHAR(100),
            category VARCHAR(30),
            hire_date DATE,
            vacation_days_available REAL NOT NULL DEFAULT 15.0,
            active BOOLEAN NOT NULL DEFAULT true,
            supervisor_id UUID REFERENCES employees(id),
            user_id UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS category VARCHAR(30)",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS supervisor_id UUID REFERENCES employees(id)",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id)",
        "CREATE TABLE IF NOT EXISTS employee_contracts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            contract_type VARCHAR(20) NOT NULL DEFAULT 'Indefinido',
            salary_base DOUBLE PRECISION NOT NULL,
            weekly_hours INTEGER NOT NULL DEFAULT 40,
            ley_karin_signed BOOLEAN NOT NULL DEFAULT false,
            start_date DATE NOT NULL,
            end_date DATE,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS employee_documents (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            doc_type VARCHAR(50) NOT NULL,
            file_name VARCHAR(255) NOT NULL,
            file_url VARCHAR(500),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS employee_attendance_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            timestamp TIMESTAMPTZ NOT NULL,
            entry_type VARCHAR(20) NOT NULL,
            device_id VARCHAR(100),
            location_hash VARCHAR(255),
            source VARCHAR(20) NOT NULL DEFAULT 'api',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS employee_attendance_modifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            attendance_id UUID NOT NULL REFERENCES employee_attendance_logs(id) ON DELETE CASCADE,
            original_value VARCHAR(255) NOT NULL,
            new_value VARCHAR(255) NOT NULL,
            reason TEXT NOT NULL,
            modified_by UUID NOT NULL REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS leave_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            leave_type VARCHAR(30) NOT NULL,
            start_date DATE NOT NULL,
            end_date DATE NOT NULL,
            reason TEXT,
            status VARCHAR(20) NOT NULL DEFAULT 'Pendiente',
            approved_by UUID REFERENCES users(id),
            approved_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS employee_pension_funds (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE UNIQUE,
            pension_fund VARCHAR(20) NOT NULL DEFAULT 'Provida',
            health_system VARCHAR(20) NOT NULL DEFAULT 'Fonasa',
            health_plan_name VARCHAR(255),
            health_fixed_amount DOUBLE PRECISION,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS payrolls (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            month INTEGER NOT NULL,
            year INTEGER NOT NULL,
            salary_base DOUBLE PRECISION NOT NULL DEFAULT 0,
            gratificacion DOUBLE PRECISION NOT NULL DEFAULT 0,
            non_taxable_earnings DOUBLE PRECISION NOT NULL DEFAULT 0,
            taxable_income DOUBLE PRECISION NOT NULL DEFAULT 0,
            afp_discount DOUBLE PRECISION NOT NULL DEFAULT 0,
            health_discount DOUBLE PRECISION NOT NULL DEFAULT 0,
            unemployment_discount DOUBLE PRECISION NOT NULL DEFAULT 0,
            income_tax DOUBLE PRECISION NOT NULL DEFAULT 0,
            other_deductions DOUBLE PRECISION NOT NULL DEFAULT 0,
            net_salary DOUBLE PRECISION NOT NULL DEFAULT 0,
            lre_exported BOOLEAN NOT NULL DEFAULT false,
            previred_exported BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(employee_id, month, year)
        )",
        "CREATE TABLE IF NOT EXISTS employee_geofences (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            lat DOUBLE PRECISION NOT NULL,
            lng DOUBLE PRECISION NOT NULL,
            radius_meters DOUBLE PRECISION NOT NULL DEFAULT 100,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS complaints (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            complainant_name VARCHAR(255),
            complainant_email VARCHAR(255),
            accused_rut VARCHAR(12),
            complaint_type VARCHAR(50) NOT NULL,
            description TEXT NOT NULL,
            status VARCHAR(20) NOT NULL DEFAULT 'Pendiente',
            resolution TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS academic_changelog (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            entity_type VARCHAR(50) NOT NULL,
            entity_id UUID NOT NULL,
            action VARCHAR(20) NOT NULL,
            field_name VARCHAR(100),
            old_value TEXT,
            new_value TEXT,
            changed_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE courses ADD COLUMN IF NOT EXISTS max_students INTEGER DEFAULT 35",
        "ALTER TABLE courses ADD COLUMN IF NOT EXISTS academic_year INTEGER",
        "CREATE TABLE IF NOT EXISTS medical_licenses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            license_type VARCHAR(30) NOT NULL,
            folio VARCHAR(50),
            start_date DATE NOT NULL,
            end_date DATE NOT NULL,
            days INTEGER NOT NULL,
            diagnosis VARCHAR(255),
            status VARCHAR(20) NOT NULL DEFAULT 'Pendiente',
            file_url VARCHAR(500),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS teacher_evaluations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
            evaluator_id UUID REFERENCES users(id),
            evaluation_type VARCHAR(50) NOT NULL,
            score DOUBLE PRECISION,
            observations TEXT,
            period VARCHAR(20),
            year INTEGER NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE employee_contracts ADD COLUMN IF NOT EXISTS digitally_signed BOOLEAN DEFAULT false",
        "ALTER TABLE employee_contracts ADD COLUMN IF NOT EXISTS signed_at TIMESTAMPTZ",
        "ALTER TABLE employee_contracts ADD COLUMN IF NOT EXISTS signature_file_url VARCHAR(500)",
        "CREATE TABLE IF NOT EXISTS family_members (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID REFERENCES prospects(id) ON DELETE CASCADE,
            student_id UUID REFERENCES students(id) ON DELETE CASCADE,
            rut VARCHAR(12) NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            relationship VARCHAR(50) NOT NULL,
            is_enrolled BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS notifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id),
            title VARCHAR(255) NOT NULL,
            body TEXT,
            notification_type VARCHAR(50) NOT NULL DEFAULT 'info',
            read BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, read, created_at DESC)",
        "CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            provider_name VARCHAR(100) NOT NULL,
            api_key_hash VARCHAR(255) NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS license_plans (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            description TEXT,
            price_monthly DECIMAL(10,2) NOT NULL DEFAULT 0,
            price_yearly DECIMAL(10,2) NOT NULL DEFAULT 0,
            featured BOOLEAN NOT NULL DEFAULT false,
            sort_order INT NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT true,
            is_custom BOOLEAN NOT NULL DEFAULT false,
            show_in_portal BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS plan_modules (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            plan_id UUID NOT NULL REFERENCES license_plans(id) ON DELETE CASCADE,
            module_key VARCHAR(50) NOT NULL,
            module_name VARCHAR(100) NOT NULL,
            included BOOLEAN NOT NULL DEFAULT true,
            sub_modules JSONB DEFAULT '[]',
            UNIQUE(plan_id, module_key)
        )",
        "CREATE TABLE IF NOT EXISTS corporation_licenses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID NOT NULL REFERENCES corporations(id) ON DELETE CASCADE,
            plan_id UUID NOT NULL REFERENCES license_plans(id),
            start_date DATE NOT NULL,
            end_date DATE,
            auto_renew BOOLEAN NOT NULL DEFAULT false,
            grace_period_days INT NOT NULL DEFAULT 30,
            status VARCHAR(20) NOT NULL DEFAULT 'active',
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS corporation_module_overrides (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID NOT NULL REFERENCES corporations(id) ON DELETE CASCADE,
            module_key VARCHAR(50) NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT true,
            reason TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS license_payments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_license_id UUID NOT NULL REFERENCES corporation_licenses(id),
            amount DECIMAL(10,2) NOT NULL,
            currency VARCHAR(3) NOT NULL DEFAULT 'CLP',
            payment_method VARCHAR(30) NOT NULL,
            status VARCHAR(20) NOT NULL DEFAULT 'pending',
            transaction_id VARCHAR(100),
            paid_at TIMESTAMPTZ,
            period_start DATE,
            period_end DATE,
            receipt_url TEXT,
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS license_extensions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_license_id UUID NOT NULL REFERENCES corporation_licenses(id) ON DELETE CASCADE,
            days_extended INT NOT NULL,
            reason VARCHAR(255) NOT NULL,
            approved_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS admin_activity_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            admin_id UUID NOT NULL REFERENCES users(id),
            action VARCHAR(50) NOT NULL,
            entity_type VARCHAR(50),
            entity_id UUID,
            details JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS legal_representatives (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID REFERENCES corporations(id) ON DELETE CASCADE,
            school_id UUID REFERENCES schools(id) ON DELETE CASCADE,
            rut VARCHAR(12) NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            phone VARCHAR(20),
            address VARCHAR(500),
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "ALTER TABLE legal_representatives ADD COLUMN IF NOT EXISTS corporation_id UUID REFERENCES corporations(id) ON DELETE CASCADE",
        "ALTER TABLE legal_representatives ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES schools(id) ON DELETE CASCADE",
    ];

    for stmt in statements {
        sqlx::query(stmt).execute(pool).await.unwrap_or_else(|e| {
            tracing::warn!("Schema statement skipped: {e}");
            Default::default()
        });
    }

    tracing::info!("Database schema initialized");
}
