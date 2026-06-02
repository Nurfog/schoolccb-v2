#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
pub async fn run(pool: &PgPool) {
    // Search path: colegios schema first for school tables, public for shared/CRM tables
    sqlx::query("SET search_path TO colegios, public")
        .execute(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Could not set search_path: {e}");
            Default::default()
        });

    let statements = vec![
        // ========================
        // SHARED / SYSTEM TABLES (public schema)
        // ========================
        "CREATE TABLE IF NOT EXISTS public.users (
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
        "CREATE TABLE IF NOT EXISTS public.corporations (
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
        "CREATE TABLE IF NOT EXISTS public.schools (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID NOT NULL REFERENCES public.corporations(id),
            name VARCHAR(255) NOT NULL,
            address VARCHAR(500),
            phone VARCHAR(20),
            logo_url VARCHAR(500),
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.refresh_tokens (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES public.users(id),
            token_hash VARCHAR(255) NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            revoked BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.password_reset_tokens (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES public.users(id),
            token_hash VARCHAR(255) NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            used BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(50) UNIQUE NOT NULL,
            description TEXT,
            is_system BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.permission_definitions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            module VARCHAR(50) NOT NULL,
            resource VARCHAR(50) NOT NULL,
            label VARCHAR(100) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(module, resource)
        )",
        "CREATE TABLE IF NOT EXISTS public.role_permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            role_id UUID NOT NULL REFERENCES public.roles(id) ON DELETE CASCADE,
            permission_id UUID NOT NULL REFERENCES public.permission_definitions(id) ON DELETE CASCADE,
            can_create BOOLEAN NOT NULL DEFAULT false,
            can_read BOOLEAN NOT NULL DEFAULT false,
            can_update BOOLEAN NOT NULL DEFAULT false,
            can_delete BOOLEAN NOT NULL DEFAULT false,
            UNIQUE(role_id, permission_id)
        )",
        "CREATE TABLE IF NOT EXISTS public.user_roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
            role_id UUID NOT NULL REFERENCES public.roles(id) ON DELETE CASCADE,
            UNIQUE(user_id, role_id)
        )",
        "CREATE TABLE IF NOT EXISTS public.license_plans (
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
        "CREATE TABLE IF NOT EXISTS public.plan_modules (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            plan_id UUID NOT NULL REFERENCES public.license_plans(id) ON DELETE CASCADE,
            module_key VARCHAR(50) NOT NULL,
            module_name VARCHAR(100) NOT NULL,
            included BOOLEAN NOT NULL DEFAULT true,
            sub_modules JSONB DEFAULT '[]',
            UNIQUE(plan_id, module_key)
        )",
        "CREATE TABLE IF NOT EXISTS public.corporation_licenses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID NOT NULL REFERENCES public.corporations(id) ON DELETE CASCADE,
            plan_id UUID NOT NULL REFERENCES public.license_plans(id),
            start_date DATE NOT NULL,
            end_date DATE,
            auto_renew BOOLEAN NOT NULL DEFAULT false,
            grace_period_days INT NOT NULL DEFAULT 30,
            status VARCHAR(20) NOT NULL DEFAULT 'active',
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.corporation_module_overrides (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID NOT NULL REFERENCES public.corporations(id) ON DELETE CASCADE,
            module_key VARCHAR(50) NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT true,
            reason TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.license_payments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_license_id UUID NOT NULL REFERENCES public.corporation_licenses(id),
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
        "CREATE TABLE IF NOT EXISTS public.license_extensions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_license_id UUID NOT NULL REFERENCES public.corporation_licenses(id) ON DELETE CASCADE,
            days_extended INT NOT NULL,
            reason VARCHAR(255) NOT NULL,
            approved_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.admin_activity_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            admin_id UUID NOT NULL REFERENCES public.users(id),
            action VARCHAR(50) NOT NULL,
            entity_type VARCHAR(50),
            entity_id UUID,
            details JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.legal_representatives (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID REFERENCES public.corporations(id) ON DELETE CASCADE,
            school_id UUID REFERENCES public.schools(id) ON DELETE CASCADE,
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
        "CREATE TABLE IF NOT EXISTS public.user_favorites (
            user_id UUID NOT NULL REFERENCES public.users(id),
            module_id VARCHAR(50) NOT NULL,
            PRIMARY KEY (user_id, module_id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            provider_name VARCHAR(100) NOT NULL,
            api_key_hash VARCHAR(255) NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.audit_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            entity_type VARCHAR(50) NOT NULL,
            entity_id UUID NOT NULL,
            action VARCHAR(20) NOT NULL,
            user_id UUID,
            changes JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.event_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            event_type VARCHAR(50) NOT NULL,
            payload JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",

        // ========================
        // CRM / SALES TABLES (public schema)
        // ========================
        "CREATE TABLE IF NOT EXISTS public.pipeline_stages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_final BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.prospects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            rut VARCHAR(12),
            email VARCHAR(255),
            phone VARCHAR(20),
            current_stage_id UUID REFERENCES public.pipeline_stages(id),
            assigned_user_id UUID REFERENCES public.users(id),
            source VARCHAR(50),
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.prospect_activities (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID NOT NULL REFERENCES public.prospects(id) ON DELETE CASCADE,
            activity_type VARCHAR(20) NOT NULL DEFAULT 'note',
            subject VARCHAR(255) NOT NULL,
            description TEXT,
            scheduled_at TIMESTAMPTZ,
            is_completed BOOLEAN NOT NULL DEFAULT false,
            created_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.prospect_documents (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID NOT NULL REFERENCES public.prospects(id) ON DELETE CASCADE,
            file_name VARCHAR(255) NOT NULL,
            s3_url VARCHAR(500),
            doc_type VARCHAR(50) NOT NULL DEFAULT 'other',
            is_verified BOOLEAN NOT NULL DEFAULT false,
            uploaded_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS public.prospect_reminders (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID NOT NULL REFERENCES public.prospects(id) ON DELETE CASCADE,
            reminder_type VARCHAR(30) NOT NULL DEFAULT 'follow_up',
            title VARCHAR(255) NOT NULL,
            description TEXT,
            remind_at TIMESTAMPTZ NOT NULL,
            is_sent BOOLEAN NOT NULL DEFAULT false,
            created_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",

        // ========================
        // SCHOOL TABLES (colegios schema)
        // ========================
        "CREATE TABLE IF NOT EXISTS colegios.students (
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
            school_id UUID REFERENCES public.schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.courses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            subject VARCHAR(255),
            grade_level VARCHAR(20) NOT NULL,
            section VARCHAR(10) NOT NULL,
            teacher_id UUID REFERENCES public.users(id),
            plan VARCHAR(2),
            classroom_id UUID,
            max_students INTEGER DEFAULT 35,
            academic_year INTEGER,
            school_id UUID REFERENCES public.schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.enrollments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            course_id UUID NOT NULL REFERENCES colegios.courses(id),
            year INTEGER NOT NULL,
            active BOOLEAN NOT NULL DEFAULT true,
            school_id UUID REFERENCES public.schools(id),
            UNIQUE(student_id, course_id, year)
        )",
        "CREATE TABLE IF NOT EXISTS colegios.attendance (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            course_id UUID NOT NULL REFERENCES colegios.courses(id),
            date DATE NOT NULL,
            time TIME,
            status VARCHAR(20) NOT NULL DEFAULT 'Presente',
            subject VARCHAR(255) NOT NULL,
            teacher_id UUID NOT NULL REFERENCES public.users(id),
            observation TEXT,
            school_id UUID REFERENCES public.schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(student_id, course_id, date, subject)
        )",
        "CREATE TABLE IF NOT EXISTS colegios.grades (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            subject VARCHAR(255) NOT NULL,
            grade DOUBLE PRECISION NOT NULL,
            grade_type VARCHAR(20) NOT NULL DEFAULT 'Sumativa',
            semester INTEGER NOT NULL DEFAULT 1,
            year INTEGER NOT NULL,
            date DATE NOT NULL,
            teacher_id UUID NOT NULL REFERENCES public.users(id),
            observation TEXT,
            course_subject_id UUID,
            category_id UUID,
            school_id UUID REFERENCES public.schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.subjects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            code VARCHAR(20) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            level VARCHAR(20),
            hours_per_week INTEGER NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.academic_years (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            year INTEGER NOT NULL UNIQUE,
            name VARCHAR(255) NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.academic_periods (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            year INTEGER NOT NULL,
            semester INTEGER NOT NULL DEFAULT 1,
            start_date DATE NOT NULL,
            end_date DATE NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.course_subjects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            course_id UUID NOT NULL REFERENCES colegios.courses(id),
            subject_id UUID NOT NULL REFERENCES colegios.subjects(id),
            teacher_id UUID NOT NULL REFERENCES public.users(id),
            academic_year INTEGER NOT NULL,
            hours_per_week INTEGER NOT NULL DEFAULT 0,
            UNIQUE(course_id, subject_id, academic_year),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.grade_categories (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            course_subject_id UUID NOT NULL REFERENCES colegios.course_subjects(id),
            name VARCHAR(255) NOT NULL,
            weight_percentage DOUBLE PRECISION NOT NULL DEFAULT 100.0,
            evaluation_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.classrooms (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            capacity INTEGER NOT NULL DEFAULT 30,
            location VARCHAR(255),
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.grade_levels (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            code VARCHAR(20) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            plan VARCHAR(20),
            sort_order INTEGER NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.fees (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            description VARCHAR(255) NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            due_date DATE NOT NULL,
            paid BOOLEAN NOT NULL DEFAULT false,
            paid_date DATE,
            paid_amount DOUBLE PRECISION,
            school_id UUID REFERENCES public.schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.payments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            fee_id UUID NOT NULL REFERENCES colegios.fees(id),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            amount DOUBLE PRECISION NOT NULL,
            payment_date DATE NOT NULL DEFAULT CURRENT_DATE,
            payment_method VARCHAR(50) NOT NULL DEFAULT 'Efectivo',
            reference VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.payment_transactions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            fee_id UUID NOT NULL REFERENCES colegios.fees(id),
            token VARCHAR(255) NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            status VARCHAR(20) NOT NULL DEFAULT 'INITIALIZED',
            authorization_code VARCHAR(50),
            payment_type VARCHAR(20),
            gateway_url TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.admission_scholarships (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            school_id UUID NOT NULL,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            discount_type VARCHAR(20) NOT NULL DEFAULT 'percentage',
            discount_value DECIMAL(5,2) NOT NULL,
            max_beneficiaries INT DEFAULT 0,
            current_beneficiaries INT DEFAULT 0,
            requirements JSONB DEFAULT '{}',
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.student_scholarships (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            admission_scholarship_id UUID REFERENCES colegios.admission_scholarships(id),
            name VARCHAR(255) NOT NULL,
            discount_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
            approved BOOLEAN NOT NULL DEFAULT false,
            approved_by UUID REFERENCES public.users(id),
            valid_from DATE NOT NULL,
            valid_until DATE NOT NULL,
            school_id UUID REFERENCES public.schools(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.employees (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            school_id UUID REFERENCES public.schools(id),
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
            supervisor_id UUID,
            user_id UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.employee_contracts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
            contract_type VARCHAR(20) NOT NULL DEFAULT 'Indefinido',
            salary_base DOUBLE PRECISION NOT NULL,
            weekly_hours INTEGER NOT NULL DEFAULT 40,
            ley_karin_signed BOOLEAN NOT NULL DEFAULT false,
            start_date DATE NOT NULL,
            end_date DATE,
            active BOOLEAN NOT NULL DEFAULT true,
            digitally_signed BOOLEAN DEFAULT false,
            signed_at TIMESTAMPTZ,
            signature_file_url VARCHAR(500),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.employee_documents (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
            doc_type VARCHAR(50) NOT NULL,
            file_name VARCHAR(255) NOT NULL,
            file_url VARCHAR(500),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.employee_attendance_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
            timestamp TIMESTAMPTZ NOT NULL,
            entry_type VARCHAR(20) NOT NULL,
            device_id VARCHAR(100),
            location_hash VARCHAR(255),
            source VARCHAR(20) NOT NULL DEFAULT 'api',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.employee_attendance_modifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            attendance_id UUID NOT NULL REFERENCES colegios.employee_attendance_logs(id) ON DELETE CASCADE,
            original_value VARCHAR(255) NOT NULL,
            new_value VARCHAR(255) NOT NULL,
            reason TEXT NOT NULL,
            modified_by UUID NOT NULL REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.leave_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
            leave_type VARCHAR(30) NOT NULL,
            start_date DATE NOT NULL,
            end_date DATE NOT NULL,
            reason TEXT,
            status VARCHAR(20) NOT NULL DEFAULT 'Pendiente',
            approved_by UUID REFERENCES public.users(id),
            approved_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.employee_pension_funds (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE UNIQUE,
            pension_fund VARCHAR(20) NOT NULL DEFAULT 'Provida',
            health_system VARCHAR(20) NOT NULL DEFAULT 'Fonasa',
            health_plan_name VARCHAR(255),
            health_fixed_amount DOUBLE PRECISION,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.payrolls (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
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
        "CREATE TABLE IF NOT EXISTS colegios.employee_geofences (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
            lat DOUBLE PRECISION NOT NULL,
            lng DOUBLE PRECISION NOT NULL,
            radius_meters DOUBLE PRECISION NOT NULL DEFAULT 100,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.medical_licenses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
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
        "CREATE TABLE IF NOT EXISTS colegios.teacher_evaluations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            employee_id UUID NOT NULL REFERENCES colegios.employees(id) ON DELETE CASCADE,
            evaluator_id UUID REFERENCES public.users(id),
            evaluation_type VARCHAR(50) NOT NULL,
            score DOUBLE PRECISION,
            observations TEXT,
            period VARCHAR(20),
            year INTEGER NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.holidays (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            school_id UUID,
            date DATE NOT NULL,
            name VARCHAR(255) NOT NULL,
            holiday_type VARCHAR(20) NOT NULL DEFAULT 'legal',
            is_recurring BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.complementary_subjects (
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
        "CREATE TABLE IF NOT EXISTS colegios.complementary_subject_enrollments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            subject_id UUID NOT NULL REFERENCES colegios.complementary_subjects(id) ON DELETE CASCADE,
            student_id UUID NOT NULL,
            enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.student_annotations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id) ON DELETE CASCADE,
            annotation_type VARCHAR(30) NOT NULL DEFAULT 'observacion',
            description TEXT NOT NULL,
            severity VARCHAR(20) NOT NULL DEFAULT 'leve',
            created_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.enrollment_contracts (
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
        "CREATE TABLE IF NOT EXISTS colegios.guardian_relationships (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            guardian_user_id UUID NOT NULL REFERENCES public.users(id),
            relationship VARCHAR(50) NOT NULL,
            authorized_pickup BOOLEAN NOT NULL DEFAULT false,
            receives_notifications BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(student_id, guardian_user_id)
        )",
        "CREATE TABLE IF NOT EXISTS colegios.family_members (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prospect_id UUID REFERENCES public.prospects(id) ON DELETE CASCADE,
            student_id UUID REFERENCES colegios.students(id) ON DELETE CASCADE,
            rut VARCHAR(12) NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            relationship VARCHAR(50) NOT NULL,
            is_enrolled BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.interview_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES colegios.students(id),
            teacher_id UUID NOT NULL REFERENCES public.users(id),
            date DATE NOT NULL DEFAULT CURRENT_DATE,
            reason TEXT NOT NULL,
            notes TEXT NOT NULL,
            follow_up TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.messages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            sender_id UUID NOT NULL REFERENCES public.users(id),
            receiver_id UUID NOT NULL REFERENCES public.users(id),
            subject VARCHAR(255) NOT NULL,
            body TEXT NOT NULL,
            read BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.notifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES public.users(id),
            title VARCHAR(255) NOT NULL,
            body TEXT,
            notification_type VARCHAR(50) NOT NULL DEFAULT 'info',
            read BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.school_config (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            corporation_id UUID REFERENCES public.corporations(id),
            school_name VARCHAR(255) NOT NULL DEFAULT '',
            school_logo_url VARCHAR(500) NOT NULL DEFAULT '',
            primary_color VARCHAR(7) NOT NULL DEFAULT '#1A2B3C',
            secondary_color VARCHAR(7) NOT NULL DEFAULT '#243B4F',
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.user_preferences (
            user_id UUID PRIMARY KEY REFERENCES public.users(id),
            show_module_manager BOOLEAN NOT NULL DEFAULT true,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.subject_hours (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            subject_id UUID NOT NULL REFERENCES colegios.subjects(id),
            level VARCHAR(20) NOT NULL,
            hours_per_week INTEGER NOT NULL DEFAULT 0,
            UNIQUE(subject_id, level)
        )",
        "CREATE TABLE IF NOT EXISTS colegios.custom_field_definitions (
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
        "CREATE TABLE IF NOT EXISTS colegios.custom_field_values (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            field_definition_id UUID NOT NULL REFERENCES colegios.custom_field_definitions(id) ON DELETE CASCADE,
            entity_id UUID NOT NULL,
            value TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(field_definition_id, entity_id)
        )",
        "CREATE TABLE IF NOT EXISTS colegios.academic_changelog (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            entity_type VARCHAR(50) NOT NULL,
            entity_id UUID NOT NULL,
            action VARCHAR(20) NOT NULL,
            field_name VARCHAR(100),
            old_value TEXT,
            new_value TEXT,
            changed_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.teacher_schedules (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            teacher_id UUID NOT NULL,
            day_of_week INT NOT NULL DEFAULT 0,
            start_time TIME NOT NULL,
            end_time TIME NOT NULL,
            schedule_type VARCHAR(20) NOT NULL DEFAULT 'class',
            subject_id UUID,
            course_id UUID,
            room VARCHAR(100),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.substitute_schedule (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            original_teacher_id UUID NOT NULL,
            substitute_teacher_id UUID NOT NULL,
            schedule_id UUID REFERENCES colegios.teacher_schedules(id) ON DELETE CASCADE,
            start_date DATE NOT NULL,
            end_date DATE NOT NULL,
            reason TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.teacher_contract_hours (
            teacher_id UUID NOT NULL,
            academic_year_id INT NOT NULL DEFAULT 0,
            total_hours REAL NOT NULL DEFAULT 0,
            class_hours REAL NOT NULL DEFAULT 0,
            admin_hours REAL NOT NULL DEFAULT 0,
            extra_hours REAL NOT NULL DEFAULT 0,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (teacher_id, academic_year_id)
        )",
        "CREATE TABLE IF NOT EXISTS colegios.extra_duties (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            teacher_id UUID NOT NULL,
            duty_type VARCHAR(50) NOT NULL DEFAULT 'jefatura',
            description TEXT,
            extra_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            is_paid BOOLEAN NOT NULL DEFAULT false,
            period VARCHAR(20),
            approved_by UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.interview_process (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            candidate_name VARCHAR(255) NOT NULL,
            candidate_email VARCHAR(255),
            candidate_phone VARCHAR(20),
            position VARCHAR(255) NOT NULL,
            interviewer_id UUID,
            interview_date TIMESTAMPTZ,
            result VARCHAR(20) DEFAULT 'pending',
            notes TEXT,
            status VARCHAR(20) NOT NULL DEFAULT 'pendiente',
            school_id UUID,
            created_by UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.agenda_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            description TEXT,
            event_date DATE NOT NULL,
            event_type VARCHAR(20) NOT NULL DEFAULT 'Evento',
            created_by UUID REFERENCES public.users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS colegios.complaints (
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

        // ========================
        // INDEXES (colegios schema)
        // ========================
        "CREATE INDEX IF NOT EXISTS idx_attendance_date ON colegios.attendance(date)",
        "CREATE INDEX IF NOT EXISTS idx_attendance_student ON colegios.attendance(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_grades_student ON colegios.grades(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_grades_subject ON colegios.grades(subject)",
        "CREATE INDEX IF NOT EXISTS idx_grades_course_subject ON colegios.grades(course_subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_guardian_relationships_student ON colegios.guardian_relationships(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_guardian_relationships_guardian ON colegios.guardian_relationships(guardian_user_id)",
        "CREATE INDEX IF NOT EXISTS idx_subjects_active ON colegios.subjects(active)",
        "CREATE INDEX IF NOT EXISTS idx_academic_periods_year ON colegios.academic_periods(year)",
        "CREATE INDEX IF NOT EXISTS idx_course_subjects_course ON colegios.course_subjects(course_id)",
        "CREATE INDEX IF NOT EXISTS idx_course_subjects_teacher ON colegios.course_subjects(teacher_id)",
        "CREATE INDEX IF NOT EXISTS idx_grade_categories_subject ON colegios.grade_categories(course_subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_messages_receiver ON colegios.messages(receiver_id)",
        "CREATE INDEX IF NOT EXISTS idx_messages_sender ON colegios.messages(sender_id)",
        "CREATE INDEX IF NOT EXISTS idx_interview_logs_student ON colegios.interview_logs(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_fees_student ON colegios.fees(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_payments_fee ON colegios.payments(fee_id)",
        "CREATE INDEX IF NOT EXISTS idx_admission_scholarships_school ON colegios.admission_scholarships(school_id)",
        "CREATE INDEX IF NOT EXISTS idx_student_scholarships_student ON colegios.student_scholarships(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_student_scholarships_admission ON colegios.student_scholarships(admission_scholarship_id)",
        "CREATE INDEX IF NOT EXISTS idx_student_annotations_student ON colegios.student_annotations(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_holidays_date ON colegios.holidays(date)",
        "CREATE INDEX IF NOT EXISTS idx_complementary_subjects_course ON colegios.complementary_subjects(course_id)",
        "CREATE INDEX IF NOT EXISTS idx_complementary_enrollments_subject ON colegios.complementary_subject_enrollments(subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_notifications_user ON colegios.notifications(user_id, read, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_subject_hours_subject ON colegios.subject_hours(subject_id)",
        "CREATE INDEX IF NOT EXISTS idx_custom_fields_entity ON colegios.custom_field_definitions(entity_type)",
        "CREATE INDEX IF NOT EXISTS idx_custom_field_values_entity ON colegios.custom_field_values(entity_id)",

        // ========================
        // BACKWARD-COMPAT ALTER TABLE (public schema) — ensure old public tables
        // have all columns for data migration to colegios schema
        // ========================
        "ALTER TABLE public.users ADD COLUMN IF NOT EXISTS corporation_id UUID REFERENCES public.corporations(id)",
        "ALTER TABLE public.users ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.users ADD COLUMN IF NOT EXISTS admin_type VARCHAR(20)",
        "ALTER TABLE public.users ADD COLUMN IF NOT EXISTS managed_school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.corporations ADD COLUMN IF NOT EXISTS legal_representative_name VARCHAR(255)",
        "ALTER TABLE public.corporations ADD COLUMN IF NOT EXISTS legal_representative_rut VARCHAR(12)",
        "ALTER TABLE public.corporations ADD COLUMN IF NOT EXISTS legal_representative_email VARCHAR(255)",
        "ALTER TABLE public.license_plans ADD COLUMN IF NOT EXISTS is_custom BOOLEAN NOT NULL DEFAULT false",
        "ALTER TABLE public.license_plans ADD COLUMN IF NOT EXISTS show_in_portal BOOLEAN NOT NULL DEFAULT true",
        "ALTER TABLE public.plan_modules ADD COLUMN IF NOT EXISTS sub_modules JSONB DEFAULT '[]'",
        "ALTER TABLE public.school_config ADD COLUMN IF NOT EXISTS corporation_id UUID REFERENCES public.corporations(id)",
        "ALTER TABLE public.legal_representatives ADD COLUMN IF NOT EXISTS corporation_id UUID REFERENCES public.corporations(id) ON DELETE CASCADE",
        "ALTER TABLE public.legal_representatives ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id) ON DELETE CASCADE",
        "ALTER TABLE public.students ADD COLUMN IF NOT EXISTS diseases TEXT",
        "ALTER TABLE public.students ADD COLUMN IF NOT EXISTS allergies TEXT",
        "ALTER TABLE public.students ADD COLUMN IF NOT EXISTS emergency_contact_name VARCHAR(255)",
        "ALTER TABLE public.students ADD COLUMN IF NOT EXISTS emergency_contact_phone VARCHAR(20)",
        "ALTER TABLE public.students ADD COLUMN IF NOT EXISTS emergency_contact_relation VARCHAR(100)",
        "ALTER TABLE public.students ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.courses ADD COLUMN IF NOT EXISTS plan VARCHAR(2)",
        "ALTER TABLE public.courses ADD COLUMN IF NOT EXISTS classroom_id UUID",
        "ALTER TABLE public.courses ADD COLUMN IF NOT EXISTS max_students INTEGER DEFAULT 35",
        "ALTER TABLE public.courses ADD COLUMN IF NOT EXISTS academic_year INTEGER",
        "ALTER TABLE public.courses ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.courses ALTER COLUMN subject DROP NOT NULL",
        "ALTER TABLE public.enrollments ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.fees ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.admission_scholarships ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.student_scholarships ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.grades ADD COLUMN IF NOT EXISTS course_subject_id UUID",
        "ALTER TABLE public.grades ADD COLUMN IF NOT EXISTS category_id UUID",
        "ALTER TABLE public.grades ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.attendance ADD COLUMN IF NOT EXISTS school_id UUID REFERENCES public.schools(id)",
        "ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS category VARCHAR(30)",
        "ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS supervisor_id UUID",
        "ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES public.users(id)",
        "ALTER TABLE public.employee_contracts ADD COLUMN IF NOT EXISTS digitally_signed BOOLEAN DEFAULT false",
        "ALTER TABLE public.employee_contracts ADD COLUMN IF NOT EXISTS signed_at TIMESTAMPTZ",
        "ALTER TABLE public.employee_contracts ADD COLUMN IF NOT EXISTS signature_file_url VARCHAR(500)",

        // ========================
        // INDEXES (public schema)
        // ========================
        "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON public.refresh_tokens(token_hash)",
        "CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_hash ON public.password_reset_tokens(token_hash)",
        "CREATE INDEX IF NOT EXISTS idx_prospects_stage ON public.prospects(current_stage_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospects_assigned ON public.prospects(assigned_user_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospect_activities_prospect ON public.prospect_activities(prospect_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospect_documents_prospect ON public.prospect_documents(prospect_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospect_reminders_prospect ON public.prospect_reminders(prospect_id)",
        "CREATE INDEX IF NOT EXISTS idx_prospect_reminders_unsent ON public.prospect_reminders(remind_at) WHERE is_sent = false",
        "CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON public.audit_log(entity_type, entity_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_log_created ON public.audit_log(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_event_log_type ON public.event_log(event_type)",
        "CREATE INDEX IF NOT EXISTS idx_event_log_created ON public.event_log(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_payment_transactions_token ON colegios.payment_transactions(token)",
        "CREATE INDEX IF NOT EXISTS idx_payment_transactions_fee ON colegios.payment_transactions(fee_id)",
    ];

    for stmt in statements {
        sqlx::query(stmt).execute(pool).await.unwrap_or_else(|e| {
            tracing::warn!("Schema statement skipped: {e}");
            Default::default()
        });
    }

    migrate_school_data(pool).await;

    tracing::info!("Database schema initialized");
}

async fn migrate_school_data(pool: &PgPool) {
    let school_tables = [
        "students", "courses", "enrollments", "attendance", "grades",
        "subjects", "academic_years", "academic_periods", "course_subjects", "grade_categories",
        "classrooms", "grade_levels",
        "fees", "payments", "payment_transactions", "admission_scholarships", "student_scholarships",
        "employees", "employee_contracts", "employee_documents",
        "employee_attendance_logs", "employee_attendance_modifications",
        "leave_requests", "employee_pension_funds", "payrolls",
        "employee_geofences", "medical_licenses", "teacher_evaluations",
        "holidays", "complementary_subjects", "complementary_subject_enrollments",
        "student_annotations", "enrollment_contracts",
        "guardian_relationships", "family_members", "interview_logs",
        "messages", "notifications", "school_config", "user_preferences",
        "subject_hours", "custom_field_definitions", "custom_field_values",
        "academic_changelog", "agenda_events", "complaints",
        "teacher_schedules", "substitute_schedule", "teacher_contract_hours",
        "extra_duties", "interview_process",
    ];

    for table in school_tables {
        // Check if source has data and target is empty
        let src_count: Result<(i64,), _> = sqlx::query_as(
            &format!("SELECT COUNT(*) FROM public.{table}")
        ).fetch_one(pool).await;

        let dst_count: Result<(i64,), _> = sqlx::query_as(
            &format!("SELECT COUNT(*) FROM colegios.{table}")
        ).fetch_one(pool).await;

        match (src_count, dst_count) {
            (Ok((src,)), Ok((0,))) if src > 0 => {
                tracing::info!("Migrating {src} rows from public.{table} to colegios.{table}");
                let result = sqlx::query(
                    &format!("INSERT INTO colegios.{table} SELECT * FROM public.{table} ON CONFLICT DO NOTHING")
                ).execute(pool).await;
                if let Err(e) = result {
                    tracing::warn!("Could not migrate public.{table}: {e}");
                }
            }
            (Ok((_,)), Ok((dst,))) if dst > 0 => {
                tracing::debug!("colegios.{table} already has {dst} rows, skipping migration");
            }
            (Err(e), _) => {
                tracing::debug!("Cannot migrate {table} (source may not exist): {e}");
            }
            _ => {}
        }
    }
}
