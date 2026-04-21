# Plan de Implementación: Mejoras al Grafo Semántico y Chat RAG

## Problema Actual

1. **Grafo basado en títulos**: El método `rebuild_graph()` usa TF-IDF local sobre títulos, no embeddings de contenido completo
2. **Sin integración chat-grafo**: El chat RAG existe en el backend pero no hay UI para usarlo
3. **UI básica**: Colores, tipografía y espaciado necesitan mejoras

## Solución

### 1. Usar Embeddings Reales para el Grafo

**Archivo**: `src/bin/app.rs`

**Cambios en `rebuild_graph()`**:
- Eliminar el cálculo TF-IDF local
- Cargar embeddings reales desde la base de datos (columna `content_embedding`)
- Calcular similitud coseno usando los vectores de 768 dimensiones
- Crear edges solo para similitudes > threshold configurable

**Nuevo código**:
```rust
fn rebuild_graph(&mut self) {
    // Cargar embeddings reales de la BD
    // Calcular similitud coseno entre vectores de 768-dim
    // Crear edges basados en similitud de contenido completo
}
```

### 2. Añadir Panel de Chat RAG

**Archivo**: `src/bin/app.rs`

**Nueva vista**: `View::Chat`

**Componentes**:
- Input de texto para mensajes
- Historial de conversación (user/assistant)
- Botón "Send" con estilo moderno
- Indicador de "typing..." mientras espera respuesta
- Highlight de nodos referenciados en el grafo

**Flujo**:
1. Usuario escribe mensaje
2. POST a `/api/vaults/{vault_id}/chat`
3. Mostrar respuesta del asistente
4. Highlight nodos en `referenced_node_ids` con efecto glow

### 3. Mejoras de UI/UX

**Colores modernos**:
```rust
// Paleta actualizada
const BG_DARK: Color32 = Color32::from_rgb(15, 15, 20);
const BG_CARD: Color32 = Color32::from_rgb(25, 25, 35);
const ACCENT: Color32 = Color32::from_rgb(139, 92, 246); // violet-500
const ACCENT_HOVER: Color32 = Color32::from_rgb(167, 139, 250); // violet-400
const TEXT: Color32 = Color32::from_rgb(240, 240, 245);
const TEXT_SUB: Color32 = Color32::from_rgb(160, 160, 180);
const TEXT_DIM: Color32 = Color32::from_rgb(100, 100, 120);
```

**Tipografía**:
- Títulos: 18-24px, bold
- Cuerpo: 14px, regular
- Subtítulos: 12px, medium
- Monospace para IDs/códigos

**Espaciado**:
- Padding cards: 16-20px
- Spacing entre elementos: 8-12px
- Margins entre secciones: 20-30px

**Efectos**:
- Hover states en botones
- Smooth transitions (0.2s)
- Glow effect para nodos destacados
- Rounded corners (8-12px)

## Tareas

### Fase 1: Backend (si es necesario)
- [ ] Verificar que el endpoint `/api/vaults/{vault_id}/chat` funciona correctamente
- [ ] Verificar que los embeddings se generan del contenido completo (no títulos)

### Fase 2: Grafo con Embeddings Reales
- [ ] Modificar `rebuild_graph()` para cargar embeddings de la BD
- [ ] Implementar cálculo de similitud coseno con vectores de 768-dim
- [ ] Añadir threshold configurable en UI (slider)
- [ ] Actualizar visualización de edges con pesos correctos

### Fase 3: Chat RAG UI
- [ ] Añadir `View::Chat` al enum
- [ ] Crear `show_chat()` method
- [ ] Implementar input de mensajes
- [ ] Implementar historial de conversación
- [ ] Conectar con endpoint `/api/vaults/{vault_id}/chat`
- [ ] Implementar highlight de nodos referenciados

### Fase 4: Mejoras Visuales
- [ ] Actualizar paleta de colores
- [ ] Mejorar tipografía (tamaños, pesos)
- [ ] Añadir efectos hover/active
- [ ] Mejorar spacing y padding
- [ ] Añadir animaciones suaves
- [ ] Mejorar diseño de cards
- [ ] Mejorar diseño de botones

### Fase 5: Integración Chat-Grafo
- [ ] Cuando se recibe respuesta del chat, highlight nodos en el grafo
- [ ] Añadir botón para "ver en grafo" desde el chat
- [ ] Sincronizar selección entre chat y grafo
- [ ] Añadir efecto glow a nodos referenciados

## Notas Técnicas

### Cálculo de Similitud Coseno

```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}
```

### Estructura de Mensaje de Chat

```rust
struct ChatMessage {
    role: String, // "user" | "assistant"
    content: String,
    referenced_nodes: Vec<Uuid>, // solo para assistant
    timestamp: DateTime<Utc>,
}
```

### Highlight de Nodos

```rust
// En show_graph(), al dibujar nodos:
let is_referenced = self.chat_referenced_nodes.contains(&node.item_id);
if is_referenced {
    // Dibujar glow effect
    painter.circle_filled(node.pos, radius + 4.0, Color32::from_rgba_unmultiplied(139, 92, 246, 60));
}
```
