-- ===================================================================
-- FIX: Corregir grade_level en estudiantes y cursos nuevos
-- para que coincida con el formato "1° Básico", "I° Medio" etc.
-- ===================================================================
UPDATE students SET grade_level = '1° Básico' WHERE grade_level = '1 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '2° Básico' WHERE grade_level = '2 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '3° Básico' WHERE grade_level = '3 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '4° Básico' WHERE grade_level = '4 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '5° Básico' WHERE grade_level = '5 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '6° Básico' WHERE grade_level = '6 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '7° Básico' WHERE grade_level = '7 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = '8° Básico' WHERE grade_level = '8 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = 'I° Medio' WHERE grade_level = 'I Medio' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE students SET grade_level = 'II° Medio' WHERE grade_level = 'II Medio' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');

UPDATE courses SET grade_level = '1° Básico' WHERE grade_level = '1 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '2° Básico' WHERE grade_level = '2 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '3° Básico' WHERE grade_level = '3 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '4° Básico' WHERE grade_level = '4 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '5° Básico' WHERE grade_level = '5 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '6° Básico' WHERE grade_level = '6 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '7° Básico' WHERE grade_level = '7 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = '8° Básico' WHERE grade_level = '8 Basico' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = 'I° Medio' WHERE grade_level = 'I Medio' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET grade_level = 'II° Medio' WHERE grade_level = 'II Medio' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');

-- Fix subject names in courses too
UPDATE courses SET subject = 'Lenguaje y Comunicación' WHERE subject = 'Lenguaje y Comunicacion' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET subject = 'Matemática' WHERE subject = 'Matematica' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
UPDATE courses SET subject = 'Historia, Geografía y Cs. Sociales' WHERE subject = 'Historia y Cs. Sociales' AND school_id = (SELECT id FROM schools WHERE name = 'Colegio Predeterminado');
