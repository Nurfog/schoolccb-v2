-- ===================================================================
-- SEED COMPREHENSIVO PARTE 1: Usuarios, Roles, Asignaciones
-- ===================================================================
-- Contraseña para todos los usuarios: test123
-- Hash Argon2id: $argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A
-- ===================================================================

DO $$
DECLARE
    v_corp_id UUID;
    v_school_pred UUID;
    v_school_san_andres UUID;
    v_school_los_andes UUID;
    v_school_liceo UUID;
    v_password_hash TEXT := '$argon2id$v=19$m=524288,t=2,p=1$RUlxRzAyWk9MM3dpUk5Vc1J5R2QxQQ$dp/2i5+BKot7ytBn1nt/VwtExPkjRv/kTRDJNm+wl1A';

    v_dir1 UUID; v_dir2 UUID; v_dir3 UUID; v_dir4 UUID;
    v_utp1 UUID; v_utp2 UUID; v_utp3 UUID; v_utp4 UUID;
    v_admin_school UUID; v_admin_corp UUID;
    v_sost UUID;
    v_jefe_ventas UUID; v_agente1 UUID; v_agente2 UUID; v_agente3 UUID;
    v_admision1 UUID; v_admision2 UUID;

    v_prof_len1 UUID; v_prof_len2 UUID; v_prof_len3 UUID; v_prof_len4 UUID; v_prof_len5 UUID;
    v_prof_mat1 UUID; v_prof_mat2 UUID; v_prof_mat3 UUID; v_prof_mat4 UUID;
    v_prof_cie1 UUID; v_prof_cie2 UUID; v_prof_cie3 UUID;
    v_prof_his1 UUID; v_prof_his2 UUID;
    v_prof_ing1 UUID; v_prof_ing2 UUID;
    v_prof_art1 UUID; v_prof_mus1 UUID; v_prof_efi1 UUID; v_prof_tec1 UUID;
    v_prof_rel1 UUID; v_prof_ori1 UUID;
    v_prof_fil1 UUID; v_prof_ciu1 UUID;
    v_prof_ldi1 UUID; v_prof_bio1 UUID; v_prof_fis1 UUID; v_prof_qui1 UUID;
    v_prof_lin1 UUID;
BEGIN

    SELECT id INTO v_corp_id FROM corporations WHERE name = 'Corporación Educativa';
    SELECT id INTO v_school_pred FROM schools WHERE name = 'Colegio Predeterminado';
    SELECT id INTO v_school_san_andres FROM schools WHERE name = 'Colegio San Andrés';
    SELECT id INTO v_school_los_andes FROM schools WHERE name = 'Colegio Los Andes';
    SELECT id INTO v_school_liceo FROM schools WHERE name = 'Liceo Bicentenario Norte';

    RAISE NOTICE 'Referencias obtenidas - Corp: %, School: %', v_corp_id, v_school_pred;

    -- ===================================================================
    -- 1.1 DIRECTORES
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '22.345.678-9', 'Daniela Soto Pizarro', 'daniela.soto@colegio.cl', v_password_hash, 'Director', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'daniela.soto@colegio.cl')
    RETURNING id INTO v_dir1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '24.111.222-3', 'Ricardo Alvarez Munoz', 'ricardo.alvarez@sansanandres.cl', v_password_hash, 'Director', true, v_school_san_andres
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ricardo.alvarez@sansanandres.cl')
    RETURNING id INTO v_dir2;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '25.222.333-4', 'Maria Veronica Campos', 'maria.campos@losandes.cl', v_password_hash, 'Director', true, v_school_los_andes
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'maria.campos@losandes.cl')
    RETURNING id INTO v_dir3;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '26.333.444-5', 'Patricio Bustos Silva', 'patricio.bustos@liceonorte.cl', v_password_hash, 'Director', true, v_school_liceo
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'patricio.bustos@liceonorte.cl')
    RETURNING id INTO v_dir4;

    -- ===================================================================
    -- 1.2 UTP
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '23.456.789-0', 'Andres Nunez Campos', 'andres.nunez@colegio.cl', v_password_hash, 'UTP', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'andres.nunez@colegio.cl')
    RETURNING id INTO v_utp1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '27.444.555-6', 'Carmen Luz Rojas', 'carmen.rojas@sansanandres.cl', v_password_hash, 'UTP', true, v_school_san_andres
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carmen.rojas@sansanandres.cl')
    RETURNING id INTO v_utp2;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '28.555.666-7', 'Hector Molina Vega', 'hector.molina@losandes.cl', v_password_hash, 'UTP', true, v_school_los_andes
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'hector.molina@losandes.cl')
    RETURNING id INTO v_utp3;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '29.666.777-8', 'Sandra Paredes Diaz', 'sandra.paredes@liceonorte.cl', v_password_hash, 'UTP', true, v_school_liceo
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'sandra.paredes@liceonorte.cl')
    RETURNING id INTO v_utp4;

    -- ===================================================================
    -- 1.3 ADMINISTRADORES
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id, admin_type)
    SELECT gen_random_uuid(), '30.777.888-9', 'Rodrigo Fuentes Maldonado', 'rodrigo.fuentes@colegio.cl', v_password_hash, 'Administrador', true, v_school_pred, 'school'
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'rodrigo.fuentes@colegio.cl')
    RETURNING id INTO v_admin_school;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, corporation_id, admin_type)
    SELECT gen_random_uuid(), '31.888.999-0', 'Monica Larrain Echeverria', 'monica.larrain@corporacion.cl', v_password_hash, 'Administrador', true, v_corp_id, 'global'
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'monica.larrain@corporacion.cl')
    RETURNING id INTO v_admin_corp;

    -- ===================================================================
    -- 1.4 SOSTENEDOR
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active, corporation_id)
    SELECT gen_random_uuid(), '32.999.000-1', 'Fernando Hurtado Oyarzun', 'fernando.hurtado@corporacion.cl', v_password_hash, 'Sostenedor', true, v_corp_id
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'fernando.hurtado@corporacion.cl')
    RETURNING id INTO v_sost;

    -- ===================================================================
    -- 1.5 EQUIPO COMERCIAL (CRM VENTAS)
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active)
    SELECT gen_random_uuid(), '33.000.111-2', 'Cristobal Padilla Ruiz', 'cristobal.padilla@ventas.cl', v_password_hash, 'JefeVentas', true
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'cristobal.padilla@ventas.cl')
    RETURNING id INTO v_jefe_ventas;

    INSERT INTO users (id, rut, name, email, password_hash, role, active)
    SELECT gen_random_uuid(), '34.111.222-3', 'Valentina Aguirre Soto', 'valentina.aguirre@ventas.cl', v_password_hash, 'AgenteVentas', true
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'valentina.aguirre@ventas.cl')
    RETURNING id INTO v_agente1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active)
    SELECT gen_random_uuid(), '35.222.333-4', 'Sebastian Lyon Parra', 'sebastian.lyon@ventas.cl', v_password_hash, 'AgenteVentas', true
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'sebastian.lyon@ventas.cl')
    RETURNING id INTO v_agente2;

    INSERT INTO users (id, rut, name, email, password_hash, role, active)
    SELECT gen_random_uuid(), '36.333.444-5', 'Francisca Irarrazabal Cox', 'francisca.irarrazabal@ventas.cl', v_password_hash, 'AgenteVentas', true
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'francisca.irarrazabal@ventas.cl')
    RETURNING id INTO v_agente3;

    -- ===================================================================
    -- 1.6 EQUIPO DE ADMISION
    -- ===================================================================
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '37.444.555-6', 'Paulina Riquelme Jara', 'paulina.riquelme@colegio.cl', v_password_hash, 'Admision', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'paulina.riquelme@colegio.cl')
    RETURNING id INTO v_admision1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '38.555.666-7', 'Matias Cerda Leiva', 'matias.cerda@colegio.cl', v_password_hash, 'Admision', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'matias.cerda@colegio.cl')
    RETURNING id INTO v_admision2;

    -- ===================================================================
    -- 2. PROFESORES POR ASIGNATURA
    -- ===================================================================
    -- Lenguaje y Comunicacion (1-6 Basico)
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '12.345.678-9', 'Maria Gonzalez Rojas', 'maria.gonzalez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'maria.gonzalez@colegio.cl')
    RETURNING id INTO v_prof_len1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '39.666.777-8', 'Claudia Verdugo Soto', 'claudia.verdugo@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'claudia.verdugo@colegio.cl')
    RETURNING id INTO v_prof_len2;

    -- Lengua y Literatura (7+)
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '17.890.123-4', 'Jose Ramirez Lopez', 'jose.ramirez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'jose.ramirez@colegio.cl')
    RETURNING id INTO v_prof_len3;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '44.111.222-3', 'Teresa Valdivia Cortes', 'teresa.valdivia@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'teresa.valdivia@colegio.cl')
    RETURNING id INTO v_prof_len4;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '56.333.444-5', 'Luis Arancibia Soto', 'luis.arancibia@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'luis.arancibia@colegio.cl')
    RETURNING id INTO v_prof_len5;

    -- Matematica
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '13.456.789-0', 'Carlos Munoz Soto', 'carlos.munoz@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carlos.munoz@colegio.cl')
    RETURNING id INTO v_prof_mat1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '40.777.888-9', 'Paola Martinez Vega', 'paola.martinez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'paola.martinez@colegio.cl')
    RETURNING id INTO v_prof_mat2;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '54.111.222-3', 'Mauricio Ibanez Palma', 'mauricio.ibanez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'mauricio.ibanez@colegio.cl')
    RETURNING id INTO v_prof_mat3;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '57.444.555-6', 'Gabriela Pino Vidal', 'gabriela.pino@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'gabriela.pino@colegio.cl')
    RETURNING id INTO v_prof_mat4;

    -- Ciencias Naturales
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '14.567.890-1', 'Ana Maria Torres Perez', 'ana.torres@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ana.torres@colegio.cl')
    RETURNING id INTO v_prof_cie1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '41.888.999-0', 'Hugo Salinas Bustos', 'hugo.salinas@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'hugo.salinas@colegio.cl')
    RETURNING id INTO v_prof_cie2;

    -- Historia
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '15.678.901-2', 'Pedro Martinez Vega', 'pedro.martinez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'pedro.martinez@colegio.cl')
    RETURNING id INTO v_prof_his1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '42.999.000-1', 'Ximena Rios Perez', 'ximena.rios@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ximena.rios@colegio.cl')
    RETURNING id INTO v_prof_his2;

    -- Ingles
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '16.789.012-3', 'Laura Fernandez Diaz', 'laura.fernandez@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'laura.fernandez@colegio.cl')
    RETURNING id INTO v_prof_ing1;

    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '43.000.111-2', 'Thomas Muller Krebs', 'thomas.muller@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'thomas.muller@colegio.cl')
    RETURNING id INTO v_prof_ing2;

    -- Artes Visuales
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '18.901.234-5', 'Patricia Castillo Silva', 'patricia.castillo@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'patricia.castillo@colegio.cl')
    RETURNING id INTO v_prof_art1;

    -- Musica
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '19.012.345-6', 'Roberto Vega Morales', 'roberto.vega@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'roberto.vega@colegio.cl')
    RETURNING id INTO v_prof_mus1;

    -- Educacion Fisica
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '20.123.456-7', 'Carolina Rivas Contreras', 'carolina.rivas@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'carolina.rivas@colegio.cl')
    RETURNING id INTO v_prof_efi1;

    -- Tecnologia
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '45.222.333-4', 'Cristian Molina Tapia', 'cristian.molina@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'cristian.molina@colegio.cl')
    RETURNING id INTO v_prof_tec1;

    -- Religion
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '46.333.444-5', 'Sor Elena Contreras', 'elena.contreras@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'elena.contreras@colegio.cl')
    RETURNING id INTO v_prof_rel1;

    -- Orientacion
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '47.444.555-6', 'Marcela Soto Pino', 'marcela.soto@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'marcela.soto@colegio.cl')
    RETURNING id INTO v_prof_ori1;

    -- Filosofia
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '48.555.666-7', 'Ivan Guerrero Riquelme', 'ivan.guerrero@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'ivan.guerrero@colegio.cl')
    RETURNING id INTO v_prof_fil1;

    -- Educacion Ciudadana
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '49.666.777-8', 'Camila Flores Perez', 'camila.flores@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'camila.flores@colegio.cl')
    RETURNING id INTO v_prof_ciu1;

    -- Limites, Derivadas e Integrales
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '50.777.888-9', 'Renato Leiva Farfan', 'renato.leiva@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'renato.leiva@colegio.cl')
    RETURNING id INTO v_prof_ldi1;

    -- Biologia
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '51.888.999-0', 'Alejandra Cruz Mardones', 'alejandra.cruz@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'alejandra.cruz@colegio.cl')
    RETURNING id INTO v_prof_bio1;

    -- Fisica
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '52.999.000-1', 'Jorge Tapia Lorca', 'jorge.tapia@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'jorge.tapia@colegio.cl')
    RETURNING id INTO v_prof_fis1;

    -- Quimica
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '53.000.111-2', 'Daniela Soto Catalan', 'daniela.soto2@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'daniela.soto2@colegio.cl')
    RETURNING id INTO v_prof_qui1;

    -- Lengua Indigena (Mapuzugun)
    INSERT INTO users (id, rut, name, email, password_hash, role, active, school_id)
    SELECT gen_random_uuid(), '55.222.333-4', 'Elisa Huenchullan Lefian', 'elisa.huenchullan@colegio.cl', v_password_hash, 'Profesor', true, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'elisa.huenchullan@colegio.cl')
    RETURNING id INTO v_prof_lin1;

    RAISE NOTICE 'Usuarios del sistema creados correctamente';

    -- ===================================================================
    -- 3. ASIGNAR ROLES (user_roles)
    -- ===================================================================
    INSERT INTO user_roles (id, user_id, role_id)
    SELECT gen_random_uuid(), u.id, r.id
    FROM users u, roles r
    WHERE u.role = r.name
      AND NOT EXISTS (SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id AND ur.role_id = r.id);

    RAISE NOTICE 'Roles asignados correctamente';

    -- ===================================================================
    -- 4. AGENTES CRM VENTAS
    -- ===================================================================
    INSERT INTO crm_sales_agents (id, user_id, quota_monthly, quota_quarterly, commission_rate, active)
    SELECT gen_random_uuid(), v_jefe_ventas, 30000000, 90000000, 5.00, true
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_agents WHERE user_id = v_jefe_ventas);

    INSERT INTO crm_sales_agents (id, user_id, quota_monthly, quota_quarterly, commission_rate, active)
    SELECT gen_random_uuid(), v_agente1, 15000000, 45000000, 3.50, true
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_agents WHERE user_id = v_agente1);

    INSERT INTO crm_sales_agents (id, user_id, quota_monthly, quota_quarterly, commission_rate, active)
    SELECT gen_random_uuid(), v_agente2, 15000000, 45000000, 3.50, true
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_agents WHERE user_id = v_agente2);

    INSERT INTO crm_sales_agents (id, user_id, quota_monthly, quota_quarterly, commission_rate, active)
    SELECT gen_random_uuid(), v_agente3, 20000000, 60000000, 4.00, true
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_agents WHERE user_id = v_agente3);

    RAISE NOTICE 'Agentes CRM creados correctamente';

END $$;
