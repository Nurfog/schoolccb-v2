-- ===================================================================
-- SEED PARTE 2: CRM Ventas + Admision de Alumnos
-- ===================================================================
DO $$
DECLARE
    v_sstage_nuevo UUID := '10000000-0000-0000-0000-000000000001';
    v_sstage_contactado UUID := '10000000-0000-0000-0000-000000000002';
    v_sstage_requisitos UUID := '10000000-0000-0000-0000-000000000003';
    v_sstage_propuesta UUID := '10000000-0000-0000-0000-000000000004';
    v_sstage_negociacion UUID := '10000000-0000-0000-0000-000000000005';
    v_sstage_contrato UUID := '10000000-0000-0000-0000-000000000006';
    v_sstage_ganado UUID := '10000000-0000-0000-0000-000000000007';
    v_sstage_perdido UUID := '10000000-0000-0000-0000-000000000008';

    v_astage_contacto UUID; v_astage_tour UUID; v_astage_eval UUID;
    v_astage_doc UUID; v_astage_aceptado UUID; v_astage_matriculado UUID;

    v_agente1 UUID; v_agente2 UUID; v_agente3 UUID;
    v_admision1 UUID; v_admision2 UUID;
BEGIN
    SELECT id INTO v_agente1 FROM users WHERE email = 'valentina.aguirre@ventas.cl';
    SELECT id INTO v_agente2 FROM users WHERE email = 'sebastian.lyon@ventas.cl';
    SELECT id INTO v_agente3 FROM users WHERE email = 'francisca.irarrazabal@ventas.cl';
    SELECT id INTO v_admision1 FROM users WHERE email = 'paulina.riquelme@colegio.cl';
    SELECT id INTO v_admision2 FROM users WHERE email = 'matias.cerda@colegio.cl';

    SELECT id INTO v_astage_contacto FROM pipeline_stages WHERE name = 'Primer Contacto';
    SELECT id INTO v_astage_tour FROM pipeline_stages WHERE name = 'Tour Escolar';
    SELECT id INTO v_astage_eval FROM pipeline_stages WHERE name = 'Evaluacion';
    SELECT id INTO v_astage_doc FROM pipeline_stages WHERE name = 'Documentacion';
    SELECT id INTO v_astage_aceptado FROM pipeline_stages WHERE name = 'Aceptado';
    SELECT id INTO v_astage_matriculado FROM pipeline_stages WHERE name = 'Matriculado';

    -- ===================================================================
    -- 5. CRM VENTAS - PROSPECTOS EN DISTINTAS ETAPAS
    -- ===================================================================
    -- Nuevo
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Marcela', 'Vasquez', '11.111.111-1', 'marcela.vasquez@colegionuevo.cl', '+56 9 1111 1111', 'Colegio Nuevo Horizonte', 'Sostenedor', 'web', v_sstage_nuevo, v_agente1, 199900, 'Interesado en plan Corporativo'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'marcela.vasquez@colegionuevo.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Roberto', 'Lopez', '12.222.333-4', 'roberto.lopez@colegioluz.cl', '+56 9 2222 3333', 'Colegio Luz y Saber', 'Director', 'referido', v_sstage_nuevo, v_agente1, 99900, 'Recomendado por otro cliente'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'roberto.lopez@colegioluz.cl');

    -- Contactado
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Andrea', 'Martinez', '13.333.444-5', 'andrea.martinez@colegioalba.cl', '+56 9 3333 4444', 'Colegio Alba del Valle', 'Administrador', 'llamada', v_sstage_contactado, v_agente2, 49900, 'Se contacto via telefonica, agendada demo'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'andrea.martinez@colegioalba.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Jose', 'Ramirez', '14.444.555-6', 'jose.ramirez@liceolaserena.cl', '+56 9 4444 5555', 'Liceo La Serena', 'UTP', 'email', v_sstage_contactado, v_agente2, 99900, 'Solicito informacion por email'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'jose.ramirez@liceolaserena.cl');

    -- Requisitos
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, requirements, notes)
    SELECT gen_random_uuid(), 'Carolina', 'Castro', '15.555.666-7', 'carolina.castro@colegiosol.cl', '+56 9 5555 6666', 'Colegio Sol Naciente', 'Sostenedor', 'web', v_sstage_requisitos, v_agente2, 199900, '{"estudiantes": 800, "sedes": 2, "modulos_deseados": ["students", "grades", "hr", "finance", "sige"]}', 'Requiere solucion multi-sede con SIGE'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'carolina.castro@colegiosol.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, requirements, notes)
    SELECT gen_random_uuid(), 'Felipe', 'Torres', '16.666.777-8', 'felipe.torres@colegioandes.cl', '+56 9 6666 7777', 'Colegio Los Andes (Independiente)', 'Director', 'feria', v_sstage_requisitos, v_agente3, 99900, '{"estudiantes": 350, "sedes": 1, "modulos_deseados": ["students", "grades", "hr", "attendance"]}', 'Contacto de feria educativa'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'felipe.torres@colegioandes.cl');

    -- Propuesta
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Maria', 'Paz', '17.777.888-9', 'maria.paz@colegiomar.cl', '+56 9 7777 8888', 'Colegio Mar del Sur', 'Sostenedor', 'web', v_sstage_propuesta, v_agente1, 199900, 'Propuesta Corporativa enviada'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'maria.paz@colegiomar.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Pablo', 'Herrera', '18.888.999-0', 'pablo.herrera@colegioverde.cl', '+56 9 8888 9999', 'Colegio Verde Esperanza', 'Director', 'referido', v_sstage_propuesta, v_agente1, 99900, 'Propuesta Profesional enviada'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'pablo.herrera@colegioverde.cl');

    -- Negociacion
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Angela', 'Munoz', '19.999.000-1', 'angela.munoz@liceoalto.cl', '+56 9 9999 0000', 'Liceo Alto Rendimiento', 'UTP', 'email', v_sstage_negociacion, v_agente3, 49900, 'Negociando descuento por volumen, contraoferta enviada'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'angela.munoz@liceoalto.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Gonzalo', 'Paredes', '20.000.111-2', 'gonzalo.paredes@colegionorte.cl', '+56 9 0000 1111', 'Colegio del Norte', 'Sostenedor', 'llamada', v_sstage_negociacion, v_agente2, 199900, 'Definiendo ultimos detalles del contrato'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'gonzalo.paredes@colegionorte.cl');

    -- Contrato
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Daniel', 'Rivas', '21.111.222-3', 'daniel.rivas@colegiosur.cl', '+56 9 1111 2222', 'Colegio Sur Austral', 'Administrador', 'web', v_sstage_contrato, v_agente1, 99900, 'Contrato en revision legal'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'daniel.rivas@colegiosur.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Valeria', 'Bravo', '22.222.333-4', 'valeria.bravo@institutosol.cl', '+56 9 2222 3333', 'Instituto Sol Oriente', 'Director', 'feria', v_sstage_contrato, v_agente3, 199900, 'Contrato listo para firma digital'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'valeria.bravo@institutosol.cl');

    -- Cerrado Ganado
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Claudio', 'Espinoza', '23.333.444-5', 'claudio.espinoza@colegioazul.cl', '+56 9 3333 4444', 'Colegio Azul Profundo', 'Sostenedor', 'web', v_sstage_ganado, v_agente1, 199900, 'Cliente activo desde marzo 2026'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'claudio.espinoza@colegioazul.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Margarita', 'Fuentes', '24.444.555-6', 'margarita.fuentes@colegiosol.cl', '+56 9 4444 5555', 'Colegio Sol del Pacifico', 'Director', 'referido', v_sstage_ganado, v_agente2, 99900, 'Cliente activo con plan Profesional'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'margarita.fuentes@colegiosol.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Raul', 'Cardenas', '25.555.666-7', 'raul.cardenas@liceomaule.cl', '+56 9 5555 6666', 'Liceo del Maule', 'UTP', 'email', v_sstage_ganado, v_agente3, 49900, 'Cliente activo con plan Basico'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'raul.cardenas@liceomaule.cl');

    -- Cerrado Perdido
    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Eduardo', 'Silva', '26.666.777-8', 'eduardo.silva@otrosistema.cl', '+56 9 6666 7777', 'Colegio Puerto Seguro', 'Sostenedor', 'web', v_sstage_perdido, v_agente1, 99900, 'Perdido - Eligieron otro proveedor'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'eduardo.silva@otrosistema.cl');

    INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position, source, current_stage_id, assigned_to, estimated_value, notes)
    SELECT gen_random_uuid(), 'Loreto', 'Gallardo', '27.777.888-9', 'loreto.gallardo@colegiomont.cl', '+56 9 7777 8888', 'Colegio Monte Alto', 'Director', 'llamada', v_sstage_perdido, v_agente2, 49900, 'Perdido - Sin presupuesto'
    WHERE NOT EXISTS (SELECT 1 FROM crm_sales_prospects WHERE email = 'loreto.gallardo@colegiomont.cl');

    RAISE NOTICE 'CRM Ventas - Prospectos creados correctamente';

    -- ===================================================================
    -- 6. ACTIVIDADES CRM
    -- ===================================================================
    INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by)
    SELECT gen_random_uuid(), cp.id, 'llamada', 'Llamada de seguimiento', 'Primera llamada de contacto, interesado en demo', NOW() - INTERVAL '3 days', true, v_agente1
    FROM crm_sales_prospects cp WHERE cp.email = 'marcela.vasquez@colegionuevo.cl'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_activities WHERE prospect_id = cp.id AND subject = 'Llamada de seguimiento');

    INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by)
    SELECT gen_random_uuid(), cp.id, 'demo', 'Demo programada', 'Demo del plan Corporativo con equipo directivo', NOW() + INTERVAL '5 days', false, v_agente1
    FROM crm_sales_prospects cp WHERE cp.email = 'andrea.martinez@colegioalba.cl'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_activities WHERE prospect_id = cp.id AND subject = 'Demo programada');

    INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by)
    SELECT gen_random_uuid(), cp.id, 'email', 'Envio de propuesta', 'Propuesta comercial enviada con plan Corporativo', NOW() - INTERVAL '1 day', true, v_agente1
    FROM crm_sales_prospects cp WHERE cp.email = 'maria.paz@colegiomar.cl'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_activities WHERE prospect_id = cp.id AND subject = 'Envio de propuesta');

    INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by)
    SELECT gen_random_uuid(), cp.id, 'demo', 'Demo tecnica', 'Demo avanzada con modulo SIGE y RRHH', NOW() + INTERVAL '2 days', false, v_agente1
    FROM crm_sales_prospects cp WHERE cp.email = 'carolina.castro@colegiosol.cl'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_activities WHERE prospect_id = cp.id AND subject = 'Demo tecnica');

    INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by)
    SELECT gen_random_uuid(), cp.id, 'reunion', 'Reunion de cierre', 'Ultima reunion para firma de contrato', NOW() - INTERVAL '2 days', true, v_agente3
    FROM crm_sales_prospects cp WHERE cp.email = 'valeria.bravo@institutosol.cl'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_activities WHERE prospect_id = cp.id AND subject = 'Reunion de cierre');

    -- ===================================================================
    -- 7. CONTRATOS Y PROPUESTAS CRM
    -- ===================================================================
    INSERT INTO crm_sales_contracts (id, prospect_id, plan_id, modules, total_value, discount, status, signed_at, verified_at, activated_at)
    SELECT gen_random_uuid(), cp.id, lp.id, '["dashboard","students","courses","enrollments","subjects","grade-levels","academic-years","classrooms","attendance","grades","hr","payroll","my-portal","finance","admission","reports","notifications","agenda","sige","corporations","complaints"]', 199900, 0, 'active', NOW() - INTERVAL '60 days', NOW() - INTERVAL '58 days', NOW() - INTERVAL '58 days'
    FROM crm_sales_prospects cp, license_plans lp
    WHERE cp.email = 'claudio.espinoza@colegioazul.cl' AND lp.name = 'Corporativo'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_contracts WHERE prospect_id = cp.id);

    INSERT INTO crm_sales_contracts (id, prospect_id, plan_id, modules, total_value, discount, status, signed_at, verified_at, activated_at)
    SELECT gen_random_uuid(), cp.id, lp.id, '["dashboard","students","courses","enrollments","subjects","grade-levels","academic-years","classrooms","attendance","grades","hr","payroll","my-portal","finance","admission","reports","notifications","agenda"]', 99900, 10000, 'active', NOW() - INTERVAL '45 days', NOW() - INTERVAL '43 days', NOW() - INTERVAL '43 days'
    FROM crm_sales_prospects cp, license_plans lp
    WHERE cp.email = 'margarita.fuentes@colegiosol.cl' AND lp.name = 'Profesional'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_contracts WHERE prospect_id = cp.id);

    INSERT INTO crm_sales_contracts (id, prospect_id, plan_id, modules, total_value, discount, status, signed_at, verified_at, activated_at)
    SELECT gen_random_uuid(), cp.id, lp.id, '["dashboard","students","courses","enrollments","attendance","grades"]', 49900, 0, 'active', NOW() - INTERVAL '30 days', NOW() - INTERVAL '28 days', NOW() - INTERVAL '28 days'
    FROM crm_sales_prospects cp, license_plans lp
    WHERE cp.email = 'raul.cardenas@liceomaule.cl' AND lp.name = 'Basico'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_contracts WHERE prospect_id = cp.id);

    INSERT INTO crm_sales_proposals (id, prospect_id, plan_id, modules, total_value, discount, version, status, created_by)
    SELECT gen_random_uuid(), cp.id, lp.id, '["dashboard","students","courses","enrollments","subjects","grade-levels","academic-years","classrooms","attendance","grades","hr","payroll","my-portal","finance","admission","reports","notifications","agenda","sige","corporations","complaints"]', 199900, 0, 1, 'enviada', v_agente1
    FROM crm_sales_prospects cp, license_plans lp
    WHERE cp.email = 'maria.paz@colegiomar.cl' AND lp.name = 'Corporativo'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_proposals WHERE prospect_id = cp.id AND version = 1);

    INSERT INTO crm_sales_proposals (id, prospect_id, plan_id, modules, total_value, discount, version, status, created_by)
    SELECT gen_random_uuid(), cp.id, lp.id, '["dashboard","students","courses","enrollments","subjects","grade-levels","academic-years","classrooms","attendance","grades","hr","payroll","my-portal","finance","admission","reports","notifications","agenda"]', 99900, 5000, 1, 'enviada', v_agente1
    FROM crm_sales_prospects cp, license_plans lp
    WHERE cp.email = 'pablo.herrera@colegioverde.cl' AND lp.name = 'Profesional'
    AND NOT EXISTS (SELECT 1 FROM crm_sales_proposals WHERE prospect_id = cp.id AND version = 1);

    SELECT id INTO v_agente1 FROM users WHERE email = 'valentina.aguirre@ventas.cl';
    SELECT id INTO v_agente2 FROM users WHERE email = 'sebastian.lyon@ventas.cl';
    SELECT id INTO v_agente3 FROM users WHERE email = 'francisca.irarrazabal@ventas.cl';

    INSERT INTO crm_sales_goals (id, agent_id, goal_type, target_amount, target_count, period_start, period_end, achieved_amount, achieved_count)
    SELECT gen_random_uuid(), sa.id, 'monthly', 15000000, 3, '2026-05-01', '2026-05-31', 5000000, 1
    FROM crm_sales_agents sa WHERE sa.user_id = v_agente1
    AND NOT EXISTS (SELECT 1 FROM crm_sales_goals WHERE agent_id = sa.id AND goal_type = 'monthly');

    INSERT INTO crm_sales_goals (id, agent_id, goal_type, target_amount, target_count, period_start, period_end, achieved_amount, achieved_count)
    SELECT gen_random_uuid(), sa.id, 'monthly', 15000000, 3, '2026-05-01', '2026-05-31', 8000000, 2
    FROM crm_sales_agents sa WHERE sa.user_id = v_agente2
    AND NOT EXISTS (SELECT 1 FROM crm_sales_goals WHERE agent_id = sa.id AND goal_type = 'monthly');

    INSERT INTO crm_sales_goals (id, agent_id, goal_type, target_amount, target_count, period_start, period_end, achieved_amount, achieved_count)
    SELECT gen_random_uuid(), sa.id, 'monthly', 20000000, 4, '2026-05-01', '2026-05-31', 12000000, 2
    FROM crm_sales_agents sa WHERE sa.user_id = v_agente3
    AND NOT EXISTS (SELECT 1 FROM crm_sales_goals WHERE agent_id = sa.id AND goal_type = 'monthly');

    RAISE NOTICE 'CRM Ventas - Actividades, Contratos y Metas creados';

    -- ===================================================================
    -- 8. ADMISION - PROSPECTOS EN DISTINTAS ETAPAS
    -- ===================================================================
    -- Primer Contacto
    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Isidora', 'Cisternas', '28.888.999-0', 'isidora.cisternas@padres.cl', '+56 9 8888 9900', v_astage_contacto, v_admision1, 'web', 'Solicito informacion para 1 Basico'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '28.888.999-0');

    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Julian', 'Munoz', '29.999.000-1', 'julian.munoz@padres.cl', '+56 9 9999 0011', v_astage_contacto, v_admision1, 'referido', 'Recomendado por apoderado actual'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '29.999.000-1');

    -- Tour Escolar
    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Amanda', 'Solar', '30.000.111-2', 'amanda.solar@padres.cl', '+56 9 0000 1122', v_astage_tour, v_admision1, 'web', 'Tour agendado para proxima semana, Pre-kinder'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '30.000.111-2');

    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Benjamin', 'Herrera', '31.111.222-3', 'benjamin.herrera@padres.cl', '+56 9 1111 2233', v_astage_tour, v_admision2, 'llamada', 'Tour realizado, muy interesado en 5 Basico'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '31.111.222-3');

    -- Evaluacion
    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Emilia', 'Valdes', '32.222.333-4', 'emilia.valdes@padres.cl', '+56 9 2222 3344', v_astage_eval, v_admision2, 'web', 'Evaluacion diagnostica agendada para 3 Basico'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '32.222.333-4');

    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Mateo', 'Cruz', '33.333.444-5', 'mateo.cruz@padres.cl', '+56 9 3333 4455', v_astage_eval, v_admision1, 'referido', 'Evaluacion completada, resultados favorables para 7 Basico'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '33.333.444-5');

    -- Documentacion
    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Antonia', 'Pena', '34.444.555-6', 'antonia.pena@padres.cl', '+56 9 4444 5566', v_astage_doc, v_admision1, 'web', 'Entregando documentos para 1 Basico 2026'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '34.444.555-6');

    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Gaspar', 'Ibarra', '35.555.666-7', 'gaspar.ibarra@padres.cl', '+56 9 5555 6677', v_astage_doc, v_admision2, 'email', 'Documentacion completa para II Medio, solo falta certificado'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '35.555.666-7');

    -- Aceptado
    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Josefa', 'Arancibia', '36.666.777-8', 'josefa.arancibia@padres.cl', '+56 9 6666 7788', v_astage_aceptado, v_admision1, 'web', 'Aceptada para 4 Basico A, pendiente matricula'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '36.666.777-8');

    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Vicente', 'Molina', '37.777.888-9', 'vicente.molina@padres.cl', '+56 9 7777 8899', v_astage_aceptado, v_admision2, 'llamada', 'Aceptado para 6 Basico B, matriculandose esta semana'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '37.777.888-9');

    -- Matriculado
    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Florencia', 'Tapia', '38.888.999-0', 'florencia.tapia@padres.cl', '+56 9 8888 9900', v_astage_matriculado, v_admision1, 'web', 'Matriculada en 1 Basico A - 2026'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '38.888.999-0');

    INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes)
    SELECT gen_random_uuid(), 'Agustin', 'Rivas', '39.999.000-1', 'agustin.rivas@padres.cl', '+56 9 9999 0011', v_astage_matriculado, v_admision2, 'referido', 'Matriculado en 8 Basico A - 2026'
    WHERE NOT EXISTS (SELECT 1 FROM prospects WHERE rut = '39.999.000-1');

    RAISE NOTICE 'Admision - Prospectos creados correctamente';
END $$;
