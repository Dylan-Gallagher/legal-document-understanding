use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub struct Relation {
    pub head: String,
    pub tail: String,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum LegalDocument {
    AiAct,
    GDPR,
    DGA,
}

impl fmt::Display for LegalDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LegalDocument::AiAct => write!(f, "AI Act"),
            LegalDocument::GDPR => write!(f, "GDPR"),
            LegalDocument::DGA => write!(f, "DGA"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalTexts {
    #[serde(rename = "AI Act")]
    pub ai_act: Act,
    #[serde(rename = "GDPR")]
    pub gdpr: Act,
    #[serde(rename = "DGA")]
    pub dga: Act,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Act {
    #[serde(flatten)]
    pub articles: HashMap<String, Article>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    #[serde(flatten)]
    pub points: HashMap<String, PointValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointValue {
    Sentence(String),
    Subpoints(HashMap<String, String>),
}

pub struct ArticleExtractor {
    config: HashMap<String, String>,
    legal_texts: Option<LegalTexts>,
    official_names: HashMap<LegalDocument, &'static str>,
    pub external_references: Option<Vec<Relation>>,
}

impl ArticleExtractor {
    pub fn new(config: HashMap<String, String>) -> Self {
        let official_names = HashMap::from([
            (LegalDocument::GDPR, "Regulation (EU) 2016/679"),
            (LegalDocument::DGA, "Regulation (EU) 2018/1724"),
            (LegalDocument::AiAct, "Regulation(EU) 2024/1689"),
        ]);

        ArticleExtractor {
            config,
            legal_texts: None,
            official_names,
            external_references: None,
        }
    }

    fn parse_legal_texts(&mut self) {
        // Read in legal_texts.json
        let legal_texts_file = File::open("../legal_texts.json").unwrap();
        let reader = BufReader::new(legal_texts_file);
        let legal_texts: LegalTexts = serde_json::from_reader(reader).unwrap();

        self.legal_texts = Some(legal_texts)
    }

    fn find_internal_links(&self) {}

    fn find_external_links(&mut self, source: &LegalDocument, target: &LegalDocument) {
        let mut references: Vec<Relation> = Vec::new();

        if let Some(legal_text) = self.legal_texts.borrow() {
            let source_regulation = match source {
                LegalDocument::AiAct => legal_text.ai_act.borrow(),
                LegalDocument::GDPR => legal_text.gdpr.borrow(),
                LegalDocument::DGA => legal_text.dga.borrow(),
            };
            for (article_name, article) in source_regulation.articles.iter() {
                for (_point_name, point) in article.points.iter() {
                    match point {
                        PointValue::Sentence(sent) => {
                            if sent.contains(self.official_names.get(target).unwrap()) {
                                references.push(Relation {
                                    head: String::from(article_name),
                                    tail: target.to_string(),
                                })
                            }
                        }
                        PointValue::Subpoints(map) => {
                            for (_subpoint_name, subpoint) in map.iter() {
                                if subpoint.contains(self.official_names.get(target).unwrap()) {
                                    references.push(Relation {
                                        head: String::from(article_name),
                                        tail: target.to_string(),
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }

        self.external_references = Some(references);
    }

    pub fn run(&mut self, source: &LegalDocument, target: &LegalDocument) {
        self.parse_legal_texts();
        if self.config.get("extraction_method").unwrap() == "internal" {
            self.find_internal_links();
        } else {
            // external
            self.find_external_links(source, target);
        }
    }
}

// fn main() {
//     // let test_str = String::from("As mentioned in Regulation (EU) 2016/679, it was very successful");
//     // let test_str_ref = &test_str;

//     let config = HashMap::from([(String::from("extraction_method"), String::from("external"))]);
//     let mut article_extractor = ArticleExtractor::new(config);
//     article_extractor.run("AI Act", "GDPR");
//     println!("{:#?}", article_extractor.external_references.unwrap());
// }
