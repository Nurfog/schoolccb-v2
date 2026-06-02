-- Create the colegios schema for school-specific data (separate from CRM/sales data in public)
CREATE SCHEMA IF NOT EXISTS colegios;

-- Set search_path: colegios first for school tables, public for shared/CRM tables
ALTER DATABASE schoolccb SET search_path TO colegios, public;

-- Grant usage
GRANT USAGE ON SCHEMA colegios TO public;
GRANT ALL ON ALL TABLES IN SCHEMA colegios TO schoolccb;
GRANT ALL ON ALL SEQUENCES IN SCHEMA colegios TO schoolccb;
