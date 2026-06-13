-- Migration 002: Fix monetary columns to use DECIMAL instead of DOUBLE PRECISION
-- DOUBLE PRECISION causes rounding errors in financial calculations

-- Fees table
ALTER TABLE fees ALTER COLUMN amount TYPE DECIMAL(12,2);
ALTER TABLE fees ALTER COLUMN paid_amount TYPE DECIMAL(12,2);

-- Payments table
ALTER TABLE payments ALTER COLUMN amount TYPE DECIMAL(12,2);

-- Scholarships (percentage but better as DECIMAL for precision)
ALTER TABLE scholarships ALTER COLUMN discount_percentage TYPE DECIMAL(5,2);

-- Payroll tables
ALTER TABLE employees ALTER COLUMN salary_base TYPE DECIMAL(12,2);
ALTER TABLE employee_health ALTER COLUMN health_fixed_amount TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN salary_base TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN gratificacion TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN non_taxable_earnings TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN taxable_income TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN afp_discount TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN health_discount TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN unemployment_discount TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN income_tax TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN other_deductions TYPE DECIMAL(12,2);
ALTER TABLE payrolls ALTER COLUMN net_salary TYPE DECIMAL(12,2);

-- Invoice tables
ALTER TABLE invoices ALTER COLUMN total_fee TYPE DECIMAL(12,2);
ALTER TABLE invoices ALTER COLUMN discount_amount TYPE DECIMAL(12,2);
ALTER TABLE invoices ALTER COLUMN final_amount TYPE DECIMAL(12,2);
ALTER TABLE invoice_items ALTER COLUMN extra_amount TYPE DECIMAL(12,2);
