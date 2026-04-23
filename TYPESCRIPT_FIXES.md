# 🔧 TypeScript Fixes Applied

## Errores Arreglados

### 1. AuthService.ts - signUpWithPassword
**Error**: `Property 'signUpWithPassword' does not exist on type 'SupabaseAuthClient'`

**Fix**: Cambié `signUpWithPassword` a `signUp`
```typescript
// Antes
const { data, error } = await supabase.auth.signUpWithPassword({...})

// Después
const { data, error } = await supabase.auth.signUp({...})
```

### 2. VaultService.ts - Parameter 'row' implicitly has an 'any' type
**Error**: `Parameter 'row' implicitly has an 'any' type` (línea 88)

**Fix**: Agregué tipo `any` al parámetro
```typescript
// Antes
return result.rows.map((row) => ({...}))

// Después
return result.rows.map((row: any) => ({...}))
```

### 3. ContentService.ts - Parameter 'row' implicitly has an 'any' type
**Error**: `Parameter 'row' implicitly has an 'any' type` (línea 148)

**Fix**: Agregué tipo `any` al parámetro
```typescript
// Antes
const items = result.rows.map((row) => ({...}))

// Después
const items = result.rows.map((row: any) => ({...}))
```

### 4. GraphService.ts - Parameter 'row' implicitly has an 'any' type (2 lugares)
**Error**: `Parameter 'row' implicitly has an 'any' type` (líneas 37 y 130)

**Fix**: Agregué tipo `any` a ambos parámetros
```typescript
// Antes
const contentItems: ContentItem[] = contentResult.rows.map((row) => ({...}))
return result.rows.map((row) => row.content_item_id)

// Después
const contentItems: ContentItem[] = contentResult.rows.map((row: any) => ({...}))
return result.rows.map((row: any) => row.content_item_id)
```

### 5. EmbeddingService.ts - Parameter 'row' implicitly has an 'any' type
**Error**: `Parameter 'row' implicitly has an 'any' type` (línea 118)

**Fix**: Agregué tipo `any` al parámetro
```typescript
// Antes
return result.rows.map((row) => row.content_item_id)

// Después
return result.rows.map((row: any) => row.content_item_id)
```

## ✅ Todos los Errores Arreglados

Los errores de TypeScript han sido corregidos. Ahora puedes hacer push y Vercel debería compilar correctamente.

## 🚀 Próximos Pasos

```bash
git add api/src/services/
git commit -m "Fix: TypeScript type errors in services"
git push origin main
```

Vercel se re-deployará automáticamente. 🎉
