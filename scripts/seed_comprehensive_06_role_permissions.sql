-- ===================================================================
-- ASIGNACION DE PERMISOS POR ROL
-- Define exactamente qué puede ver y hacer cada perfil
-- ===================================================================
-- Modo de uso: esta funcion permite activar/desactivar permisos
-- por rol de forma masiva y clara.
-- ===================================================================

DO $$
DECLARE
    -- Helpers: asigna permiso CRUD a un rol para un modulo/recurso
    v_role_id UUID;
    v_perm_id UUID;
    v_count INT;
BEGIN

    -- ===================================================================
    -- FUNCION AUXILIAR: asigna un permiso a un rol
    -- ===================================================================
    -- Parametros: role_name, module, resource, c, r, u, d
    -- Ej: ('Director', 'students', 'view', true, true, true, true)

    -- ===================================================================
    -- 1. DIRECTOR - Acceso completo a todo lo academico y administrativo
    -- ===================================================================
    v_count := 0;
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN (
        'students', 'courses', 'enrollments', 'subjects', 'grade-levels',
        'academic-years', 'classrooms', 'attendance', 'grades', 'hr',
        'finance', 'reports', 'notifications', 'agenda', 'sige', 'complaints'
    ) LOOP
        SELECT id INTO v_role_id FROM roles WHERE name = 'Director';
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;
    RAISE NOTICE 'Director: % permisos asignados', v_count;

    -- ===================================================================
    -- 2. UTP - Acceso total a lo academico, sin finanzas ni RRHH
    -- ===================================================================
    v_count := 0;
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN (
        'students', 'courses', 'enrollments', 'subjects', 'grade-levels',
        'academic-years', 'classrooms', 'attendance', 'grades',
        'reports', 'notifications', 'agenda', 'sige'
    ) LOOP
        SELECT id INTO v_role_id FROM roles WHERE name = 'UTP';
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;
    -- UTP solo lectura en HR y payroll
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN ('hr', 'payroll') LOOP
        SELECT id INTO v_role_id FROM roles WHERE name = 'UTP';
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;
    RAISE NOTICE 'UTP: % permisos asignados', v_count;

    -- ===================================================================
    -- 3. PROFESOR - Solo sus cursos, notas y asistencia
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'Profesor';

    -- Lectura de estudiantes
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'students' AND pd.resource = 'view' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Cursos (solo lectura)
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'courses' AND pd.resource = 'view' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Asistencia: crear y leer (toman asistencia)
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'attendance' AND pd.resource IN ('records', 'reports') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Notas: crear y leer (ponen notas)
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'grades' AND pd.resource IN ('view', 'create') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Categorias y periodos: solo lectura
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'grades' AND pd.resource IN ('periods', 'categories', 'reports') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Agenda: ver y crear eventos
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'agenda' AND pd.resource IN ('events', 'view') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'Profesor: % permisos asignados', v_count;

    -- ===================================================================
    -- 4. APODERADO - Solo portal de autogestion (ver sus pupilos)
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'Apoderado';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'my-portal' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Pueden ver notas de sus pupilos
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'grades' AND pd.resource = 'view' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Pueden ver asistencia de sus pupilos
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'attendance' AND pd.resource = 'reports' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Pueden ver y enviar mensajes
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'notifications' AND pd.resource = 'view' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'Apoderado: % permisos asignados', v_count;

    -- ===================================================================
    -- 5. ALUMNO - Solo ver sus propias notas y asistencia
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'Alumno';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'my-portal' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'grades' AND pd.resource = 'view' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'attendance' AND pd.resource = 'reports' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'Alumno: % permisos asignados', v_count;

    -- ===================================================================
    -- 6. ADMISION - Acceso completo al modulo de admision
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'Admision';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN ('admission', 'classrooms', 'notifications') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Pueden ver estudiantes basicos
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'students' AND pd.resource = 'view' LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'Admision: % permisos asignados', v_count;

    -- ===================================================================
    -- 7. ADMINISTRADOR - Acceso completo a todo (gestion del sistema)
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'Administrador';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module NOT IN ('sales', 'corporations') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'Administrador: % permisos asignados', v_count;

    -- ===================================================================
    -- 8. SOSTENEDOR - Visión corporativa, sin operaciones diarias
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'Sostenedor';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN (
        'corporations', 'reports', 'finance', 'sige', 'audit', 'config',
        'users', 'roles', 'students', 'courses', 'enrollments'
    ) LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    -- Solo lectura en academicos
    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN ('grades', 'attendance', 'subjects', 'hr', 'payroll') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, true, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'Sostenedor: % permisos asignados', v_count;

    -- ===================================================================
    -- 9. JEFE VENTAS - CRM ventas completo + reportes
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'JefeVentas';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module IN ('sales', 'reports', 'corporations') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'JefeVentas: % permisos asignados', v_count;

    -- ===================================================================
    -- 10. AGENTE VENTAS - Solo CRM, ver y crear sus prospectos
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'AgenteVentas';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'sales' AND pd.resource IN ('view', 'create') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd WHERE pd.module = 'sales' AND pd.resource IN ('edit', 'delete', 'assign', 'contract', 'activate') LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, false, false, false, false
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'AgenteVentas: % permisos asignados', v_count;

    -- ===================================================================
    -- 11. GERENTE GENERAL - Full access (ya lo tiene por codigo, pero semantico)
    -- ===================================================================
    v_count := 0;
    SELECT id INTO v_role_id FROM roles WHERE name = 'GerenteGeneral';

    FOR v_perm_id IN SELECT pd.id FROM permission_definitions pd LOOP
        INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete)
        SELECT gen_random_uuid(), v_role_id, v_perm_id, true, true, true, true
        WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission_id = v_perm_id);
        v_count := v_count + 1;
    END LOOP;

    RAISE NOTICE 'GerenteGeneral: % permisos asignados (full access)', v_count;

    RAISE NOTICE '============================================';
    RAISE NOTICE 'PERMISOS POR ROL ASIGNADOS CORRECTAMENTE';
    RAISE NOTICE '============================================';
END $$;
