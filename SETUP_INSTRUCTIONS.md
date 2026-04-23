# 🚀 Cofre Vault Platform - Setup Instructions

Tu aplicación está **100% lista para producción**. Aquí está todo lo que necesitas hacer:

## 📋 Paso 1: Preparar Supabase

1. **Crear proyecto en Supabase**
   - Ve a https://supabase.com
   - Crea un nuevo proyecto
   - Anota tu `Project URL` y `Anon Key`

2. **Habilitar pgvector**
   - En Supabase, ve a SQL Editor
   - Ejecuta: `CREATE EXTENSION IF NOT EXISTS vector;`

3. **Ejecutar migraciones SQL**
   - Ve a la carpeta `supabase/migrations/`
   - Copia el contenido de cada archivo `.sql` en orden:
     - `001_initial_schema.sql`
     - `002_pgvector_extension.sql`
     - `003_indexes.sql`
     - `004_constraints.sql`
   - Pega cada uno en SQL Editor de Supabase y ejecuta

## 🔑 Paso 2: Configurar Variables de Entorno

1. **Copiar archivo de ejemplo**
   ```bash
   cd api
   cp .env.example .env
   ```

2. **Llenar `.env` con tus credenciales**
   ```
   DATABASE_URL=postgresql://[user]:[password]@[host]:[port]/[database]
   SUPABASE_URL=https://your-project.supabase.co
   SUPABASE_KEY=your-anon-key
   GEMINI_API_KEY=your-gemini-api-key
   ELEVENLABS_API_KEY=your-elevenlabs-api-key
   EMBEDDING_MODEL=text-embedding-004
   LLM_MODEL=gemini-1.5-flash
   SIMILARITY_THRESHOLD=0.8
   NODE_ENV=production
   ```

   **Dónde obtener cada credencial:**
   - `DATABASE_URL`: Supabase > Project Settings > Database > Connection String
   - `SUPABASE_URL` y `SUPABASE_KEY`: Supabase > Project Settings > API
   - `GEMINI_API_KEY`: Google AI Studio (https://aistudio.google.com/app/apikey)
   - `ELEVENLABS_API_KEY`: ElevenLabs Dashboard (https://elevenlabs.io/app/home)

## 🌐 Paso 3: Deploy en Vercel

### Opción A: Usando Vercel CLI (Recomendado)

```bash
cd api
npm install -g vercel
vercel login
vercel deploy
```

Vercel te pedirá que confirmes la configuración. Cuando pregunte por variables de entorno, selecciona "Add environment variables" y copia las de tu `.env`.

### Opción B: Usando GitHub Integration

1. Push tu código a GitHub
2. Ve a https://vercel.com
3. Click "New Project"
4. Selecciona tu repositorio
5. En "Environment Variables", agrega todas las variables de `.env`
6. Click "Deploy"

## ✅ Paso 4: Verificar Deployment

Una vez deployado, verifica que funciona:

```bash
curl https://your-vercel-domain.vercel.app/health
```

Deberías recibir:
```json
{
  "status": "ok"
}
```

## 🧪 Paso 5: Probar la API

### 1. Crear una cuenta

```bash
curl -X POST https://your-vercel-domain.vercel.app/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123"
  }'
```

Guarda el `session_token` que recibes.

### 2. Crear un vault

```bash
curl -X POST https://your-vercel-domain.vercel.app/api/vaults \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Mi Primer Vault",
    "description": "Un vault de prueba"
  }'
```

### 3. Crear un tag

```bash
curl -X POST https://your-vercel-domain.vercel.app/api/vaults/<vault_id>/tags \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Importante",
    "color": "#FF0000"
  }'
```

### 4. Agregar contenido

```bash
curl -X POST https://your-vercel-domain.vercel.app/api/vaults/<vault_id>/content \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "content_type": "link",
    "url": "https://example.com",
    "title": "Ejemplo"
  }'
```

### 5. Chat

```bash
curl -X POST https://your-vercel-domain.vercel.app/api/vaults/<vault_id>/chat \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "¿Qué hay en este vault?"
  }'
```

## 📚 Documentación

- **README.md**: Overview del proyecto
- **API.md**: Documentación completa de endpoints
- **ARCHITECTURE.md**: Diseño del sistema
- **DEPLOYMENT.md**: Guía de deployment
- **QUICKSTART.md**: Inicio rápido

## 🛠️ Desarrollo Local

Si quieres desarrollar localmente:

```bash
cd api
npm install
npm run dev
```

El servidor correrá en `http://localhost:3000`

## 📊 Monitoreo

### Vercel Logs
- Ve a tu proyecto en Vercel
- Click "Deployments"
- Selecciona un deployment
- Click "Logs"

### Supabase Monitoring
- Ve a tu proyecto en Supabase
- Sección "Database" para ver estado de conexiones
- Sección "Logs" para ver queries

## 🔒 Seguridad

✅ Todas las credenciales están en variables de entorno (nunca en código)
✅ JWT tokens para autenticación
✅ Rate limiting en endpoints
✅ Validación de entrada
✅ CORS configurado
✅ Errores sanitizados

## 🚨 Troubleshooting

### Error de conexión a base de datos
1. Verifica que `DATABASE_URL` es correcto
2. Verifica que pgvector está habilitado en Supabase
3. Verifica que las migraciones se ejecutaron

### Error de API keys
1. Verifica que `GEMINI_API_KEY` es válido
2. Verifica que `ELEVENLABS_API_KEY` es válido
3. Verifica que tienes cuota disponible en ambas APIs

### Error 401 Unauthorized
1. Verifica que el token JWT es válido
2. Verifica que el header es: `Authorization: Bearer <token>`
3. Verifica que el token no ha expirado

## 📞 Soporte

Si tienes problemas:
1. Revisa los logs en Vercel
2. Revisa los logs en Supabase
3. Verifica que todas las variables de entorno están configuradas
4. Revisa la documentación en `api/README.md`

## 🎉 ¡Listo!

Tu aplicación está completamente lista para producción. Todos los servicios están implementados:

✅ Autenticación con JWT
✅ Gestión de vaults colaborativos
✅ Almacenamiento de contenido
✅ Transcripción de audio
✅ Embeddings con Gemini
✅ Búsqueda semántica
✅ Grafo semántico
✅ Chat RAG
✅ Control de acceso basado en roles
✅ Caching y optimización
✅ Manejo robusto de errores
✅ Logging estructurado
✅ Rate limiting
✅ Validación de entrada

¡A deployar! 🚀
