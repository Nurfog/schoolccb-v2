# Skills del Currículum Nacional de Chile

Skills organizadas por nivel y asignatura para consulta RAG sobre el currículum chileno.

## Estructura

```
cn/
├── normativa/                    # Decretos, leyes, resoluciones
├── bases_curriculares/           # Bases curriculares de todos los niveles
├── planes_estudio/               # Planes de estudio oficiales
├── programas_estudio/            # Programas de estudio por asignatura
├── transversales/                # Skills transversales (OAT, OAG, etc.)
├── {nivel}-{curso}-{asignatura}/ # Skills específicas por curso+asignatura
│   ├── SKILL.md
│   └── kb.json                   # Chunks con embeddings
└── README.md
```

## Niveles principales

- `1_6_Basico/` — 1° a 6° Básico (6 cursos × ~12 asignaturas)
- `7_8_Basico_1_2_Medio/` — 7° Básico a 2° Medio (4 cursos × ~11 asignaturas)
- `3_4_Medio/` — 3° y 4° Medio FG/HC
- `3_4_Medio_TP/` — Formación Técnico-Profesional
- `Educacion_Parvularia/` — Sala Cuna, Nivel Medio, Nivel Transición
- `EPJA/` — Educación de Personas Jóvenes y Adultas
- `Lengua_Indigena_7_8/` — Lengua Indígena
- `Pueblos_Originarios_1_6/` — Pueblos Originarios Ancestrales

## Procesamiento

Para procesar los PDFs y generar embeddings + skills:

```bash
cd docs/cn
python3 process_rag.py
```

Requiere: `pdftotext`, `OLLAMA_URL` apuntando a Ollama local con `nomic-embed-text` y `qwen3.5:4b`.

## Fuente

MINEDUC - Unidad de Currículum y Evaluación
https://www.curriculumnacional.cl
