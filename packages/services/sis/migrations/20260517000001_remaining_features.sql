-- Calendario Académico
CREATE TABLE IF NOT EXISTS academic_calendar (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    event_type VARCHAR(30) NOT NULL DEFAULT 'event',
    event_date DATE NOT NULL,
    start_time TIME,
    end_time TIME,
    all_day BOOLEAN NOT NULL DEFAULT false,
    color VARCHAR(20),
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS holidays (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id UUID,
    date DATE NOT NULL,
    name VARCHAR(255) NOT NULL,
    holiday_type VARCHAR(20) NOT NULL DEFAULT 'legal',
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS exam_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id UUID NOT NULL,
    course_id UUID,
    subject_id UUID,
    title VARCHAR(255) NOT NULL,
    exam_date DATE NOT NULL,
    start_time TIME,
    end_time TIME,
    responsible_teacher_id UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Reuniones de Apoderados
CREATE TABLE IF NOT EXISTS parent_meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id UUID NOT NULL,
    teacher_id UUID NOT NULL,
    guardian_user_id UUID,
    student_id UUID,
    scheduled_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME,
    meeting_type VARCHAR(20) NOT NULL DEFAULT 'individual',
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled',
    location VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS general_meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id UUID NOT NULL,
    course_id UUID,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    meeting_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME,
    location VARCHAR(255),
    agenda JSONB DEFAULT '[]',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_minutes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    content TEXT NOT NULL,
    attachments JSONB DEFAULT '[]',
    sent_by_email BOOLEAN NOT NULL DEFAULT false,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Horarios Docentes
CREATE TABLE IF NOT EXISTS teacher_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    teacher_id UUID NOT NULL,
    day_of_week INT NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    schedule_type VARCHAR(20) NOT NULL DEFAULT 'class',
    subject_id UUID,
    course_id UUID,
    room VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS substitute_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_teacher_id UUID NOT NULL,
    substitute_teacher_id UUID NOT NULL,
    schedule_date DATE NOT NULL,
    reason VARCHAR(255),
    approved_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS teacher_contract_hours (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    teacher_id UUID NOT NULL,
    academic_year_id UUID,
    total_hours INT NOT NULL DEFAULT 0,
    class_hours INT NOT NULL DEFAULT 0,
    admin_hours INT NOT NULL DEFAULT 0,
    extra_hours INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(teacher_id, academic_year_id)
);

CREATE TABLE IF NOT EXISTS extra_duties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    teacher_id UUID NOT NULL,
    duty_type VARCHAR(50) NOT NULL,
    description TEXT,
    extra_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    period VARCHAR(20),
    is_paid BOOLEAN NOT NULL DEFAULT false,
    approved_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cursos Complementarios
CREATE TABLE IF NOT EXISTS complementary_subjects (
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
);

CREATE INDEX IF NOT EXISTS idx_academic_calendar_school ON academic_calendar(school_id, event_date);
CREATE INDEX IF NOT EXISTS idx_exam_schedule_course ON exam_schedule(school_id, exam_date);
CREATE INDEX IF NOT EXISTS idx_parent_meetings_teacher ON parent_meetings(teacher_id, scheduled_date);
CREATE INDEX IF NOT EXISTS idx_teacher_schedules_teacher ON teacher_schedules(teacher_id);
CREATE INDEX IF NOT EXISTS idx_extra_duties_teacher ON extra_duties(teacher_id);
CREATE INDEX IF NOT EXISTS idx_complementary_subjects_course ON complementary_subjects(course_id);
