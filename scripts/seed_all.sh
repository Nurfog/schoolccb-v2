#!/bin/bash
# ============================================================
# Script para ejecutar todos los seeders comprehensive
# Ejecutar SOLO despues de que los servicios hayan iniciado
# y creado las tablas automaticamente
# ============================================================
set -e

DB_URL="${DATABASE_URL:-postgres://schoolccb:schoolccb@localhost:5432/schoolccb}"

echo "============================================"
echo "Ejecutando seeders comprehensive..."
echo "============================================"

echo ""
echo "1/6 - Usuarios, Roles y Agentes CRM..."
psql "$DB_URL" -f seed_comprehensive_01_users_roles.sql

echo ""
echo "2/6 - CRM Ventas y Admision de Alumnos..."
psql "$DB_URL" -f seed_comprehensive_02_crm_admission.sql

echo ""
echo "3/6 - Apoderados y Estudiantes..."
psql "$DB_URL" -f seed_comprehensive_03_students.sql

echo ""
echo "4/6 - Cursos, Matriculas y Empleados RRHH..."
psql "$DB_URL" -f seed_comprehensive_04_courses_employees.sql

echo ""
echo "5/6 - Correccion de tildes y formatos..."
psql "$DB_URL" -f seed_comprehensive_05_fix_gradelevels.sql

echo ""
echo "6/6 - Asignacion de Permisos por Rol..."
psql "$DB_URL" -f seed_comprehensive_06_role_permissions.sql

echo ""
echo "============================================"
echo "SEED COMPREHENSIVO COMPLETADO!"
echo "============================================"
echo ""
echo "Credenciales de acceso (password: test123):"
echo "============================================"
echo ""
echo "=== PLATAFORMA ESCOLAR (SchoolCCB) ==="
echo "Director:       daniela.soto@colegio.cl"
echo "UTP:            andres.nunez@colegio.cl"
echo "Administrador:  rodrigo.fuentes@colegio.cl"
echo "Sostenedor:     fernando.hurtado@corporacion.cl"
echo "Admision:       paulina.riquelme@colegio.cl"
echo ""
echo "=== PROFESORES POR ASIGNATURA ==="
echo "Lenguaje:       maria.gonzalez@colegio.cl"
echo "Matematica:     carlos.munoz@colegio.cl"
echo "Ciencias:       ana.torres@colegio.cl"
echo "Historia:       pedro.martinez@colegio.cl"
echo "Ingles:         laura.fernandez@colegio.cl"
echo "Artes:          patricia.castillo@colegio.cl"
echo "Musica:         roberto.vega@colegio.cl"
echo "Ed. Fisica:     carolina.rivas@colegio.cl"
echo ""
echo "=== CRM VENTAS ==="
echo "Jefe Ventas:    cristobal.padilla@ventas.cl"
echo "Agente 1:       valentina.aguirre@ventas.cl"
echo "Agente 2:       sebastian.lyon@ventas.cl"
echo "Agente 3:       francisca.irarrazabal@ventas.cl"
echo ""
echo "=== APODERADOS ==="
echo "Apoderado:      [nombre].gonzalez@apoderados.cl"
echo ""
echo "============================================"
