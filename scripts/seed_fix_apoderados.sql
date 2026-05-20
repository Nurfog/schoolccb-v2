DO $$
DECLARE
    v_school_pred UUID;
    v_password_hash TEXT := '$argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A';
BEGIN
    SELECT id INTO v_school_pred FROM schools WHERE name = 'Colegio Predeterminado';

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '85.567.890-1', 'Ricardo Gonzalez Munoz', 'ricardo.gonzalez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ricardo.gonzalez@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '45.678.901-2', 'Marta Rivas Lopez', 'marta.rivas@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'marta.rivas@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '46.789.012-3', 'Pablo Torres Vega', 'pablo.torres@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'pablo.torres@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '47.890.123-4', 'Catherine Fernandez Soto', 'catherine.fernandez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'catherine.fernandez@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '48.901.234-5', 'Felipe Ramirez Herrera', 'felipe.ramirez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'felipe.ramirez@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '49.012.345-6', 'Andrea Morales Rojas', 'andrea.morales@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'andrea.morales@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '50.123.456-7', 'Jorge Araya Contreras', 'jorge.araya@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'jorge.araya@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '51.234.567-8', 'Marcela Valenzuela Rivas', 'marcela.valenzuela@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'marcela.valenzuela@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '52.345.678-9', 'Cristian Navarrete Toledo', 'cristian.navarrete@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'cristian.navarrete@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '53.456.789-0', 'Karen Sandoval Diaz', 'karen.sandoval@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'karen.sandoval@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '54.567.890-1', 'Diego Gutierrez Bravo', 'diego.gutierrez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'diego.gutierrez@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '55.678.901-2', 'Paula Fuentes Sepulveda', 'paula.fuentes@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'paula.fuentes@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '56.789.012-3', 'Alvaro Vargas Medina', 'alvaro.vargas@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'alvaro.vargas@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '57.890.123-4', 'Daniela Orellana Paredes', 'daniela.orellana@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'daniela.orellana@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '58.901.234-5', 'Andres Molina Campos', 'andres.molina@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'andres.molina@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '59.012.345-6', 'Carolina Cruz Garrido', 'carolina.cruz@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carolina.cruz@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '60.123.456-7', 'Claudio Pena Vasquez', 'claudio.pena@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'claudio.pena@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '61.234.567-8', 'Pamela Martinez Lopez', 'pamela.martinez@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'pamela.martinez@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '62.345.678-9', 'Hernan Valdivia Sanchez', 'hernan.valdivia@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'hernan.valdivia@apoderados.cl');

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '63.456.789-0', 'Sofia Lagos Pizarro', 'sofia.lagos@apoderados.cl', v_password_hash, 'Apoderado', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'sofia.lagos@apoderados.cl');

    RAISE NOTICE 'Apoderados creados correctamente';
END $$;
