-- ===================================================================
-- SEED PARTE 3: Apoderados, Estudiantes y Relaciones
-- ===================================================================
DO $$
DECLARE
    v_school_pred UUID;
    v_password_hash TEXT := '$argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A';

    v_apo1 UUID; v_apo2 UUID; v_apo3 UUID; v_apo4 UUID; v_apo5 UUID;
    v_apo6 UUID; v_apo7 UUID; v_apo8 UUID; v_apo9 UUID; v_apo10 UUID;
    v_apo11 UUID; v_apo12 UUID; v_apo13 UUID; v_apo14 UUID; v_apo15 UUID;
    v_apo16 UUID; v_apo17 UUID; v_apo18 UUID; v_apo19 UUID; v_apo20 UUID;

    v_stu1 UUID; v_stu2 UUID; v_stu3 UUID; v_stu4 UUID; v_stu5 UUID;
    v_stu6 UUID; v_stu7 UUID; v_stu8 UUID; v_stu9 UUID; v_stu10 UUID;
    v_stu11 UUID; v_stu12 UUID; v_stu13 UUID; v_stu14 UUID; v_stu15 UUID;
    v_stu16 UUID; v_stu17 UUID; v_stu18 UUID; v_stu19 UUID; v_stu20 UUID;
    v_stu21 UUID; v_stu22 UUID; v_stu23 UUID; v_stu24 UUID; v_stu25 UUID;
    v_stu26 UUID; v_stu27 UUID; v_stu28 UUID; v_stu29 UUID; v_stu30 UUID;

    v_prof_lin1 UUID; v_prof_ing1 UUID; v_prof_ing2 UUID; v_prof_efi1 UUID;
    v_admision1 UUID; v_admision2 UUID; v_agente3 UUID;
BEGIN
    SELECT id INTO v_school_pred FROM schools WHERE name = 'Colegio Predeterminado';
    SELECT id INTO v_prof_lin1 FROM users WHERE email = 'elisa.huenchullan@colegio.cl';
    SELECT id INTO v_prof_ing1 FROM users WHERE email = 'laura.fernandez@colegio.cl';
    SELECT id INTO v_prof_ing2 FROM users WHERE email = 'thomas.muller@colegio.cl';
    SELECT id INTO v_prof_efi1 FROM users WHERE email = 'carolina.rivas@colegio.cl';
    SELECT id INTO v_admision1 FROM users WHERE email = 'paulina.riquelme@colegio.cl';
    SELECT id INTO v_admision2 FROM users WHERE email = 'matias.cerda@colegio.cl';
    SELECT id INTO v_agente3 FROM users WHERE email = 'francisca.irarrazabal@ventas.cl';

    -- ===================================================================
    -- 10. APODERADOS
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '85.567.890-1', 'Ricardo Gonzalez Munoz', 'ricardo.gonzalez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ricardo.gonzalez@apoderados.cl') RETURNING id INTO v_apo1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '45.678.901-2', 'Marta Rivas Lopez', 'marta.rivas@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'marta.rivas@apoderados.cl') RETURNING id INTO v_apo2;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '46.789.012-3', 'Pablo Torres Vega', 'pablo.torres@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'pablo.torres@apoderados.cl') RETURNING id INTO v_apo3;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '47.890.123-4', 'Catherine Fernandez Soto', 'catherine.fernandez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'catherine.fernandez@apoderados.cl') RETURNING id INTO v_apo4;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '48.901.234-5', 'Felipe Ramirez Herrera', 'felipe.ramirez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'felipe.ramirez@apoderados.cl') RETURNING id INTO v_apo5;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '49.012.345-6', 'Andrea Morales Rojas', 'andrea.morales@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'andrea.morales@apoderados.cl') RETURNING id INTO v_apo6;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '50.123.456-7', 'Jorge Araya Contreras', 'jorge.araya@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'jorge.araya@apoderados.cl') RETURNING id INTO v_apo7;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '51.234.567-8', 'Marcela Valenzuela Rivas', 'marcela.valenzuela@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'marcela.valenzuela@apoderados.cl') RETURNING id INTO v_apo8;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '52.345.678-9', 'Cristian Navarrete Toledo', 'cristian.navarrete@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'cristian.navarrete@apoderados.cl') RETURNING id INTO v_apo9;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '53.456.789-0', 'Karen Sandoval Diaz', 'karen.sandoval@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'karen.sandoval@apoderados.cl') RETURNING id INTO v_apo10;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '54.567.890-1', 'Diego Gutierrez Bravo', 'diego.gutierrez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'diego.gutierrez@apoderados.cl') RETURNING id INTO v_apo11;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '55.678.901-2', 'Paula Fuentes Sepulveda', 'paula.fuentes@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'paula.fuentes@apoderados.cl') RETURNING id INTO v_apo12;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '56.789.012-3', 'Alvaro Vargas Medina', 'alvaro.vargas@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'alvaro.vargas@apoderados.cl') RETURNING id INTO v_apo13;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '57.890.123-4', 'Daniela Orellana Paredes', 'daniela.orellana@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'daniela.orellana@apoderados.cl') RETURNING id INTO v_apo14;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '58.901.234-5', 'Andres Molina Campos', 'andres.molina@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'andres.molina@apoderados.cl') RETURNING id INTO v_apo15;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '59.012.345-6', 'Carolina Cruz Garrido', 'carolina.cruz@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carolina.cruz@apoderados.cl') RETURNING id INTO v_apo16;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '60.123.456-7', 'Claudio Pena Vasquez', 'claudio.pena@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'claudio.pena@apoderados.cl') RETURNING id INTO v_apo17;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '61.234.567-8', 'Pamela Martinez Lopez', 'pamela.martinez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'pamela.martinez@apoderados.cl') RETURNING id INTO v_apo18;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '62.345.678-9', 'Hernan Valdivia Sanchez', 'hernan.valdivia@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'hernan.valdivia@apoderados.cl') RETURNING id INTO v_apo19;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '63.456.789-0', 'Sofia Lagos Pizarro', 'sofia.lagos@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'sofia.lagos@apoderados.cl') RETURNING id INTO v_apo20;

    RAISE NOTICE 'Apoderados creados correctamente';

    -- ===================================================================
    -- 11. ESTUDIANTES CON FICHAS COMPLETAS
    -- ===================================================================
    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '24.567.890-1', 'Mateo', 'Gonzalez Munoz', 'mateo.gonzalez@correo.cl', '+56 9 1234 5678', '1 Basico', 'A', '1', 'AL', '0', 'N', NULL, 'Penicilina', 'Ricardo Gonzalez', '+56 9 1111 1111', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '24.567.890-1') RETURNING id INTO v_stu1;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '25.678.901-2', 'Valentina', 'Torres Rivas', 'valentina.torres@correo.cl', '+56 9 2345 6789', '1 Basico', 'A', '1', 'AL', '1', 'N', NULL, NULL, 'Marta Rivas', '+56 9 2222 2222', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '25.678.901-2') RETURNING id INTO v_stu2;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '26.789.012-3', 'Benjamin', 'Martinez Soto', 'benjamin.martinez@correo.cl', '+56 9 3456 7890', '2 Basico', 'A', '2', 'AL', '0', 'N', NULL, NULL, 'Pablo Torres', '+56 9 3333 3333', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '26.789.012-3') RETURNING id INTO v_stu3;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '27.890.123-4', 'Isabella', 'Fernandez Vega', 'isabella.fernandez@correo.cl', '+56 9 4567 8901', '2 Basico', 'A', '2', 'AL', '0', 'T', 'Asma leve', 'Lacteos', 'Catherine Fernandez', '+56 9 4444 4444', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '27.890.123-4') RETURNING id INTO v_stu4;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '28.901.234-5', 'Santiago', 'Lopez Castillo', 'santiago.lopez@correo.cl', '+56 9 5678 9012', '3 Basico', 'B', '3', 'AL', '2', 'N', NULL, NULL, 'Felipe Ramirez', '+56 9 5555 5555', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '28.901.234-5') RETURNING id INTO v_stu5;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '29.012.345-6', 'Emilia', 'Ramirez Herrera', 'emilia.ramirez@correo.cl', '+56 9 6789 0123', '3 Basico', 'B', '3', 'AL', '0', 'N', NULL, 'Frutos secos', 'Andrea Morales', '+56 9 6666 6666', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '29.012.345-6') RETURNING id INTO v_stu6;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '30.123.456-7', 'Lucas', 'Perez Diaz', 'lucas.perez@correo.cl', '+56 9 7890 1234', '4 Basico', 'A', '4', 'AL', '0', 'N', NULL, NULL, 'Jorge Araya', '+56 9 7777 7777', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '30.123.456-7') RETURNING id INTO v_stu7;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '31.234.567-8', 'Sofia', 'Morales Rojas', 'sofia.morales@correo.cl', '+56 9 8901 2345', '4 Basico', 'A', '4', 'AL', '1', 'N', NULL, NULL, 'Andrea Morales', '+56 9 6666 6666', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '31.234.567-8') RETURNING id INTO v_stu8;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '32.345.678-9', 'Gabriel', 'Araya Contreras', 'gabriel.araya@correo.cl', '+56 9 9012 3456', '5 Basico', 'C', '5', 'AL', '0', 'P', 'Trastorno deficit atencional', NULL, 'Jorge Araya', '+56 9 7777 7777', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '32.345.678-9') RETURNING id INTO v_stu9;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '33.456.789-0', 'Martina', 'Castro Pereira', 'martina.castro@correo.cl', '+56 9 0123 4567', '5 Basico', 'C', '5', 'AL', '0', 'N', NULL, NULL, 'Marcela Valenzuela', '+56 9 8888 8888', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '33.456.789-0') RETURNING id INTO v_stu10;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '34.567.890-1', 'Joaquin', 'Valenzuela Rivas', 'joaquin.valenzuela@correo.cl', '+56 9 1122 3344', '6 Basico', 'A', '6', 'AL', '0', 'N', NULL, 'Penicilina', 'Marcela Valenzuela', '+56 9 8888 8888', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '34.567.890-1') RETURNING id INTO v_stu11;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '35.678.901-2', 'Florencia', 'Navarrete Toledo', 'florencia.navarrete@correo.cl', '+56 9 2233 4455', '6 Basico', 'A', '6', 'AL', '1', 'N', NULL, NULL, 'Cristian Navarrete', '+56 9 9999 9999', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '35.678.901-2') RETURNING id INTO v_stu12;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '36.789.012-3', 'Agustin', 'Sandoval Diaz', 'agustin.sandoval@correo.cl', '+56 9 3344 5566', '7 Basico', 'A', '7', 'AL', '0', 'N', NULL, NULL, 'Karen Sandoval', '+56 9 1010 1010', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '36.789.012-3') RETURNING id INTO v_stu13;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '37.890.123-4', 'Josefa', 'Gutierrez Bravo', 'josefa.gutierrez@correo.cl', '+56 9 4455 6677', '7 Basico', 'B', '7', 'RE', '0', 'N', NULL, NULL, 'Diego Gutierrez', '+56 9 1212 1212', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '37.890.123-4') RETURNING id INTO v_stu14;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '38.901.234-5', 'Maximiliano', 'Fuentes Sepulveda', 'maximiliano.fuentes@correo.cl', '+56 9 5566 7788', '8 Basico', 'A', '8', 'AL', '0', 'N', NULL, NULL, 'Paula Fuentes', '+56 9 1313 1313', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '38.901.234-5') RETURNING id INTO v_stu15;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '39.012.345-6', 'Antonia', 'Vargas Medina', 'antonia.vargas@correo.cl', '+56 9 6677 8899', '8 Basico', 'A', '8', 'AL', '0', 'N', NULL, NULL, 'Alvaro Vargas', '+56 9 1414 1414', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '39.012.345-6') RETURNING id INTO v_stu16;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '40.123.456-7', 'Vicente', 'Orellana Paredes', 'vicente.orellana@correo.cl', '+56 9 7788 9900', 'I Medio', 'A', '9', 'AL', '0', 'N', NULL, NULL, 'Daniela Orellana', '+56 9 1515 1515', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '40.123.456-7') RETURNING id INTO v_stu17;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '41.234.567-8', 'Catalina', 'Molina Campos', 'catalina.molina@correo.cl', '+56 9 8899 0011', 'I Medio', 'B', '9', 'AL', '2', 'T', 'Diabetes tipo 1', NULL, 'Andres Molina', '+56 9 1616 1616', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '41.234.567-8') RETURNING id INTO v_stu18;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '42.345.678-9', 'Nicolas', 'Cruz Garrido', 'nicolas.cruz@correo.cl', '+56 9 9900 1122', 'II Medio', 'A', '10', 'AL', '0', 'N', NULL, NULL, 'Carolina Cruz', '+56 9 1717 1717', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '42.345.678-9') RETURNING id INTO v_stu19;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '43.456.789-0', 'Amanda', 'Pena Vasquez', 'amanda.pena@correo.cl', '+56 9 0011 2233', 'II Medio', 'A', '10', 'AL', '0', 'N', NULL, NULL, 'Claudio Pena', '+56 9 1818 1818', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '43.456.789-0') RETURNING id INTO v_stu20;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '64.567.890-1', 'Tomas', 'Martinez Lopez', 'tomas.martinez@correo.cl', '+56 9 1919 2020', '3 Basico', 'B', '3', 'AL', '0', 'N', NULL, NULL, 'Pamela Martinez', '+56 9 1234 4321', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '64.567.890-1') RETURNING id INTO v_stu21;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '65.678.901-2', 'Trinidad', 'Valdivia Soto', 'trinidad.valdivia@correo.cl', '+56 9 2020 3030', '4 Basico', 'A', '4', 'AL', '0', 'N', NULL, NULL, 'Hernan Valdivia', '+56 9 2345 5432', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '65.678.901-2') RETURNING id INTO v_stu22;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '66.789.012-3', 'Cristobal', 'Lagos Pizarro', 'cristobal.lagos@correo.cl', '+56 9 3030 4040', '5 Basico', 'C', '5', 'AL', '1', 'N', NULL, NULL, 'Sofia Lagos', '+56 9 3456 6543', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '66.789.012-3') RETURNING id INTO v_stu23;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '67.890.123-4', 'Rayen', 'Huenuán Lefian', 'rayen.huenuan@correo.cl', '+56 9 4040 5050', '5 Basico', 'C', '5', 'AL', '0', 'N', NULL, NULL, 'Elisa Huenchullan', '+56 9 4567 7654', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '67.890.123-4') RETURNING id INTO v_stu24;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '68.901.234-5', 'Leon', 'Riquelme Vega', 'leon.riquelme@correo.cl', '+56 9 5050 6060', '6 Basico', 'A', '6', 'AL', '0', 'N', 'Hiperactividad', NULL, 'Paulina Riquelme', '+56 9 5678 8765', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '68.901.234-5') RETURNING id INTO v_stu25;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '69.012.345-6', 'Amparo', 'Herrera Diaz', 'amparo.herrera@correo.cl', '+56 9 6060 7070', '7 Basico', 'A', '7', 'AL', '0', 'N', NULL, 'Penicilina', 'Laura Fernandez', '+56 9 6789 9876', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '69.012.345-6') RETURNING id INTO v_stu26;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '70.123.456-7', 'Bruno', 'Cifuentes Leiva', 'bruno.cifuentes@correo.cl', '+56 9 7070 8080', '8 Basico', 'A', '8', 'AL', '0', 'N', NULL, NULL, 'Matias Cerda', '+56 9 7890 0987', 'Tutor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '70.123.456-7') RETURNING id INTO v_stu27;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '71.234.567-8', 'Magdalena', 'Lira Cox', 'magdalena.lira@correo.cl', '+56 9 8080 9090', 'I Medio', 'A', '9', 'AL', '2', 'N', NULL, 'Frutos secos', 'Francisca Irarrazabal', '+56 9 8901 1098', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '71.234.567-8') RETURNING id INTO v_stu28;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '72.345.678-9', 'Felix', 'Muller Araya', 'felix.muller@correo.cl', '+56 9 9090 1010', 'II Medio', 'A', '10', 'RE', '0', 'P', 'Dislexia', NULL, 'Thomas Muller', '+56 9 9012 2109', 'Padre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '72.345.678-9') RETURNING id INTO v_stu29;

    INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, cod_nivel, condicion, prioritario, nee, diseases, allergies, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, enrolled, school_id)
    SELECT gen_random_uuid(), '73.456.789-0', 'Julieta', 'Soto Rivas', 'julieta.soto@correo.cl', '+56 9 1010 1112', '1 Basico', 'A', '1', 'AL', '0', 'N', NULL, NULL, 'Carolina Rivas', '+56 9 0123 3210', 'Madre', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM students WHERE rut = '73.456.789-0') RETURNING id INTO v_stu30;

    RAISE NOTICE 'Estudiantes creados correctamente';

    -- ===================================================================
    -- 12. RELACIONES APODERADO-ESTUDIANTE
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
