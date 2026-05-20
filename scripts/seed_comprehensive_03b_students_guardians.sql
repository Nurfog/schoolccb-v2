-- ===================================================================
-- FIX: Obtener IDs de estudiantes existentes + agregar los nuevos
-- y crear relaciones apoderado-estudiante
-- ===================================================================
DO $$
DECLARE
    v_school_pred UUID;
    v_stu1 UUID; v_stu2 UUID; v_stu3 UUID; v_stu4 UUID; v_stu5 UUID;
    v_stu6 UUID; v_stu7 UUID; v_stu8 UUID; v_stu9 UUID; v_stu10 UUID;
    v_stu11 UUID; v_stu12 UUID; v_stu13 UUID; v_stu14 UUID; v_stu15 UUID;
    v_stu16 UUID; v_stu17 UUID; v_stu18 UUID; v_stu19 UUID; v_stu20 UUID;
    v_stu21 UUID; v_stu22 UUID; v_stu23 UUID; v_stu24 UUID; v_stu25 UUID;
    v_stu26 UUID; v_stu27 UUID; v_stu28 UUID; v_stu29 UUID; v_stu30 UUID;

    v_apo1 UUID; v_apo2 UUID; v_apo3 UUID; v_apo4 UUID; v_apo5 UUID;
    v_apo6 UUID; v_apo7 UUID; v_apo8 UUID; v_apo9 UUID; v_apo10 UUID;
    v_apo11 UUID; v_apo12 UUID; v_apo13 UUID; v_apo14 UUID; v_apo15 UUID;
    v_apo16 UUID; v_apo17 UUID; v_apo18 UUID; v_apo19 UUID; v_apo20 UUID;

    v_prof_lin1 UUID; v_prof_ing1 UUID; v_prof_ing2 UUID; v_prof_efi1 UUID;
    v_admision1 UUID; v_admision2 UUID; v_agente3 UUID;
    v_password_hash TEXT := '$argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A';
BEGIN
    SELECT id INTO v_school_pred FROM schools WHERE name = 'Colegio Predeterminado';

    -- Obtener IDs de estudiantes existentes (20 originales)
    SELECT id INTO v_stu1 FROM students WHERE rut = '24.567.890-1';
    SELECT id INTO v_stu2 FROM students WHERE rut = '25.678.901-2';
    SELECT id INTO v_stu3 FROM students WHERE rut = '26.789.012-3';
    SELECT id INTO v_stu4 FROM students WHERE rut = '27.890.123-4';
    SELECT id INTO v_stu5 FROM students WHERE rut = '28.901.234-5';
    SELECT id INTO v_stu6 FROM students WHERE rut = '29.012.345-6';
    SELECT id INTO v_stu7 FROM students WHERE rut = '30.123.456-7';
    SELECT id INTO v_stu8 FROM students WHERE rut = '31.234.567-8';
    SELECT id INTO v_stu9 FROM students WHERE rut = '32.345.678-9';
    SELECT id INTO v_stu10 FROM students WHERE rut = '33.456.789-0';
    SELECT id INTO v_stu11 FROM students WHERE rut = '34.567.890-1';
    SELECT id INTO v_stu12 FROM students WHERE rut = '35.678.901-2';
    SELECT id INTO v_stu13 FROM students WHERE rut = '36.789.012-3';
    SELECT id INTO v_stu14 FROM students WHERE rut = '37.890.123-4';
    SELECT id INTO v_stu15 FROM students WHERE rut = '38.901.234-5';
    SELECT id INTO v_stu16 FROM students WHERE rut = '39.012.345-6';
    SELECT id INTO v_stu17 FROM students WHERE rut = '40.123.456-7';
    SELECT id INTO v_stu18 FROM students WHERE rut = '41.234.567-8';
    SELECT id INTO v_stu19 FROM students WHERE rut = '42.345.678-9';
    SELECT id INTO v_stu20 FROM students WHERE rut = '43.456.789-0';

    -- Insertar 10 nuevos estudiantes (solo si no existen)
    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '64.567.890-1', 'Tomas', 'Martinez Lopez', 'tomas.martinez@correo.cl', '+56 9 1919 2020', '1° Básico', 'A', '1', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '64.567.890-1')
    RETURNING id INTO v_stu21;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '65.678.901-2', 'Trinidad', 'Valdivia Soto', 'trinidad.valdivia@correo.cl', '+56 9 2020 3030', '4° Básico', 'A', '4', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '65.678.901-2')
    RETURNING id INTO v_stu22;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '66.789.012-3', 'Cristobal', 'Lagos Pizarro', 'cristobal.lagos@correo.cl', '+56 9 3030 4040', '5° Básico', 'C', '5', 'AL', '1', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '66.789.012-3')
    RETURNING id INTO v_stu23;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '67.890.123-4', 'Rayen', 'Huenuán Lefian', 'rayen.huenuan@correo.cl', '+56 9 4040 5050', '5° Básico', 'C', '5', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '67.890.123-4')
    RETURNING id INTO v_stu24;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '68.901.234-5', 'Leon', 'Riquelme Vega', 'leon.riquelme@correo.cl', '+56 9 5050 6060', '6° Básico', 'A', '6', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '68.901.234-5')
    RETURNING id INTO v_stu25;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '69.012.345-6', 'Amparo', 'Herrera Diaz', 'amparo.herrera@correo.cl', '+56 9 6060 7070', '7° Básico', 'A', '7', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '69.012.345-6')
    RETURNING id INTO v_stu26;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '70.123.456-7', 'Bruno', 'Cifuentes Leiva', 'bruno.cifuentes@correo.cl', '+56 9 7070 8080', '8° Básico', 'A', '8', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '70.123.456-7')
    RETURNING id INTO v_stu27;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '71.234.567-8', 'Magdalena', 'Lira Cox', 'magdalena.lira@correo.cl', '+56 9 8080 9090', 'I° Medio', 'A', '9', 'AL', '2', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '71.234.567-8')
    RETURNING id INTO v_stu28;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '72.345.678-9', 'Felix', 'Muller Araya', 'felix.muller@correo.cl', '+56 9 9090 1010', 'II° Medio', 'A', '10', 'RE', '0', 'P', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '72.345.678-9')
    RETURNING id INTO v_stu29;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled, school_id)
    SELECT gen_random_uuid(), '73.456.789-0', 'Julieta', 'Soto Rivas', 'julieta.soto@correo.cl', '+56 9 1010 1112', '1° Básico', 'A', '1', 'AL', '0', 'N', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '73.456.789-0')
    RETURNING id INTO v_stu30;

    -- Si los nuevos no se insertaron (ya existian), obtener sus IDs
    IF v_stu21 IS NULL THEN SELECT id INTO v_stu21 FROM students WHERE rut = '64.567.890-1'; END IF;
    IF v_stu22 IS NULL THEN SELECT id INTO v_stu22 FROM students WHERE rut = '65.678.901-2'; END IF;
    IF v_stu23 IS NULL THEN SELECT id INTO v_stu23 FROM students WHERE rut = '66.789.012-3'; END IF;
    IF v_stu24 IS NULL THEN SELECT id INTO v_stu24 FROM students WHERE rut = '67.890.123-4'; END IF;
    IF v_stu25 IS NULL THEN SELECT id INTO v_stu25 FROM students WHERE rut = '68.901.234-5'; END IF;
    IF v_stu26 IS NULL THEN SELECT id INTO v_stu26 FROM students WHERE rut = '69.012.345-6'; END IF;
    IF v_stu27 IS NULL THEN SELECT id INTO v_stu27 FROM students WHERE rut = '70.123.456-7'; END IF;
    IF v_stu28 IS NULL THEN SELECT id INTO v_stu28 FROM students WHERE rut = '71.234.567-8'; END IF;
    IF v_stu29 IS NULL THEN SELECT id INTO v_stu29 FROM students WHERE rut = '72.345.678-9'; END IF;
    IF v_stu30 IS NULL THEN SELECT id INTO v_stu30 FROM students WHERE rut = '73.456.789-0'; END IF;

    -- Obtener IDs de apoderados
    SELECT id INTO v_apo1 FROM users WHERE email = 'ricardo.gonzalez@apoderados.cl';
    SELECT id INTO v_apo2 FROM users WHERE email = 'marta.rivas@apoderados.cl';
    SELECT id INTO v_apo3 FROM users WHERE email = 'pablo.torres@apoderados.cl';
    SELECT id INTO v_apo4 FROM users WHERE email = 'catherine.fernandez@apoderados.cl';
    SELECT id INTO v_apo5 FROM users WHERE email = 'felipe.ramirez@apoderados.cl';
    SELECT id INTO v_apo6 FROM users WHERE email = 'andrea.morales@apoderados.cl';
    SELECT id INTO v_apo7 FROM users WHERE email = 'jorge.araya@apoderados.cl';
    SELECT id INTO v_apo8 FROM users WHERE email = 'marcela.valenzuela@apoderados.cl';
    SELECT id INTO v_apo9 FROM users WHERE email = 'cristian.navarrete@apoderados.cl';
    SELECT id INTO v_apo10 FROM users WHERE email = 'karen.sandoval@apoderados.cl';
    SELECT id INTO v_apo11 FROM users WHERE email = 'diego.gutierrez@apoderados.cl';
    SELECT id INTO v_apo12 FROM users WHERE email = 'paula.fuentes@apoderados.cl';
    SELECT id INTO v_apo13 FROM users WHERE email = 'alvaro.vargas@apoderados.cl';
    SELECT id INTO v_apo14 FROM users WHERE email = 'daniela.orellana@apoderados.cl';
    SELECT id INTO v_apo15 FROM users WHERE email = 'andres.molina@apoderados.cl';
    SELECT id INTO v_apo16 FROM users WHERE email = 'carolina.cruz@apoderados.cl';
    SELECT id INTO v_apo17 FROM users WHERE email = 'claudio.pena@apoderados.cl';
    SELECT id INTO v_apo18 FROM users WHERE email = 'pamela.martinez@apoderados.cl';
    SELECT id INTO v_apo19 FROM users WHERE email = 'hernan.valdivia@apoderados.cl';
    SELECT id INTO v_apo20 FROM users WHERE email = 'sofia.lagos@apoderados.cl';

    SELECT id INTO v_prof_lin1 FROM users WHERE email = 'elisa.huenchullan@colegio.cl';
    SELECT id INTO v_prof_ing1 FROM users WHERE email = 'laura.fernandez@colegio.cl';
    SELECT id INTO v_prof_ing2 FROM users WHERE email = 'thomas.muller@colegio.cl';
    SELECT id INTO v_prof_efi1 FROM users WHERE email = 'carolina.rivas@colegio.cl';
    SELECT id INTO v_admision1 FROM users WHERE email = 'paulina.riquelme@colegio.cl';
    SELECT id INTO v_admision2 FROM users WHERE email = 'matias.cerda@colegio.cl';
    SELECT id INTO v_agente3 FROM users WHERE email = 'francisca.irarrazabal@ventas.cl';

    RAISE NOTICE 'IDs obtenidos correctamente';

    -- ===================================================================
    -- RELACIONES APODERADO-ESTUDIANTE
    -- ===================================================================
    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu1, v_apo1, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu1 AND guardian_user_id = v_apo1);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu2, v_apo2, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu2 AND guardian_user_id = v_apo2);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu3, v_apo3, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu3 AND guardian_user_id = v_apo3);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu4, v_apo4, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu4 AND guardian_user_id = v_apo4);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu5, v_apo5, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu5 AND guardian_user_id = v_apo5);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu6, v_apo6, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu6 AND guardian_user_id = v_apo6);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu7, v_apo7, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu7 AND guardian_user_id = v_apo7);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu8, v_apo6, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu8 AND guardian_user_id = v_apo6);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu9, v_apo7, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu9 AND guardian_user_id = v_apo7);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu10, v_apo8, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu10 AND guardian_user_id = v_apo8);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu11, v_apo8, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu11 AND guardian_user_id = v_apo8);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu12, v_apo9, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu12 AND guardian_user_id = v_apo9);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu13, v_apo10, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu13 AND guardian_user_id = v_apo10);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu14, v_apo11, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu14 AND guardian_user_id = v_apo11);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu15, v_apo12, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu15 AND guardian_user_id = v_apo12);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu16, v_apo13, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu16 AND guardian_user_id = v_apo13);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu17, v_apo14, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu17 AND guardian_user_id = v_apo14);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu18, v_apo15, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu18 AND guardian_user_id = v_apo15);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu19, v_apo16, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu19 AND guardian_user_id = v_apo16);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu20, v_apo17, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu20 AND guardian_user_id = v_apo17);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu21, v_apo18, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu21 AND guardian_user_id = v_apo18);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu22, v_apo19, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu22 AND guardian_user_id = v_apo19);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu23, v_apo20, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu23 AND guardian_user_id = v_apo20);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu24, v_prof_lin1, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu24 AND guardian_user_id = v_prof_lin1);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu25, v_admision1, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu25 AND guardian_user_id = v_admision1);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu26, v_prof_ing1, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu26 AND guardian_user_id = v_prof_ing1);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu27, v_admision2, 'Tutor', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu27 AND guardian_user_id = v_admision2);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu28, v_agente3, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu28 AND guardian_user_id = v_agente3);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu29, v_prof_ing2, 'Padre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu29 AND guardian_user_id = v_prof_ing2);

    INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
    SELECT gen_random_uuid(), v_stu30, v_prof_efi1, 'Madre', true, true
    WHERE NOT EXISTS (SELECT 1 FROM guardian_relationships WHERE student_id = v_stu30 AND guardian_user_id = v_prof_efi1);

    RAISE NOTICE 'Relaciones apoderado-estudiante creadas correctamente';
END $$;
