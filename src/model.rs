use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

/// Incoming DTO. This contains an optional string,
/// even though it's mandatory, because this is an
/// easy way to fail with a specific error when the
/// string is missing, without having to intercept
/// exceptions thrown by serde.
#[derive(Deserialize, Debug)]
pub struct TrReq {
    pub valor: u32,
    pub tipo: char, // received as char (any Unicode codepoint)
    pub descricao: Option<String>,
}

/// Outgoing DTO.
#[derive(Serialize, Debug)]
pub struct TrRes {
    pub limite: u32,
    pub saldo: i32,
}

/// Database entity. We store the type as a boolean
/// since it only admits two possible states.
#[derive(Serialize, Debug, Clone)]
pub struct Tr {
    pub valor: u32,
    #[serde(serialize_with = "serialize_tipo_from_bool")]
    pub tipo: bool, // true for 'c', false for 'd'
    pub descricao: String,
    pub realizada_em: DateTime<Utc>,
}

fn serialize_tipo_from_bool<S>(x: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_char(if *x { 'c' } else { 'd' })
}

/// Outgoing DTO.
/// The date is always serialized with timezone.
#[derive(Serialize, Debug)]
pub struct ExtSd {
    pub total: i32,
    pub data_extrato: DateTime<Utc>,
    pub limite: u32,
}

/// Outgoing DTO.
#[derive(Serialize, Debug)]
pub struct ExtRes {
    pub saldo: ExtSd,
    pub ultimas_transacoes: Vec<Tr>,
}

// Vi el populoso mar, vi el alba y la tarde, vi las muchedumbres de América,
// vi una plateada telaraña en el centro de una negra pirámide, [...] vi todos
// los espejos del planeta y ninguno me reflejó, [...] vi la circulación de mi
// oscura sangre, vi el engrenaje del amor y la modificación de la muerte, [...]
// vi mi cara y mis vísceras, vi tu cara, y sentí vértigo y lloré, porque mis
// ojos habían visto ese objeto secreto y conjectural, cuyo nombre usurpan los
// hombres, pero que ningún hombre ha mirado: el inconcebible universo.
//
// Sentí infinita veneración, infinita lástima.
//
// Jorge Luis Borges, "El Aleph"
