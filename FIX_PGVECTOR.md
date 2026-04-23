# 🔧 Fix pgvector Extension

## ❌ Error Encontrado

```
ERROR: 42704: type "vector" does not exist
```

## ✅ Solución Aplicada

He movido la extensión `pgvector` al inicio de la migración 001, antes de crear la tabla `embeddings`.

**Cambio realizado**:
- Agregué `CREATE EXTENSION IF NOT EXISTS vector;` al inicio de `001_initial_schema.sql`

## 🚀 Qué Hacer Ahora

### Opción 1: Si aún no ejecutaste las migraciones

1. Ve a Supabase SQL Editor
2. Ejecuta las migraciones en orden:
   - `001_initial_schema.sql` (ahora incluye la extensión)
   - `003_indexes.sql`
   - `004_constraints.sql`

**Nota**: No necesitas ejecutar `002_pgvector_extension.sql` porque ya está en `001_initial_schema.sql`

### Opción 2: Si ya ejecutaste las migraciones

1. En Supabase SQL Editor, ejecuta:
```sql
DROP TABLE IF EXISTS embeddings CASCADE;
DROP TABLE IF EXISTS migrations CASCADE;
```

2. Luego ejecuta `001_initial_schema.sql` nuevamente

## ✅ Verificar que Funciona

En Supabase SQL Editor, ejecuta:
```sql
SELECT extname FROM pg_extension WHERE extname = 'vector';
```

Deberías ver `vector` en los resultados.

## 📝 Cambios Realizados

**Archivo**: `api/supabase/migrations/001_initial_schema.sql`

```sql
-- Agregado al inicio
CREATE EXTENSION IF NOT EXISTS vector;
```

## 🚀 Próximos Pasos

```bash
git add api/supabase/migrations/001_initial_schema.sql
git commit -m "Fix: Move pgvector extension to initial migration"
git push origin main
```

¡Listo! 🎉
