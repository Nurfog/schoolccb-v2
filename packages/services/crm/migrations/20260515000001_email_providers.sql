-- Email Providers por Corporación/Colegio
-- Cada corporación o colegio puede configurar su propio proveedor SMTP

CREATE TABLE IF NOT EXISTS email_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_id UUID,
    school_id UUID,
    provider_type VARCHAR(20) NOT NULL DEFAULT 'smtp',
    smtp_host VARCHAR(255) NOT NULL,
    smtp_port INT NOT NULL DEFAULT 587,
    smtp_username VARCHAR(255),
    smtp_password TEXT,
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    reply_to VARCHAR(255),
    max_daily_sends INT NOT NULL DEFAULT 500,
    sent_today INT NOT NULL DEFAULT 0,
    last_sent_date DATE,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(corporation_id, school_id)
);

-- Cola de envíos masivos
CREATE TABLE IF NOT EXISTS email_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID REFERENCES email_providers(id),
    corporation_id UUID,
    school_id UUID,
    sender_email VARCHAR(255) NOT NULL,
    sender_name VARCHAR(255),
    recipient_type VARCHAR(20) NOT NULL DEFAULT 'to',
    subject VARCHAR(512) NOT NULL,
    body TEXT NOT NULL,
    body_type VARCHAR(10) NOT NULL DEFAULT 'text',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    priority INT NOT NULL DEFAULT 0,
    total_recipients INT NOT NULL DEFAULT 0,
    sent_count INT NOT NULL DEFAULT 0,
    failed_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    batch_id UUID,
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_queue_status ON email_queue(status);
CREATE INDEX IF NOT EXISTS idx_email_queue_priority ON email_queue(priority, created_at);
CREATE INDEX IF NOT EXISTS idx_email_providers_corp ON email_providers(corporation_id, school_id);
