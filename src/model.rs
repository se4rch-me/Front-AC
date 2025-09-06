use serde::Serialize;

// Usamos Default para poder crear una instancia vacía fácilmente.
// Usamos Clone para poder duplicar el estado.
#[derive(Clone, Default, Serialize, Debug)]
pub struct Conexion {
    pub cota_razante: String,
    pub cota_clave: String,
    pub diametro_pulgadas: String,
    pub material: String,
    pub conecta_a: String,
}

#[derive(Clone, Default, Serialize, Debug)]
pub struct Encuesta {
    // --- Datos Generales ---
    pub pozo_numero: String, // Campo que añadimos nosotros
    pub tipo_sistema: String,
    pub tipo_pozo: String,

    // --- Tapa ---
    pub tapa_existe: String,
    pub tapa_tipo: String,
    pub tapa_estado: String,
    pub tapa_diagnostico: String,

    // --- Cargue ---
    pub cargue_existe: String,
    pub cargue_estado: String,
    pub cargue_diagnostico: String,

    // --- Cono ---
    pub cono_existe: String,
    pub cono_estado: String,
    pub cono_diagnostico: String,

    // --- Cilindro ---
    pub cilindro_material: String,
    pub cilindro_estado: String,
    pub cilindro_diagnostico: String,

    // --- Cañuela ---
    pub canuela_estado: String,
    pub canuela_diagnostico: String,

    // --- Escalones ---
    pub escalones_existe: String,
    pub escalones_tipo: String,
    pub escalones_estado: String,
    pub escalones_diagnostico: String,

    // --- Datos Finales ---
    pub estado_general_pozo: String,
    pub observaciones: String, // Campo lógico para añadir notas

    // --- Conexiones (lista de tamaño variable) ---
    #[serde(rename = "conexiones")] // Asegura que en JSON el campo se llame "conexiones"
    pub lista_conexiones: Vec<Conexion>,
}