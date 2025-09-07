use dioxus::prelude::*;
use futures_util::StreamExt;
use reqwest::multipart::Part;

mod model;
use model::{Conexion, Encuesta};

// --- Opciones para los menús de selección (sin cambios) ---
const TIPO_SISTEMA: &[&str] = &["Aguas Lluvia", "Aguas Residuales", "Combinado"];
const TIPO_POZO: &[&str] = &["Pozo", "Camara", "Alivio"];
const SI_NO: &[&str] = &["Si", "No"];
const TAPA_TIPO: &[&str] = &["Ferroconcreto", "Concreto", "Hierro sin Bisagra", "Hierro con bisagra", "Tapa Seguridad", "Tapa en fibra"];
const ESTADO_BUENO_REGULAR_MALO: &[&str] = &["Bueno", "Regular", "Malo"];
const DIAGNOSTICO_CAMBIAR_REPARAR: &[&str] = &["Cambiar", "Reparar", "No Requiere"];
const CARGUE_ESTADO: &[&str] = &["Bueno", "Regular", "Malo", "Grietas", "Partido", "Hundido"];
const CILINDRO_MATERIAL: &[&str] = &["Mamposteria", "Concreto", "GRP"];
const CILINDRO_ESTADO: &[&str] = &["Bueno", "Regular", "Malo", "Grietas", "Partido", "Huecos", "Sin Pañete", "Otro"];
const CANUELA_ESTADO: &[&str] = &["Bueno", "Regular", "Malo", "Sedimentada", "Desgastada", "Socavacion"];
const ESCALONES_TIPO: &[&str] = &["Escalones", "Ladrillos"];
const ESCALONES_ESTADO: &[&str] = &["Bueno", "Regular", "Malo", "Doblados", "Faltan", "Corroidos"];
const ESTADO_GENERAL_POZO: &[&str] = &["Infiltracion", "Represado", "Con basura", "Raices", "Fuera de Servicio", "Lleno de tierra"];

const TAILWIND_CSS: &str = include_str!("../public/css/tailwind.css");

// --- Estado de Autenticación (simplificado) ---
#[derive(Clone, Debug, PartialEq)]
enum AuthStatus {
    Authenticated,
    Unauthenticated,
}

const BACKEND_URL: &str = "http://192.168.128.15:5000";

// --- Función para verificar el estado de autenticación (con cache buster) ---
async fn check_auth_status() -> AuthStatus {
    // Usamos la hora actual para evitar la caché del navegador
    let now = web_sys::js_sys::Date::new_0().get_time();
    let url = format!("{}/auth/status?_={}", BACKEND_URL, now);
    
    log::info!("Verificando estado de autenticación en: {}", url);
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                log::info!("Estado: Autenticado");
                AuthStatus::Authenticated
            } else {
                log::info!("Estado: No Autenticado");
                AuthStatus::Unauthenticated
            }
        }
        Err(e) => {
            log::error!("No se pudo conectar al backend para verificar la autenticación: {}. Asumiendo no autenticado.", e);
            AuthStatus::Unauthenticated
        }
    }
}

async fn send_survey_request(encuesta: Encuesta, files_content: Vec<(String, Vec<u8>)>) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/ingestar-encuesta", BACKEND_URL);
    let json_data = serde_json::to_string(&encuesta).expect("No se pudo serializar la encuesta a JSON");

    log::info!("Conexión: Intentando enviar a URL: {}", url);
    log::info!("Conexión: JSON de la encuesta: {}", json_data);

    let mut form = reqwest::multipart::Form::new().text("data", json_data);

    for (file_name, file_bytes) in files_content {
        log::info!("Adjuntando archivo: {}", file_name);
        let part = Part::bytes(file_bytes).file_name(file_name);
        form = form.part("fotos", part);
    }

    let response = client.post(url).multipart(form).send().await?;
    
    if response.status() == 401 {
        log::error!("Error 401: No autorizado. El token puede haber expirado o sido revocado. Se necesita re-autenticación.");
    }
    
    Ok(())
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    launch(App);
}

#[allow(non_snake_case)]
fn App() -> Element {
    let auth_status = use_resource(check_auth_status);
    let encuesta = use_signal(Encuesta::default);
    let uploaded_files_content = use_signal(Vec::<(String, Vec<u8>)>::new);

    let send_survey = use_coroutine(|mut rx: UnboundedReceiver<(Encuesta, Vec<(String, Vec<u8>)>)>| async move {
        while let Some((encuesta_data, files)) = rx.next().await {
            log::info!("Enviando encuesta...");
            match send_survey_request(encuesta_data, files).await {
                Ok(_) => log::info!("¡Encuesta enviada con éxito!"),
                Err(e) => log::error!("Error al enviar la encuesta: {:?}", e),
            }
        }
    });

    rsx! {
        style { "{TAILWIND_CSS}" }

        main {
            class: "bg-gradient-to-br from-blue-50 to-indigo-100 min-h-screen font-sans flex items-center justify-center py-8",
            
            match auth_status.read().as_ref() {
                Some(AuthStatus::Authenticated) => rsx! {
                    SurveyForm {
                        encuesta: encuesta,
                        uploaded_files_content: uploaded_files_content,
                        on_submit: move |_| {
                            let current_survey = encuesta.read().clone();
                            let files_to_send = uploaded_files_content.read().clone();
                            send_survey.send((current_survey, files_to_send));
                        }
                    }
                },
                Some(AuthStatus::Unauthenticated) => rsx! {
                    LoginPage {}
                },
                None => rsx! {
                     LoadingScreen {}
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn LoadingScreen() -> Element {
    rsx! {
        div {
            class: "text-center",
            h1 { class: "text-3xl font-bold text-gray-700", "Verificando autenticación..." },
            p { class: "text-gray-500 mt-2", "Por favor, espere un momento." }
        }
    }
}

#[allow(non_snake_case)]
fn LoginPage() -> Element {
    let login_url = format!("{}/login", BACKEND_URL);
    rsx! {
        div {
            class: "container mx-auto p-4 sm:p-8",
            div {
                class: "bg-white p-8 rounded-xl shadow-2xl w-full max-w-md mx-auto border border-gray-200 text-center",
                header {
                    class: "mb-8",
                    h1 {
                        class: "text-4xl font-extrabold text-gray-800 mb-2",
                        "Acceso Requerido"
                    }
                    p {
                        class: "text-md text-gray-600",
                        "Se necesita autorización para subir archivos a Google Drive."
                    }
                }
                div {
                    a {
                        href: "{login_url}",
                        class: "w-full inline-block bg-blue-600 text-white font-bold py-3 px-4 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-transform transform hover:scale-105",
                        "Iniciar Sesión con Google"
                    }
                }
                 p {
                    class: "text-xs text-gray-500 mt-6",
                    "Serás redirigido a la página de inicio de sesión de Google para autorizar la aplicación."
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SurveyFormProps {
    encuesta: Signal<Encuesta>,
    uploaded_files_content: Signal<Vec<(String, Vec<u8>)>>,
    on_submit: EventHandler<()>,
}

#[allow(non_snake_case)]
fn SurveyForm(props: SurveyFormProps) -> Element {
    let mut encuesta = props.encuesta;
    let uploaded_files_content = props.uploaded_files_content;

    rsx! {
        div {
            class: "container mx-auto p-4 sm:p-8",
            div {
                class: "bg-white p-8 rounded-xl shadow-2xl w-full max-w-4xl mx-auto border border-gray-200",
                header {
                    class: "mb-8 border-b pb-4 border-blue-200",
                    h1 {
                        class: "text-4xl font-extrabold text-gray-800 text-center mb-2",
                        "Reporte de Inspección de Pozo"
                    }
                    p {
                        class: "text-md text-gray-600 text-center",
                        "Complete todos los campos para generar el reporte técnico."
                    }
                }
                form {
                    prevent_default: "onsubmit",
                    onsubmit: move |_| props.on_submit.call(()),
                    div {
                        class: "space-y-8",
                        FormFieldSection {
                            title: "Datos Generales".to_string(),
                            grid_cols: Some(3),
                            FormSelect { label: "Tipo de Sistema".to_string(), value: encuesta.read().tipo_sistema.clone(), options: TIPO_SISTEMA.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().tipo_sistema = v },
                            FormSelect { label: "Tipo de Pozo".to_string(), value: encuesta.read().tipo_pozo.clone(), options: TIPO_POZO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().tipo_pozo = v },
                            FormInput { label: "Número de Pozo".to_string(), value: encuesta.read().pozo_numero.clone(), on_input: move |v| encuesta.write().pozo_numero = v },
                        },
                        FormFieldSection {
                            title: "Tapa".to_string(),
                            FormSelect { label: "Existe".to_string(), value: encuesta.read().tapa_existe.clone(), options: SI_NO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().tapa_existe = v },
                            FormSelect { label: "Tipo".to_string(), value: encuesta.read().tapa_tipo.clone(), options: TAPA_TIPO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().tapa_tipo = v },
                            FormSelect { label: "Estado".to_string(), value: encuesta.read().tapa_estado.clone(), options: ESTADO_BUENO_REGULAR_MALO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().tapa_estado = v },
                            FormSelect { label: "Diagnóstico".to_string(), value: encuesta.read().tapa_diagnostico.clone(), options: DIAGNOSTICO_CAMBIAR_REPARAR.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().tapa_diagnostico = v },
                        },
                        FormFieldSection {
                            title: "Cargue".to_string(),
                            grid_cols: Some(3),
                            FormSelect { label: "Existe".to_string(), value: encuesta.read().cargue_existe.clone(), options: SI_NO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cargue_existe = v },
                            FormSelect { label: "Estado".to_string(), value: encuesta.read().cargue_estado.clone(), options: CARGUE_ESTADO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cargue_estado = v },
                            FormSelect { label: "Diagnóstico".to_string(), value: encuesta.read().cargue_diagnostico.clone(), options: DIAGNOSTICO_CAMBIAR_REPARAR.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cargue_diagnostico = v },
                        },
                        FormFieldSection {
                            title: "Cono".to_string(),
                            grid_cols: Some(3),
                            FormSelect { label: "Existe".to_string(), value: encuesta.read().cono_existe.clone(), options: SI_NO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cono_existe = v },
                            FormSelect { label: "Estado".to_string(), value: encuesta.read().cono_estado.clone(), options: CARGUE_ESTADO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cono_estado = v },
                            FormSelect { label: "Diagnóstico".to_string(), value: encuesta.read().cono_diagnostico.clone(), options: DIAGNOSTICO_CAMBIAR_REPARAR.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cono_diagnostico = v },
                        },
                        FormFieldSection {
                            title: "Cilindro".to_string(),
                            grid_cols: Some(3),
                            FormSelect { label: "Material".to_string(), value: encuesta.read().cilindro_material.clone(), options: CILINDRO_MATERIAL.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cilindro_material = v },
                            FormSelect { label: "Estado".to_string(), value: encuesta.read().cilindro_estado.clone(), options: CILINDRO_ESTADO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cilindro_estado = v },
                            FormSelect { label: "Diagnóstico".to_string(), value: encuesta.read().cilindro_diagnostico.clone(), options: DIAGNOSTICO_CAMBIAR_REPARAR.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().cilindro_diagnostico = v },
                        },
                        FormFieldSection {
                            title: "Cañuela".to_string(),
                            grid_cols: Some(2),
                            FormSelect { label: "Estado".to_string(), value: encuesta.read().canuela_estado.clone(), options: CANUELA_ESTADO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().canuela_estado = v },
                            FormSelect { label: "Diagnostico".to_string(), value: encuesta.read().canuela_diagnostico.clone(), options: DIAGNOSTICO_CAMBIAR_REPARAR.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().canuela_diagnostico = v },
                        },
                        FormFieldSection {
                            title: "Escalones".to_string(),
                            FormSelect { label: "Existen".to_string(), value: encuesta.read().escalones_existe.clone(), options: SI_NO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().escalones_existe = v },
                            FormSelect { label: "Tipo".to_string(), value: encuesta.read().escalones_tipo.clone(), options: ESCALONES_TIPO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().escalones_tipo = v },
                            FormSelect { label: "Estado".to_string(), value: encuesta.read().escalones_estado.clone(), options: ESCALONES_ESTADO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().escalones_estado = v },
                            FormSelect { label: "Diagnóstico".to_string(), value: encuesta.read().escalones_diagnostico.clone(), options: DIAGNOSTICO_CAMBIAR_REPARAR.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().escalones_diagnostico = v },
                        },
                        FormFieldSection {
                            title: "Evaluación Final".to_string(),
                            grid_cols: Some(1),
                            FormSelect { label: "Estado General del Pozo".to_string(), value: encuesta.read().estado_general_pozo.clone(), options: ESTADO_GENERAL_POZO.iter().map(|s| s.to_string()).collect(), on_change: move |v| encuesta.write().estado_general_pozo = v },
                            FormTextArea { label: "Observaciones".to_string(), value: encuesta.read().observaciones.clone(), on_input: move |v| encuesta.write().observaciones = v },
                        },
                        div {
                            class: "p-4 border rounded-lg",
                            h3 { class: "text-lg font-semibold text-gray-700 mb-4", "Conexiones" },
                            div { class: "space-y-4",
                                for (i, _) in encuesta.read().lista_conexiones.iter().enumerate() {
                                    div { 
                                        key: "conexion-{i}",
                                        class: "grid grid-cols-1 md:grid-cols-6 gap-4 p-4 border rounded bg-gray-50",
                                        div { class: "md:col-span-5 grid grid-cols-1 md:grid-cols-5 gap-4",
                                            FormInput { label: "Cota Razante".to_string(), value: encuesta.read().lista_conexiones[i].cota_razante.clone(), on_input: move |v| encuesta.write().lista_conexiones[i].cota_razante = v },
                                            FormInput { label: "Cota Clave".to_string(), value: encuesta.read().lista_conexiones[i].cota_clave.clone(), on_input: move |v| encuesta.write().lista_conexiones[i].cota_clave = v },
                                            FormInput { label: "Diámetro (pulg)".to_string(), value: encuesta.read().lista_conexiones[i].diametro_pulgadas.clone(), on_input: move |v| encuesta.write().lista_conexiones[i].diametro_pulgadas = v },
                                            FormInput { label: "Material".to_string(), value: encuesta.read().lista_conexiones[i].material.clone(), on_input: move |v| encuesta.write().lista_conexiones[i].material = v },
                                            FormInput { label: "Conecta A".to_string(), value: encuesta.read().lista_conexiones[i].conecta_a.clone(), on_input: move |v| encuesta.write().lista_conexiones[i].conecta_a = v },
                                        },
                                        div { class: "flex items-end justify-center",
                                            button {
                                                r#type: "button",
                                                class: "bg-red-500 text-white p-2 rounded-full hover:bg-red-600 transition h-10 w-10 flex items-center justify-center",
                                                onclick: move |_| { encuesta.write().lista_conexiones.remove(i); },
                                                "X"
                                            }
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "mt-4 bg-blue-500 text-white font-semibold py-2 px-4 rounded-lg hover:bg-blue-600 transition",
                                onclick: move |_| encuesta.write().lista_conexiones.push(Conexion::default()),
                                "+ Añadir Conexión"
                            }
                        }
                        FormFieldSection {
                            title: "Fotografías".to_string(),
                            grid_cols: Some(1),
                            div {
                                class: "flex flex-col items-center",
                                label {
                                    class: "w-full max-w-xs text-center bg-gray-600 text-white font-bold py-3 px-4 rounded-lg hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-transform transform hover:scale-105 cursor-pointer",
                                    r#for: "file-upload",
                                    "Seleccionar Imágenes"
                                }
                                input {
                                    id: "file-upload",
                                    r#type: "file",
                                    multiple: true,
                                    accept: "image/*",
                                    class: "hidden",
                                    onchange: move |evt| {
                                        let mut uploaded_files_content = uploaded_files_content.to_owned();
                                        async move {
                                            if let Some(file_engine) = evt.files() {
                                                let files = file_engine.files();
                                                let mut new_files_content = Vec::new();
                                                for file_name in &files {
                                                    if let Some(contents) = file_engine.read_file(file_name).await {
                                                        new_files_content.push((file_name.clone(), contents));
                                                    }
                                                }
                                                uploaded_files_content.set(new_files_content);
                                            }
                                        }
                                    }
                                }
                                if !uploaded_files_content.read().is_empty() {
                                    div {
                                        class: "mt-4 text-sm text-gray-600",
                                        p { class: "font-semibold mb-2", "Archivos seleccionados:" }
                                        ul { class: "list-disc list-inside",
                                            for (file_name, _) in uploaded_files_content.read().iter() {
                                                li { key: "{file_name}", "{file_name}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "mt-10 pt-6 border-t",
                            button {
                                r#type: "submit",
                                class: "w-full bg-green-600 text-white font-bold py-3 px-4 rounded-lg hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 transition-transform transform hover:scale-105",
                                "Generar Reporte"
                            }
                        }
                    }
                }
            }
        }
    }
}


// --- Componentes de Formulario (sin cambios) ---

#[derive(Props, Clone, PartialEq)]
struct FormFieldSectionProps {
    title: String,
    children: Element,
    grid_cols: Option<u32>,
}

#[allow(non_snake_case)]
fn FormFieldSection(props: FormFieldSectionProps) -> Element {
    let grid_class = format!("grid grid-cols-1 md:grid-cols-{}", props.grid_cols.unwrap_or(4));
    rsx! {
        div {
            class: "p-4 border rounded-lg",
            h3 { class: "text-lg font-semibold text-gray-700 mb-4 border-b pb-2", "{props.title}" },
            div { class: "{grid_class} gap-x-6 gap-y-4", {props.children} }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FormInputProps {
    label: String,
    value: String,
    on_input: EventHandler<String>,
}

#[allow(non_snake_case)]
fn FormInput(props: FormInputProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            label { class: "text-sm font-medium text-gray-600 mb-1", "{props.label}" },
            input {
                r#type: "text",
                class: "px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 transition text-sm",
                oninput: move |evt| props.on_input.call(evt.value()),
                value: "{props.value}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FormTextAreaProps {
    label: String,
    value: String,
    on_input: EventHandler<String>,
}

#[allow(non_snake_case)]
fn FormTextArea(props: FormTextAreaProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            label { class: "text-sm font-medium text-gray-600 mb-1", "{props.label}" },
            textarea {
                class: "px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 transition text-sm h-24",
                oninput: move |evt| props.on_input.call(evt.value()),
                value: "{props.value}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FormSelectProps {
    label: String,
    value: String,
    options: Vec<String>,
    on_change: EventHandler<String>,
}

#[allow(non_snake_case)]
fn FormSelect(props: FormSelectProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            label { class: "text-sm font-medium text-gray-600 mb-1", "{props.label}" },
            select {
                class: "px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 transition bg-white text-sm",
                onchange: move |evt| props.on_change.call(evt.value()),
                option { selected: props.value.is_empty(), disabled: true, value: "", "Seleccione una opción" },
                for option_str in props.options.iter() {
                    option { selected: props.value == *option_str, "{option_str}" }
                }
            }
        }
    }
}