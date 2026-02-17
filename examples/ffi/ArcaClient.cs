/**
 * ARCA SDK - C# Wrapper
 *
 * Ejemplo de uso desde C#/.NET usando P/Invoke
 *
 * Compilar:
 *   csc ArcaClient.cs -out:ArcaExample.exe
 *
 * Copiar libarca.so (Linux) o arca.dll (Windows) al mismo directorio.
 */

using System;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace Arca
{
    /// <summary>
    /// Cliente para integración con ARCA usando la biblioteca ARCA
    /// </summary>
    public class ArcaClient : IDisposable
    {
        private IntPtr _handle;
        private bool _disposed = false;

        #region P/Invoke Declarations

        private const string DllName = "arca";

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern void arca_free_string(IntPtr s);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr arca_last_error();

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_client_new_testing(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string certPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string keyPath,
            out IntPtr handle
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_client_new_production(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string certPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string keyPath,
            out IntPtr handle
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_client_new_pfx_testing(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string pfxPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string password,
            out IntPtr handle
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_client_new_pfx_production(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string pfxPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string password,
            out IntPtr handle
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern void arca_client_free(IntPtr handle);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_login(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string service
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_set_token_cache(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string cacheDir
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_is_authenticated(IntPtr handle);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_dummy(IntPtr handle, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_comp_ultimo_autorizado(
            IntPtr handle,
            ulong cuit,
            int ptoVta,
            int cbteTipo,
            out long outNumero
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fecae_solicitar(
            IntPtr handle,
            ulong cuit,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string invoiceJson,
            out IntPtr outJson
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_comp_consultar(
            IntPtr handle,
            ulong cuit,
            int cbteTipo,
            ulong cbteNro,
            int ptoVta,
            out IntPtr outJson
        );

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_param_get_tipos_cbte(IntPtr handle, ulong cuit, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_param_get_tipos_doc(IntPtr handle, ulong cuit, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_param_get_tipos_iva(IntPtr handle, ulong cuit, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_param_get_tipos_monedas(IntPtr handle, ulong cuit, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_param_get_ptos_venta(IntPtr handle, ulong cuit, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_fe_param_get_cotizacion(IntPtr handle, ulong cuit,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string monId, out double outCotizacion);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_padron_dummy(IntPtr handle, out IntPtr outJson);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int arca_padron_get_persona(IntPtr handle, ulong cuitRepresentada, ulong cuitConsulta, out IntPtr outJson);

        // Constantes
        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int ARCA_CBTE_FACTURA_A();
        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int ARCA_CBTE_FACTURA_B();
        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int ARCA_CBTE_FACTURA_C();
        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int ARCA_DOC_CONSUMIDOR_FINAL();
        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int ARCA_IVA_CONSUMIDOR_FINAL();

        #endregion

        #region Helper Methods

        private static string GetAndFreeString(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero) return null;
            string result = Marshal.PtrToStringUTF8(ptr);
            arca_free_string(ptr);
            return result;
        }

        private static string GetLastError()
        {
            IntPtr ptr = arca_last_error();
            return GetAndFreeString(ptr);
        }

        private void CheckResult(int result, string operation)
        {
            if (result != 0)
            {
                string error = GetLastError() ?? $"Error code: {result}";
                throw new ArcaException($"{operation} failed: {error}", result);
            }
        }

        #endregion

        #region Constructors

        /// <summary>
        /// Crea un cliente AFIP para testing con certificados PEM
        /// </summary>
        public static ArcaClient Testing(string certPath, string keyPath)
        {
            var client = new ArcaClient();
            int result = arca_client_new_testing(certPath, keyPath, out client._handle);
            client.CheckResult(result, "Create testing client");
            return client;
        }

        /// <summary>
        /// Crea un cliente AFIP para producción con certificados PEM
        /// </summary>
        public static ArcaClient Production(string certPath, string keyPath)
        {
            var client = new ArcaClient();
            int result = arca_client_new_production(certPath, keyPath, out client._handle);
            client.CheckResult(result, "Create production client");
            return client;
        }

        /// <summary>
        /// Crea un cliente AFIP para testing con certificado PFX
        /// </summary>
        public static ArcaClient TestingPfx(string pfxPath, string password = "")
        {
            var client = new ArcaClient();
            int result = arca_client_new_pfx_testing(pfxPath, password, out client._handle);
            client.CheckResult(result, "Create testing PFX client");
            return client;
        }

        /// <summary>
        /// Crea un cliente AFIP para producción con certificado PFX
        /// </summary>
        public static ArcaClient ProductionPfx(string pfxPath, string password = "")
        {
            var client = new ArcaClient();
            int result = arca_client_new_pfx_production(pfxPath, password, out client._handle);
            client.CheckResult(result, "Create production PFX client");
            return client;
        }

        private ArcaClient() { }

        #endregion

        #region Authentication

        /// <summary>
        /// Autentica con WSAA para un servicio específico
        /// </summary>
        public void Login(string service = "wsfe")
        {
            CheckResult(arca_login(_handle, service), "Login");
        }

        /// <summary>
        /// Configura el directorio de cache para persistir tokens WSAA en disco.
        /// Llamar antes de Login(). Evita lockouts de 12h si el proceso se reinicia.
        /// </summary>
        public void SetTokenCache(string cacheDir)
        {
            CheckResult(arca_set_token_cache(_handle, cacheDir), "SetTokenCache");
        }

        /// <summary>
        /// Verifica si el cliente está autenticado
        /// </summary>
        public bool IsAuthenticated => arca_is_authenticated(_handle) == 1;

        #endregion

        #region WSFEv1 Methods

        /// <summary>
        /// Verifica el estado del servicio WSFEv1
        /// </summary>
        public JsonDocument FeDummy()
        {
            IntPtr json;
            CheckResult(arca_fe_dummy(_handle, out json), "FEDummy");
            return JsonDocument.Parse(GetAndFreeString(json));
        }

        /// <summary>
        /// Obtiene el último comprobante autorizado
        /// </summary>
        public long FeCompUltimoAutorizado(ulong cuit, int ptoVta, int cbteTipo)
        {
            long numero;
            CheckResult(arca_fe_comp_ultimo_autorizado(_handle, cuit, ptoVta, cbteTipo, out numero), "FECompUltimoAutorizado");
            return numero;
        }

        /// <summary>
        /// Solicita CAE para una factura
        /// </summary>
        public JsonDocument FeCaeSolicitar(ulong cuit, object invoice)
        {
            string invoiceJson = JsonSerializer.Serialize(invoice);
            IntPtr json;
            CheckResult(arca_fecae_solicitar(_handle, cuit, invoiceJson, out json), "FECAESolicitar");
            return JsonDocument.Parse(GetAndFreeString(json));
        }

        /// <summary>
        /// Consulta un comprobante emitido
        /// </summary>
        public JsonDocument FeCompConsultar(ulong cuit, int cbteTipo, ulong cbteNro, int ptoVta)
        {
            IntPtr json;
            CheckResult(arca_fe_comp_consultar(_handle, cuit, cbteTipo, cbteNro, ptoVta, out json), "FECompConsultar");
            return JsonDocument.Parse(GetAndFreeString(json));
        }

        /// <summary>
        /// Obtiene los tipos de comprobante disponibles
        /// </summary>
        public JsonDocument FeParamGetTiposCbte(ulong cuit)
        {
            IntPtr json;
            CheckResult(arca_fe_param_get_tipos_cbte(_handle, cuit, out json), "FEParamGetTiposCbte");
            return JsonDocument.Parse(GetAndFreeString(json));
        }

        /// <summary>
        /// Obtiene la cotización de una moneda
        /// </summary>
        public double FeParamGetCotizacion(ulong cuit, string monId)
        {
            double cotizacion;
            CheckResult(arca_fe_param_get_cotizacion(_handle, cuit, monId, out cotizacion), "FEParamGetCotizacion");
            return cotizacion;
        }

        #endregion

        #region Constants

        public static class CbteTipos
        {
            public static int FacturaA => ARCA_CBTE_FACTURA_A();
            public static int FacturaB => ARCA_CBTE_FACTURA_B();
            public static int FacturaC => ARCA_CBTE_FACTURA_C();
        }

        public static class DocTipos
        {
            public static int ConsumidorFinal => ARCA_DOC_CONSUMIDOR_FINAL();
        }

        public static class CondicionIva
        {
            public static int ConsumidorFinal => ARCA_IVA_CONSUMIDOR_FINAL();
        }

        #endregion

        #region IDisposable

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                if (_handle != IntPtr.Zero)
                {
                    arca_client_free(_handle);
                    _handle = IntPtr.Zero;
                }
                _disposed = true;
            }
        }

        ~ArcaClient()
        {
            Dispose(false);
        }

        #endregion
    }

    /// <summary>
    /// Excepción específica para errores ARCA
    /// </summary>
    public class ArcaException : Exception
    {
        public int ErrorCode { get; }

        public ArcaException(string message, int errorCode) : base(message)
        {
            ErrorCode = errorCode;
        }
    }

    /// <summary>
    /// Ejemplo de uso
    /// </summary>
    class Program
    {
        static void Main(string[] args)
        {
            const ulong CUIT = 20299879624;
            const int PTO_VTA = 2;

            try
            {
                // Crear cliente con PFX
                using var client = ArcaClient.ProductionPfx("certificado.pfx", "");

                // Habilitar cache de tokens (evita lockouts de 12h)
                client.SetTokenCache("./cache");

                // Autenticar
                client.Login("wsfe");
                Console.WriteLine("Login exitoso!");

                // Verificar servicio
                var dummy = client.FeDummy();
                Console.WriteLine($"AppServer: {dummy.RootElement.GetProperty("AppServer")}");

                // Obtener último comprobante
                long ultimo = client.FeCompUltimoAutorizado(CUIT, PTO_VTA, ArcaClient.CbteTipos.FacturaC);
                Console.WriteLine($"Último comprobante: {ultimo}");

                // Emitir factura
                var invoice = new
                {
                    pto_vta = PTO_VTA,
                    cbte_tipo = ArcaClient.CbteTipos.FacturaC,
                    concepto = 1,
                    doc_tipo = ArcaClient.DocTipos.ConsumidorFinal,
                    doc_nro = 0,
                    cbte_desde = ultimo + 1,
                    cbte_hasta = ultimo + 1,
                    cbte_fch = DateTime.Now.ToString("yyyyMMdd"),
                    imp_total = 1.0,
                    imp_tot_conc = 0.0,
                    imp_neto = 1.0,
                    imp_op_ex = 0.0,
                    imp_trib = 0.0,
                    imp_iva = 0.0,
                    mon_id = "PES",
                    mon_cotiz = 1.0,
                    condicion_iva_receptor = ArcaClient.CondicionIva.ConsumidorFinal,
                    iva = new object[] { }
                };

                var result = client.FeCaeSolicitar(CUIT, invoice);
                Console.WriteLine($"Resultado: {result.RootElement.GetProperty("resultado")}");
                Console.WriteLine($"CAE: {result.RootElement.GetProperty("cae")}");
            }
            catch (ArcaException ex)
            {
                Console.WriteLine($"Error AFIP [{ex.ErrorCode}]: {ex.Message}");
            }
        }
    }
}
