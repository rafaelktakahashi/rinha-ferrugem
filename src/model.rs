use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize, Debug)]
pub struct TrReq {
    pub valor: u32,
    pub tipo: char, // received as char (any Unicode codepoint)
    pub descricao: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct TrRes {
    pub limite: u32,
    pub saldo: i32,
}

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

#[derive(Serialize, Debug)]
pub struct ExtSd {
    pub total: i32,
    pub data_extrato: DateTime<Utc>,
    pub limite: u32,
}

#[derive(Serialize, Debug)]
pub struct ExtRes {
    pub saldo: ExtSd,
    pub ultimas_transacoes: Vec<Tr>,
}
