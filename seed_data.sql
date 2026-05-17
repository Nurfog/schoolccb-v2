-- =============================================================
-- SEED: Datos ficticios para revisión del sistema SchoolCBB
-- =============================================================
-- Contraseña para todos los usuarios: test123
-- Hash Argon2id: $argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A
-- =============================================================

DO $$
DECLARE
    v_corp_id UUID;
    v_school_pred UUID;
    v_school_san_andres UUID;
    v_school_los_andes UUID;
    v_school_liceo UUID;
    v_password_hash TEXT := '$argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A';
BEGIN

-- Obtener corporación existente
SELECT id INTO v_corp_id FROM corporations WHERE name = 'Corporación Educativa';

-- =============================================================
-- COLEGIOS
-- =============================================================
INSERT INTO schools (id, corporation_id, name, address, phone, active)
SELECT gen_random_uuid(), v_corp_id, 'Colegio San Andrés', 'Av. Providencia 1500, Santiago', '+56 2 2123 4567', true
WHERE NOT EXISTS (SELECT 1 FROM schools WHERE name = 'Colegio San Andrés')
RETURNING id INTO v_school_san_andres;

INSERT INTO schools (id, corporation_id, name, address, phone, active)
SELECT gen_random_uuid(), v_corp_id, 'Colegio Los Andes', 'Calle Comercio 450, Rancagua', '+56 72 2123 456', true
WHERE NOT EXISTS (SELECT 1 FROM schools WHERE name = 'Colegio Los Andes')
RETURNING id INTO v_school_los_andes;

INSERT INTO schools (id, corporation_id, name, address, phone, active)
SELECT gen_random_uuid(), v_corp_id, 'Liceo Bicentenario Norte', 'Av. Américo Vespucio 2100, Santiago', '+56 2 2987 6543', true
WHERE NOT EXISTS (SELECT 1 FROM schools WHERE name = 'Liceo Bicentenario Norte')
RETURNING id INTO v_school_liceo;

SELECT id INTO v_school_pred FROM schools WHERE name = 'Colegio Predeterminado';

RAISE NOTICE 'Corporación ID: %', v_corp_id;
RAISE NOTICE 'Colegios: Pred=% SanAndres=% LosAndes=% Liceo=%', v_school_pred, v_school_san_andres, v_school_los_andes, v_school_liceo;

-- =============================================================
-- USUARIOS DOCENTES Y ADMINISTRATIVOS
-- =============================================================
INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '12.345.678-9', 'María González Rojas', 'maria.gonzalez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'maria.gonzalez@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '13.456.789-0', 'Carlos Muñoz Soto', 'carlos.munoz@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carlos.munoz@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '14.567.890-1', 'Ana María Torres Pérez', 'ana.torres@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ana.torres@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '15.678.901-2', 'Pedro Martínez Vega', 'pedro.martinez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'pedro.martinez@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '16.789.012-3', 'Laura Fernández Díaz', 'laura.fernandez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'laura.fernandez@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '17.890.123-4', 'José Ramírez López', 'jose.ramirez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'jose.ramirez@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '18.901.234-5', 'Patricia Castillo Silva', 'patricia.castillo@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'patricia.castillo@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '19.012.345-6', 'Roberto Vega Morales', 'roberto.vega@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'roberto.vega@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '20.123.456-7', 'Carolina Rivas Contreras', 'carolina.rivas@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carolina.rivas@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '21.234.567-8', 'Felipe Herrera Cárdenas', 'felipe.herrera@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'felipe.herrera@colegio.cl');

-- Director y UTP
INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '22.345.678-9', 'Daniela Soto Pizarro', 'daniela.soto@colegio.cl', v_password_hash, 'Director', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'daniela.soto@colegio.cl');

INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
SELECT gen_random_uuid(), '23.456.789-0', 'Andrés Núñez Campos', 'andres.nunez@colegio.cl', v_password_hash, 'UTP', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'andres.nunez@colegio.cl');

RAISE NOTICE 'Usuarios creados correctamente';

-- =============================================================
-- ESTUDIANTES
-- =============================================================
INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '24.567.890-1', 'Mateo', 'González Muñoz', 'mateo.gonzalez@correo.cl', '+56 9 1234 5678', '1° Básico', 'A', '1', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '24.567.890-1');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '25.678.901-2', 'Valentina', 'Torres Rivas', 'valentina.torres@correo.cl', '+56 9 2345 6789', '1° Básico', 'A', '1', 'AL', '1', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '25.678.901-2');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '26.789.012-3', 'Benjamín', 'Martínez Soto', 'benjamin.martinez@correo.cl', '+56 9 3456 7890', '2° Básico', 'A', '2', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '26.789.012-3');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '27.890.123-4', 'Isabella', 'Fernández Vega', 'isabella.fernandez@correo.cl', '+56 9 4567 8901', '2° Básico', 'A', '2', 'AL', '0', 'T', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '27.890.123-4');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '28.901.234-5', 'Santiago', 'López Castillo', 'santiago.lopez@correo.cl', '+56 9 5678 9012', '3° Básico', 'B', '3', 'AL', '2', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '28.901.234-5');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '29.012.345-6', 'Emilia', 'Ramírez Herrera', 'emilia.ramirez@correo.cl', '+56 9 6789 0123', '3° Básico', 'B', '3', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '29.012.345-6');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '30.123.456-7', 'Lucas', 'Pérez Díaz', 'lucas.perez@correo.cl', '+56 9 7890 1234', '4° Básico', 'A', '4', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '30.123.456-7');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '31.234.567-8', 'Sofía', 'Morales Rojas', 'sofia.morales@correo.cl', '+56 9 8901 2345', '4° Básico', 'A', '4', 'AL', '1', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '31.234.567-8');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '32.345.678-9', 'Gabriel', 'Araya Contreras', 'gabriel.araya@correo.cl', '+56 9 9012 3456', '5° Básico', 'C', '5', 'AL', '0', 'P', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '32.345.678-9');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '33.456.789-0', 'Martina', 'Castro Pereira', 'martina.castro@correo.cl', '+56 9 0123 4567', '5° Básico', 'C', '5', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '33.456.789-0');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '34.567.890-1', 'Joaquín', 'Valenzuela Rivas', 'joaquin.valenzuela@correo.cl', '+56 9 1122 3344', '6° Básico', 'A', '6', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '34.567.890-1');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '35.678.901-2', 'Florencia', 'Navarrete Toledo', 'florencia.navarrete@correo.cl', '+56 9 2233 4455', '6° Básico', 'A', '6', 'AL', '1', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '35.678.901-2');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '36.789.012-3', 'Agustín', 'Sandoval Díaz', 'agustin.sandoval@correo.cl', '+56 9 3344 5566', '7° Básico', 'A', '7', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '36.789.012-3');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '37.890.123-4', 'Josefa', 'Gutiérrez Bravo', 'josefa.gutierrez@correo.cl', '+56 9 4455 6677', '7° Básico', 'B', '7', 'RE', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '37.890.123-4');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '38.901.234-5', 'Maximiliano', 'Fuentes Sepúlveda', 'maximiliano.fuentes@correo.cl', '+56 9 5566 7788', '8° Básico', 'A', '8', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '38.901.234-5');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '39.012.345-6', 'Antonia', 'Vargas Medina', 'antonia.vargas@correo.cl', '+56 9 6677 8899', '8° Básico', 'A', '8', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '39.012.345-6');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '40.123.456-7', 'Vicente', 'Orellana Paredes', 'vicente.orellana@correo.cl', '+56 9 7788 9900', 'I° Medio', 'A', '9', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '40.123.456-7');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '41.234.567-8', 'Catalina', 'Molina Campos', 'catalina.molina@correo.cl', '+56 9 8899 0011', 'I° Medio', 'B', '9', 'AL', '2', 'T', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '41.234.567-8');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '42.345.678-9', 'Nicolás', 'Cruz Garrido', 'nicolas.cruz@correo.cl', '+56 9 9900 1122', 'II° Medio', 'A', '10', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '42.345.678-9');

INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
SELECT gen_random_uuid(), '43.456.789-0', 'Amanda', 'Peña Vásquez', 'amanda.pena@correo.cl', '+56 9 0011 2233', 'II° Medio', 'A', '10', 'AL', '0', 'N', true, v_school_pred
WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '43.456.789-0');

RAISE NOTICE 'Estudiantes creados correctamente';

-- =============================================================
-- AÑO ACADÉMICO 2026
-- =============================================================
INSERT INTO academic_years (id, year, name, is_active)
SELECT gen_random_uuid(), 2026, 'Año Académico 2026', true
WHERE NOT EXISTS (SELECT 1 FROM academic_years WHERE year = 2026);

-- =============================================================
-- CURSOS
-- =============================================================
INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '1° Básico A', 'Lenguaje y Comunicación', '1° Básico', 'A', (SELECT id FROM users WHERE email = 'maria.gonzalez@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '1° Básico A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '2° Básico A', 'Matemática', '2° Básico', 'A', (SELECT id FROM users WHERE email = 'carlos.munoz@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '2° Básico A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '3° Básico B', 'Ciencias Naturales', '3° Básico', 'B', (SELECT id FROM users WHERE email = 'ana.torres@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '3° Básico B' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '4° Básico A', 'Historia, Geografía y Cs. Sociales', '4° Básico', 'A', (SELECT id FROM users WHERE email = 'pedro.martinez@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '4° Básico A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '5° Básico C', 'Lenguaje y Comunicación', '5° Básico', 'C', (SELECT id FROM users WHERE email = 'laura.fernandez@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '5° Básico C' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '6° Básico A', 'Matemática', '6° Básico', 'A', (SELECT id FROM users WHERE email = 'maria.gonzalez@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '6° Básico A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '7° Básico A', 'Lengua y Literatura', '7° Básico', 'A', (SELECT id FROM users WHERE email = 'carlos.munoz@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '7° Básico A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), '8° Básico A', 'Matemática', '8° Básico', 'A', (SELECT id FROM users WHERE email = 'ana.torres@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '8° Básico A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), 'I° Medio A', 'Lengua y Literatura', 'I° Medio', 'A', (SELECT id FROM users WHERE email = 'pedro.martinez@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = 'I° Medio A' AND school_id = v_school_pred);

INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, school_id, academic_year)
SELECT gen_random_uuid(), 'II° Medio A', 'Matemática', 'II° Medio', 'A', (SELECT id FROM users WHERE email = 'laura.fernandez@colegio.cl'), v_school_pred, 2026
WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = 'II° Medio A' AND school_id = v_school_pred);

RAISE NOTICE 'Cursos creados correctamente';

-- =============================================================
-- MATRÍCULAS (enrollments)
-- =============================================================
INSERT INTO enrollments (id, student_id, course_id, year, active, school_id)
SELECT gen_random_uuid(), s.id, c.id, 2026, true, v_school_pred
FROM students s, courses c
WHERE s.school_id = v_school_pred AND c.school_id = v_school_pred
  AND s.grade_level = c.grade_level AND s.section = c.section
  AND NOT EXISTS (
      SELECT 1 FROM enrollments e
      WHERE e.student_id = s.id AND e.course_id = c.id AND e.year = 2026
  );

RAISE NOTICE 'Matrículas creadas correctamente';

RAISE NOTICE '============================================';
RAISE NOTICE '✅ SEED COMPLETADO EXITOSAMENTE';
RAISE NOTICE '============================================';
RAISE NOTICE 'Usuarios creados (contraseña: test123):';
RAISE NOTICE '  Director: daniela.soto@colegio.cl';
RAISE NOTICE '  UTP: andres.nunez@colegio.cl';
RAISE NOTICE '  Profesores: maria.gonzalez@colegio.cl, carlos.munoz@colegio.cl, ...';
RAISE NOTICE '============================================';

END $$;
