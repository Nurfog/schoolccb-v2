-- ===================================================================
-- SEED PARTE 4: Cursos, Matriculas y Empleados RRHH
-- ===================================================================
DO $$
DECLARE
    v_school_pred UUID;
    v_dir1 UUID; v_utp1 UUID; v_admin_school UUID;

    v_prof_len1 UUID; v_prof_len2 UUID; v_prof_len3 UUID; v_prof_len4 UUID; v_prof_len5 UUID;
    v_prof_mat1 UUID; v_prof_mat2 UUID; v_prof_mat3 UUID; v_prof_mat4 UUID;
    v_prof_cie1 UUID; v_prof_cie2 UUID;
    v_prof_his1 UUID; v_prof_his2 UUID;
    v_prof_ing1 UUID; v_prof_ing2 UUID;
    v_prof_art1 UUID; v_prof_mus1 UUID; v_prof_efi1 UUID; v_prof_tec1 UUID;
    v_prof_rel1 UUID; v_prof_ori1 UUID;
    v_prof_fil1 UUID; v_prof_ciu1 UUID;
    v_prof_ldi1 UUID; v_prof_bio1 UUID; v_prof_fis1 UUID; v_prof_qui1 UUID;
    v_prof_lin1 UUID;

    v_emp_dir1 UUID; v_emp_utp1 UUID;
BEGIN
    SELECT id INTO v_school_pred FROM schools WHERE name = 'Colegio Predeterminado';

    SELECT id INTO v_dir1 FROM users WHERE email = 'daniela.soto@colegio.cl';
    SELECT id INTO v_utp1 FROM users WHERE email = 'andres.nunez@colegio.cl';
    SELECT id INTO v_admin_school FROM users WHERE email = 'rodrigo.fuentes@colegio.cl';

    SELECT id INTO v_prof_len1 FROM users WHERE email = 'maria.gonzalez@colegio.cl';
    SELECT id INTO v_prof_len2 FROM users WHERE email = 'claudia.verdugo@colegio.cl';
    SELECT id INTO v_prof_len3 FROM users WHERE email = 'jose.ramirez@colegio.cl';
    SELECT id INTO v_prof_len4 FROM users WHERE email = 'teresa.valdivia@colegio.cl';
    SELECT id INTO v_prof_len5 FROM users WHERE email = 'luis.arancibia@colegio.cl';
    SELECT id INTO v_prof_mat1 FROM users WHERE email = 'carlos.munoz@colegio.cl';
    SELECT id INTO v_prof_mat2 FROM users WHERE email = 'paola.martinez@colegio.cl';
    SELECT id INTO v_prof_mat3 FROM users WHERE email = 'mauricio.ibanez@colegio.cl';
    SELECT id INTO v_prof_mat4 FROM users WHERE email = 'gabriela.pino@colegio.cl';
    SELECT id INTO v_prof_cie1 FROM users WHERE email = 'ana.torres@colegio.cl';
    SELECT id INTO v_prof_cie2 FROM users WHERE email = 'hugo.salinas@colegio.cl';
    SELECT id INTO v_prof_his1 FROM users WHERE email = 'pedro.martinez@colegio.cl';
    SELECT id INTO v_prof_his2 FROM users WHERE email = 'ximena.rios@colegio.cl';
    SELECT id INTO v_prof_ing1 FROM users WHERE email = 'laura.fernandez@colegio.cl';
    SELECT id INTO v_prof_ing2 FROM users WHERE email = 'thomas.muller@colegio.cl';
    SELECT id INTO v_prof_art1 FROM users WHERE email = 'patricia.castillo@colegio.cl';
    SELECT id INTO v_prof_mus1 FROM users WHERE email = 'roberto.vega@colegio.cl';
    SELECT id INTO v_prof_efi1 FROM users WHERE email = 'carolina.rivas@colegio.cl';
    SELECT id INTO v_prof_tec1 FROM users WHERE email = 'cristian.molina@colegio.cl';
    SELECT id INTO v_prof_rel1 FROM users WHERE email = 'elena.contreras@colegio.cl';
    SELECT id INTO v_prof_ori1 FROM users WHERE email = 'marcela.soto@colegio.cl';
    SELECT id INTO v_prof_fil1 FROM users WHERE email = 'ivan.guerrero@colegio.cl';
    SELECT id INTO v_prof_ciu1 FROM users WHERE email = 'camila.flores@colegio.cl';
    SELECT id INTO v_prof_ldi1 FROM users WHERE email = 'renato.leiva@colegio.cl';
    SELECT id INTO v_prof_bio1 FROM users WHERE email = 'alejandra.cruz@colegio.cl';
    SELECT id INTO v_prof_fis1 FROM users WHERE email = 'jorge.tapia@colegio.cl';
    SELECT id INTO v_prof_qui1 FROM users WHERE email = 'daniela.soto2@colegio.cl';
    SELECT id INTO v_prof_lin1 FROM users WHERE email = 'elisa.huenchullan@colegio.cl';

    -- ===================================================================
    -- CURSOS ADICIONALES
    -- ===================================================================
    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '1 Basico A', 'Matematica', '1 Basico', 'A', v_prof_mat1, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '1 Basico A' AND school_id = v_school_pred AND subject = 'Matematica');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '2 Basico A', 'Lenguaje y Comunicacion', '2 Basico', 'A', v_prof_len1, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '2 Basico A' AND school_id = v_school_pred AND subject = 'Lenguaje y Comunicacion');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '3 Basico B', 'Ciencias Naturales', '3 Basico', 'B', v_prof_cie1, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '3 Basico B' AND school_id = v_school_pred AND subject = 'Ciencias Naturales');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '4 Basico A', 'Historia y Cs. Sociales', '4 Basico', 'A', v_prof_his1, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '4 Basico A' AND school_id = v_school_pred AND subject = 'Historia y Cs. Sociales');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '5 Basico C', 'Ciencias Naturales', '5 Basico', 'C', v_prof_cie2, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '5 Basico C' AND school_id = v_school_pred AND subject = 'Ciencias Naturales');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '6 Basico A', 'Matematica', '6 Basico', 'A', v_prof_mat1, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '6 Basico A' AND school_id = v_school_pred AND subject = 'Matematica');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '7 Basico A', 'Lengua y Literatura', '7 Basico', 'A', v_prof_len3, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '7 Basico A' AND school_id = v_school_pred AND subject = 'Lengua y Literatura');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '7 Basico B', 'Matematica', '7 Basico', 'B', v_prof_mat2, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '7 Basico B' AND school_id = v_school_pred AND subject = 'Matematica');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), '8 Basico A', 'Matematica', '8 Basico', 'A', v_prof_mat2, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = '8 Basico A' AND school_id = v_school_pred AND subject = 'Matematica');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), 'I Medio A', 'Lengua y Literatura', 'I Medio', 'A', v_prof_len3, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = 'I Medio A' AND school_id = v_school_pred AND subject = 'Lengua y Literatura');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), 'I Medio B', 'Lengua y Literatura', 'I Medio', 'B', v_prof_len4, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = 'I Medio B' AND school_id = v_school_pred AND subject = 'Lengua y Literatura');

    INSERT INTO courses (id, name, subject, grade_level, section, teacher_id, plan, max_students, academic_year, school_id)
    SELECT gen_random_uuid(), 'II Medio A', 'Matematica', 'II Medio', 'A', v_prof_mat1, NULL, 35, 2026, v_school_pred
    WHERE NOT EXISTS (SELECT 1 FROM courses WHERE name = 'II Medio A' AND school_id = v_school_pred AND subject = 'Matematica');

    RAISE NOTICE 'Cursos adicionales creados';

    -- ===================================================================
    -- MATRICULAS
    -- ===================================================================
    INSERT INTO enrollments (id, student_id, course_id, year, active, school_id)
    SELECT gen_random_uuid(), s.id, c.id, 2026, true, v_school_pred
    FROM students s, courses c
    WHERE s.school_id = v_school_pred AND c.school_id = v_school_pred
      AND s.grade_level = c.grade_level
      AND s.section = c.section
      AND NOT EXISTS (
          SELECT 1 FROM enrollments e
          WHERE e.student_id = s.id AND e.course_id = c.id AND e.year = 2026
      );

    RAISE NOTICE 'Matriculas creadas correctamente';

    -- ===================================================================
    -- EMPLEADOS DIRECTIVOS Y ADMINISTRATIVOS
    -- ===================================================================
    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, user_id)
    SELECT gen_random_uuid(), v_school_pred, '22.345.678-9', 'Daniela', 'Soto Pizarro', 'daniela.soto@colegio.cl', '+56 9 8765 4321', 'Director(a) del Establecimiento', 'Directivo', '2020-03-01', 15.0, true, v_dir1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '22.345.678-9') RETURNING id INTO v_emp_dir1;

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '23.456.789-0', 'Andres', 'Nunez Campos', 'andres.nunez@colegio.cl', '+56 9 7654 3210', 'Jefe Unidad Tecnico Pedagogica (UTP)', 'Directivo', '2021-06-15', 15.0, true, v_emp_dir1, v_utp1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '23.456.789-0') RETURNING id INTO v_emp_utp1;

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '30.777.888-9', 'Rodrigo', 'Fuentes Maldonado', 'rodrigo.fuentes@colegio.cl', '+56 9 6543 2109', 'Administrador del Establecimiento', 'Directivo', '2022-01-10', 15.0, true, v_emp_dir1, v_admin_school
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '30.777.888-9');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id)
    SELECT gen_random_uuid(), v_school_pred, '74.567.890-1', 'Eugenio', 'Reyes Munoz', 'eugenio.reyes@colegio.cl', '+56 9 4321 0987', 'Contador General', 'Administrativo', '2021-04-01', 15.0, true, v_emp_dir1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '74.567.890-1');

    -- (other admin employees go here - simplified)
    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id)
    SELECT gen_random_uuid(), v_school_pred, '78.901.234-5', 'Carmen', 'Lizama Soto', 'carmen.lizama@colegio.cl', '+56 9 0987 6543', 'Secretaria Direccion', 'Administrativo', '2019-11-01', 18.0, true, v_emp_dir1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '78.901.234-5');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id)
    SELECT gen_random_uuid(), v_school_pred, '80.123.456-7', 'Francisca', 'Orellana Vega', 'francisca.orellana@colegio.cl', '+56 9 8765 4321', 'Psicologa', 'Asistente', '2021-05-10', 15.0, true, v_emp_utp1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '80.123.456-7');

    RAISE NOTICE 'Empleados administrativos creados';

    -- ===================================================================
    -- EMPLEADOS PROFESORES (seleccion representativa)
    -- ===================================================================
    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '12.345.678-9', 'Maria', 'Gonzalez Rojas', 'maria.gonzalez@colegio.cl', '+56 9 1111 1112', 'Profesor(a) Lenguaje y Comunicacion', 'Docente', '2020-03-01', 15.0, true, v_emp_utp1, v_prof_len1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '12.345.678-9');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '13.456.789-0', 'Carlos', 'Munoz Soto', 'carlos.munoz@colegio.cl', '+56 9 2222 2223', 'Profesor(a) Matematica', 'Docente', '2021-03-01', 15.0, true, v_emp_utp1, v_prof_mat1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '13.456.789-0');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '14.567.890-1', 'Ana Maria', 'Torres Perez', 'ana.torres@colegio.cl', '+56 9 3333 3334', 'Profesor(a) Ciencias Naturales', 'Docente', '2022-03-01', 15.0, true, v_emp_utp1, v_prof_cie1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '14.567.890-1');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '15.678.901-2', 'Pedro', 'Martinez Vega', 'pedro.martinez@colegio.cl', '+56 9 4444 4445', 'Profesor(a) Historia y Cs. Sociales', 'Docente', '2021-06-01', 15.0, true, v_emp_utp1, v_prof_his1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '15.678.901-2');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '16.789.012-3', 'Laura', 'Fernandez Diaz', 'laura.fernandez@colegio.cl', '+56 9 5555 5556', 'Profesor(a) Ingles', 'Docente', '2020-09-01', 15.0, true, v_emp_utp1, v_prof_ing1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '16.789.012-3');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '17.890.123-4', 'Jose', 'Ramirez Lopez', 'jose.ramirez@colegio.cl', '+56 9 6666 6667', 'Profesor(a) Lengua y Literatura', 'Docente', '2023-03-01', 15.0, true, v_emp_utp1, v_prof_len3
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '17.890.123-4');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '18.901.234-5', 'Patricia', 'Castillo Silva', 'patricia.castillo@colegio.cl', '+56 9 7777 7778', 'Profesor(a) Artes Visuales', 'Docente', '2021-09-01', 15.0, true, v_emp_utp1, v_prof_art1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '18.901.234-5');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '19.012.345-6', 'Roberto', 'Vega Morales', 'roberto.vega@colegio.cl', '+56 9 8888 8889', 'Profesor(a) Musica', 'Docente', '2020-06-01', 15.0, true, v_emp_utp1, v_prof_mus1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '19.012.345-6');

    INSERT INTO employees (id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id)
    SELECT gen_random_uuid(), v_school_pred, '20.123.456-7', 'Carolina', 'Rivas Contreras', 'carolina.rivas@colegio.cl', '+56 9 9999 9990', 'Profesor(a) Educacion Fisica', 'Docente', '2020-08-01', 15.0, true, v_emp_utp1, v_prof_efi1
    WHERE NOT EXISTS (SELECT 1 FROM employees WHERE rut = '20.123.456-7');

    RAISE NOTICE 'Empleados profesores creados';

    -- ===================================================================
    -- CONTRATOS
    -- ===================================================================
    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), v_emp_dir1, 'Planta', 2500000, 44, true, '2020-03-01', true
    WHERE NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = v_emp_dir1 AND active = true);

    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), v_emp_utp1, 'Planta', 1800000, 44, true, '2021-06-15', true
    WHERE NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = v_emp_utp1 AND active = true);

    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), e.id, 'Planta', 1200000, 44, true, e.hire_date, true
    FROM employees e WHERE e.rut = '12.345.678-9'
    AND NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = e.id AND active = true);

    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), e.id, 'Planta', 1200000, 44, true, e.hire_date, true
    FROM employees e WHERE e.rut = '13.456.789-0'
    AND NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = e.id AND active = true);

    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), e.id, 'Indefinido', 1100000, 44, true, e.hire_date, true
    FROM employees e WHERE e.rut = '14.567.890-1'
    AND NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = e.id AND active = true);

    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), e.id, 'Planta', 1200000, 44, true, e.hire_date, true
    FROM employees e WHERE e.rut = '15.678.901-2'
    AND NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = e.id AND active = true);

    INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, digitally_signed)
    SELECT gen_random_uuid(), e.id, 'Indefinido', 1100000, 44, true, e.hire_date, true
    FROM employees e WHERE e.rut = '16.789.012-3'
    AND NOT EXISTS (SELECT 1 FROM employee_contracts WHERE employee_id = e.id AND active = true);

    RAISE NOTICE 'Contratos creados';

    -- ===================================================================
    -- FONDOS DE PENSION Y SALUD
    -- ===================================================================
    INSERT INTO employee_pension_funds (id, employee_id, pension_fund, health_system)
    SELECT gen_random_uuid(), v_emp_dir1, 'Provida', 'Fonasa'
    WHERE NOT EXISTS (SELECT 1 FROM employee_pension_funds WHERE employee_id = v_emp_dir1);

    INSERT INTO employee_pension_funds (id, employee_id, pension_fund, health_system)
    SELECT gen_random_uuid(), v_emp_utp1, 'Habitat', 'Fonasa'
    WHERE NOT EXISTS (SELECT 1 FROM employee_pension_funds WHERE employee_id = v_emp_utp1);

    INSERT INTO employee_pension_funds (id, employee_id, pension_fund, health_system)
    SELECT gen_random_uuid(), e.id, 'Provida', 'Fonasa'
    FROM employees e WHERE e.rut = '12.345.678-9'
    AND NOT EXISTS (SELECT 1 FROM employee_pension_funds WHERE employee_id = e.id);

    INSERT INTO employee_pension_funds (id, employee_id, pension_fund, health_system)
    SELECT gen_random_uuid(), e.id, 'Habitat', 'Fonasa'
    FROM employees e WHERE e.rut = '13.456.789-0'
    AND NOT EXISTS (SELECT 1 FROM employee_pension_funds WHERE employee_id = e.id);

    RAISE NOTICE 'Fondos de pension creados';

    -- ===================================================================
    -- EVALUACIONES DOCENTES (usando IDs de usuarios, no de empleados)
    -- ===================================================================
    INSERT INTO teacher_evaluations (id, employee_id, evaluator_id, evaluation_type, score, observations, period, year)
    SELECT gen_random_uuid(), e.id, v_utp1, 'Desempeno General', 4.5, 'Buena gestion de aula, cumple con planificaciones', 'Semestre 1', 2025
    FROM employees e WHERE e.rut = '12.345.678-9'
    AND NOT EXISTS (SELECT 1 FROM teacher_evaluations WHERE employee_id = e.id AND year = 2025);

    INSERT INTO teacher_evaluations (id, employee_id, evaluator_id, evaluation_type, score, observations, period, year)
    SELECT gen_random_uuid(), e.id, v_utp1, 'Desempeno General', 5.0, 'Excelente manejo de contenidos y relacion con estudiantes', 'Semestre 1', 2025
    FROM employees e WHERE e.rut = '13.456.789-0'
    AND NOT EXISTS (SELECT 1 FROM teacher_evaluations WHERE employee_id = e.id AND year = 2025);

    INSERT INTO teacher_evaluations (id, employee_id, evaluator_id, evaluation_type, score, observations, period, year)
    SELECT gen_random_uuid(), e.id, v_utp1, 'Desempeno General', 4.0, 'Cumple con los objetivos, debe mejorar en planificacion', 'Semestre 1', 2025
    FROM employees e WHERE e.rut = '14.567.890-1'
    AND NOT EXISTS (SELECT 1 FROM teacher_evaluations WHERE employee_id = e.id AND year = 2025);

    INSERT INTO teacher_evaluations (id, employee_id, evaluator_id, evaluation_type, score, observations, period, year)
    SELECT gen_random_uuid(), e.id, v_utp1, 'Desempeno General', 3.5, 'Debe reforzar estrategias de evaluacion formativa', 'Semestre 1', 2025
    FROM employees e WHERE e.rut = '15.678.901-2'
    AND NOT EXISTS (SELECT 1 FROM teacher_evaluations WHERE employee_id = e.id AND year = 2025);

    INSERT INTO teacher_evaluations (id, employee_id, evaluator_id, evaluation_type, score, observations, period, year)
    SELECT gen_random_uuid(), e.id, v_utp1, 'Desempeno General', 4.8, 'Muy buen nivel de ingles, estudiantes motivados', 'Semestre 1', 2025
    FROM employees e WHERE e.rut = '16.789.012-3'
    AND NOT EXISTS (SELECT 1 FROM teacher_evaluations WHERE employee_id = e.id AND year = 2025);

    -- Evaluacion Director (employee_id = v_emp_dir1, evaluator NULL)
    INSERT INTO teacher_evaluations (id, employee_id, evaluator_id, evaluation_type, score, observations, period, year)
    SELECT gen_random_uuid(), v_emp_dir1, NULL, 'Evaluacion Directiva', 5.0, 'Liderazgo efectivo, gestion escolar destacada', 'Semestre 1', 2025
    WHERE NOT EXISTS (SELECT 1 FROM teacher_evaluations WHERE employee_id = v_emp_dir1 AND year = 2025);

    RAISE NOTICE 'Evaluaciones docentes creadas correctamente';
    RAISE NOTICE '============================================';
    RAISE NOTICE 'SEED PARTE 4 COMPLETADO EXITOSAMENTE';
    RAISE NOTICE '============================================';
END $$;
