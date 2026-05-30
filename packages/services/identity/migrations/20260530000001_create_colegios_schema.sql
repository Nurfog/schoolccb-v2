-- Create the colegios schema for school-specific data (separate from CRM/sales data in public)
CREATE SCHEMA IF NOT EXISTS colegios;

-- Set search_path at database level so all connections automatically include the schema
ALTER DATABASE schoolccb SET search_path TO public, colegios;

-- Grant usage
GRANT USAGE ON SCHEMA colegios TO public;
GRANT ALL ON ALL TABLES IN SCHEMA colegios TO schoolccb;
GRANT ALL ON ALL SEQUENCES IN SCHEMA colegios TO schoolccb;
