use std::fmt;
use serde::{Deserialize, Serialize}; 

/// Languages used in API calls.
/// 
/// See <https://partner.steamgames.com/doc/store/localization/languages> for more information.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[derive(Default)]
pub enum Language {
    Arabic,
    Bulgarian,
    ChineseSimplified,
    ChineseTraditional,
    Czech,
    Danish,
    Dutch,
    #[default]
    English,
    Finnish,
    French,
    German,
    Greek,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Norwegian,
    Polish,
    Portuguese,
    PortugueseBrazil,
    Romanian,
    Russian,
    SpanishSpain,
    SpanishLatinAmerica,
    Swedish,
    Thai,
    Turkish,
    Ukrainian,
    Vietnamese,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.web_api_language_code())
    }
}



impl Language {
    /// API language codes are used with the clientside APIs in the 
    /// [Steamworks API](https://partner.steamgames.com/doc/sdk/api).
    pub fn web_api_language_code(&self) -> &'static str {
        match self {
            Self::Arabic => "ar",
            Self::Bulgarian => "bg",
            Self::ChineseSimplified => "zh-CN",
            Self::ChineseTraditional => "zh-TW",
            Self::Czech => "cs",
            Self::Danish => "da",
            Self::Dutch => "nl",
            Self::English => "en",
            Self::Finnish => "fi",
            Self::French => "fr",
            Self::German => "de",
            Self::Greek => "el",
            Self::Hungarian => "hu",
            Self::Italian => "it",
            Self::Japanese => "ja",
            Self::Korean => "ko",
            Self::Norwegian => "no",
            Self::Polish => "pl",
            Self::Portuguese => "pt",
            Self::PortugueseBrazil => "pt-BR",
            Self::Romanian => "ro",
            Self::Russian => "ru",
            Self::SpanishSpain => "es",
            Self::SpanishLatinAmerica => "es-419",
            Self::Swedish => "sv",
            Self::Thai => "th",
            Self::Turkish => "tr",
            Self::Ukrainian => "uk",
            Self::Vietnamese => "vn",
        }
    }
    
    /// Web API language codes are used with the
    /// [Steamworks Web API](https://partner.steamgames.com/doc/webapi).
    pub fn api_language_code(&self) -> &'static str {
        match self {
            Self::Arabic => "arabic",
            Self::Bulgarian => "bulgarian",
            Self::ChineseSimplified => "schinese",
            Self::ChineseTraditional => "tchinese",
            Self::Czech => "czech",
            Self::Danish => "danish",
            Self::Dutch => "dutch",
            Self::English => "english",
            Self::Finnish => "finnish",
            Self::French => "french",
            Self::German => "german",
            Self::Greek => "greek",
            Self::Hungarian => "hungarian",
            Self::Italian => "italian",
            Self::Japanese => "japanese",
            Self::Korean => "koreana",
            Self::Norwegian => "norwegian",
            Self::Polish => "polish",
            Self::Portuguese => "portuguese",
            Self::PortugueseBrazil => "brazilian",
            Self::Romanian => "romanian",
            Self::Russian => "russian",
            Self::SpanishSpain => "spanish",
            Self::SpanishLatinAmerica => "latam",
            Self::Swedish => "swedish",
            Self::Thai => "thai",
            Self::Turkish => "turkish",
            Self::Ukrainian => "ukrainian",
            Self::Vietnamese => "vietnamese",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        
    #[test]
    fn gets_correct_codes_for_english() {
        let language = Language::English;
        
        assert_eq!(language.web_api_language_code(), "en");
        assert_eq!(language.api_language_code(), "english");
    }
}