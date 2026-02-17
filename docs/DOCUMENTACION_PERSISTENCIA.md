# Documentación Técnica: Sistema de Persistencia y Soporte Nativo PFX

Esta documentación detalla la implementación del sistema de caché persistente y el manejo automatizado de certificados PFX en el SDK **arca**.

## 1. El Problema: Ciclo de Vida de los Tokens de ARCA

El Servicio de Autenticación y Autorización (WSAA) de ARCA otorga **Tickets de Acceso (TA)** con una validez de **12 horas**. 

### Limitaciones de ARCA:
- **Cuotas de Solicitud:** ARCA limita la cantidad de TAs que se pueden solicitar en un período corto.
- **Error de "TA Válido":** Si solicitas un nuevo token mientras aún tienes uno vigente, ARCA puede retornar el error: `"El CEE ya posee un TA valido para el acceso al WSN solicitado"`.
- **Bloqueos:** El abuso de peticiones de login puede resultar en bloqueos temporales de la IP o del certificado.

---

## 2. Solución: Caché Híbrida de Dos Niveles

Para maximizar la eficiencia y cumplir con las políticas de ARCA, se implementó una estrategia de persistencia en dos capas:

### Nivel 1: Memoria RAM (Rápido)
- Se utiliza un `HashMap` protegido por un `RwLock` asíncrono.
- Ideal para aplicaciones de alto rendimiento o servicios que permanecen activos (ej. APIs).
- Evita el acceso a disco en peticiones concurrentes.

### Nivel 2: Disco / Sistema de Archivos (Persistente)
- Implementado a través del trait `TokenStore`.
- Los tokens se guardan en formato JSON.
- Permite que la aplicación se reinicie sin perder la sesión de ARCA.
- **Importante:** La librería nunca borra estos archivos; simplemente los sobrescribe cuando el token contenido ha expirado.

---

## 3. Soporte Nativo para PFX (PKCS#12)

Anteriormente, el desarrollador debía convertir manualmente sus archivos `.pfx` a `.pem` usando herramientas externas. Ahora, la librería lo maneja internamente:

- **Conversión Transparente:** El método `.credentials_pfx(data, password)` extrae automáticamente el certificado y la clave privada.
- **Soporte Legacy:** Utiliza los proveedores `legacy` y `default` de OpenSSL para ser compatible con los algoritmos de cifrado antiguos (como RC2-40-CBC) que ARCA sigue utilizando en muchos de sus certificados de homologación y producción.
- **Seguridad:** El proceso de extracción utiliza `stdin` para pasar contraseñas, evitando que queden expuestas en la lista de procesos del sistema.

---

## 4. Implementación Técnica Detallada

### A. Requisitos en `Cargo.toml`
La persistencia requiere serialización y capacidades de entrada/salida asíncronas:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync", "fs", "io-util"] }
async-trait = "0.1"
```

### B. El Trait `TokenStore`
Ubicado en `src/client/token_store.rs`, permite abstraer dónde se guardan los tokens:

```rust
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Guarda un token asociado a un servicio (ej: "wsfe")
    async fn save_token(&self, service: &str, token: &TicketAcceso) -> Result<()>;
    
    /// Recupera un token si existe
    async fn load_token(&self, service: &str) -> Result<Option<TicketAcceso>>;
}
```

### C. Lógica de Login (Algoritmo de Decisión)
Cuando se llama a `client.login("servicio")`, el SDK sigue este orden:

1. **Consulta RAM:** ¿Tengo el token en el `HashMap` interno? ¿Es válido (no ha expirado)?
   - *Si es sí:* Retorna éxito inmediatamente.
2. **Consulta Disco:** Si no está en RAM, ¿Existe el archivo `./tokens/arca_token_servicio.json`?
   - *Si existe y es válido:* Lo carga a RAM y retorna éxito.
3. **Petición a ARCA (WSAA):** Si nada de lo anterior funciona:
   - Firma un nuevo CMS usando el certificado (PEM o PFX).
   - Solicita el TA a ARCA.
   - Si tiene éxito, guarda el nuevo TA en **RAM** y en **Disco**.

---

## 5. Ejemplo de Implementación en Producción

Este es el patrón recomendado para evitar errores de conexión y maximizar la velocidad:

```rust
use arca::{ArcaClient, ArcaEnvironment};
use arca::client::token_store::FileSystemTokenStore;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Definir una ruta para la caché (asegurar permisos de escritura)
    // El SDK creará la carpeta si no existe.
    let cache_path = "./tokens_cache";
    let store = FileSystemTokenStore::new(cache_path);

    // 2. Leer el certificado PFX
    let pfx_data = fs::read("certificado.pfx")?;
    let password = "mi_password_pfx"; // O "" si no tiene

    // 3. Configurar el cliente
    let client = ArcaClient::builder()
        .environment(ArcaEnvironment::Production)
        .credentials_pfx(pfx_data, password)
        .token_store(store) // <--- Activa la persistencia
        .build()?;

    // 4. Ejecutar operaciones
    // La primera ejecución del día irá a ARCA. 
    // Las ejecuciones siguientes durante las próximas 12 horas leerán del disco.
    client.login("wsfe").await?;
    
    let ultimo = client.get_last_voucher(20123456781, 1, 6).await?;
    println!("Último comprobante: {}", ultimo);

    Ok(())
}
```

---

## 6. Troubleshooting y Consideraciones

### Error: "Failed to extract certificate"
Si recibes este error al usar PFX, asegúrate de que:
1. El binario `openssl` esté disponible en el `PATH` del sistema.
2. Si usas Linux moderno (ej. Ubuntu 22.04+), el archivo `/etc/ssl/openssl.cnf` o tu configuración local de OpenSSL debe permitir el provider `legacy`. El SDK intenta forzarlo con `--provider legacy`, pero el soporte debe estar instalado en el sistema.

### ¿Por qué sigo viendo el error de "TA Válido"?
Esto ocurre si borras manualmente la carpeta de caché antes de que el token de ARCA haya expirado. Al borrar la caché local, el SDK intenta pedir uno nuevo y ARCA (que recuerda el anterior) lo rechaza. 
**Regla de oro:** No borres la carpeta de tokens a menos que quieras forzar una re-autenticación por un cambio de certificado.

### Seguridad
- Los archivos JSON contienen la `sign` (firma) del token. Aunque son temporales, se recomienda que el directorio de caché tenga permisos de lectura/escritura limitados al usuario que ejecuta el proceso.
- El SDK utiliza la crate `zeroize` para limpiar de la memoria RAM los datos sensibles (claves privadas y contraseñas) inmediatamente después de su uso.

---

## 7. Persistencia desde FFI/DLL

La DLL (`libarca.so` / `arca.dll`) también soporta persistencia de tokens mediante la función `arca_set_token_cache`. Esto es **crítico** para aplicaciones que usan la biblioteca desde C#, Python, Delphi u otros lenguajes, ya que sin persistencia un reinicio del proceso causa un lockout de ~12 horas.

### Función C

```c
// Llamar ANTES de arca_login()
int arca_set_token_cache(ArcaHandle* handle, const char* cache_dir);
```

### Flujo con Cache (3 niveles)

```
arca_login("wsfe")
  ├─ RAM: token válido en memoria? → OK (instantáneo)
  ├─ Disco: arca_token_wsfe.json existe y no expiró? → cargar a RAM → OK
  └─ WSAA: login completo → guardar en RAM + disco → OK
```

### Ejemplo C#

```csharp
using var client = ArcaClient.ProductionPfx("certificado.pfx", "");

// Habilitar cache de tokens (evita lockouts de 12h)
client.SetTokenCache("./cache");

// Login: usa cache automáticamente
client.Login("wsfe");
```

### Ejemplo Python

```python
with ArcaClient.production_pfx("certificado.pfx", "") as client:
    # Habilitar cache de tokens (evita lockouts de 12h)
    client.set_token_cache("./cache")

    # Login: usa cache automáticamente
    client.login("wsfe")
```

### Ejemplo Delphi

```pascal
Client := TArcaClient.CreateProductionPfx('certificado.pfx', '');
try
  // Habilitar cache de tokens (evita lockouts de 12h)
  Client.SetTokenCache('./cache');

  // Login: usa cache automáticamente
  Client.Login('wsfe');
finally
  Client.Free;
end;
```

### Notas Importantes

- Si no se llama a `arca_set_token_cache`, el comportamiento es idéntico a la versión anterior (tokens solo en RAM).
- Los errores de lectura/escritura en disco son silenciosos — el login siempre puede completarse vía WSAA como fallback.
- El directorio de cache se crea automáticamente si no existe.
- Los archivos de token se nombran `arca_token_{servicio}.json` (ej: `arca_token_wsfe.json`).