-- Portal Apoderado / Alumno: certificados, citas, mensajes

CREATE TABLE IF NOT EXISTS portal_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certificate_type VARCHAR(50) NOT NULL,
    student_id UUID NOT NULL,
    requested_by UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    file_url TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS support_appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID,
    requested_by UUID NOT NULL,
    appointment_type VARCHAR(50) NOT NULL,
    reason TEXT,
    preferred_date DATE,
    preferred_time TIME,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    assigned_to UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS parent_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID NOT NULL,
    teacher_id UUID NOT NULL,
    student_id UUID,
    subject VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS teacher_available_slots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    teacher_id UUID NOT NULL,
    day_of_week INT NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    is_booked BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Admission: becas y contratos de matrícula
CREATE TABLE IF NOT EXISTS scholarships (
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
);

CREATE TABLE IF NOT EXISTS enrollment_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL,
    school_id UUID NOT NULL,
    grade_level VARCHAR(50) NOT NULL,
    guardian_user_id UUID,
    scholarship_id UUID REFERENCES scholarships(id),
    annexes JSONB DEFAULT '[]',
    total_fee DECIMAL(12,2) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    final_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    payment_plan VARCHAR(20) DEFAULT 'monthly',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    signed_at TIMESTAMPTZ,
    enrolled_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_portal_certificates_user ON portal_certificates(requested_by);
CREATE INDEX IF NOT EXISTS idx_support_appointments_user ON support_appointments(requested_by);
CREATE INDEX IF NOT EXISTS idx_scholarships_school ON scholarships(school_id);
CREATE INDEX IF NOT EXISTS idx_enrollment_contracts_student ON enrollment_contracts(student_id);
