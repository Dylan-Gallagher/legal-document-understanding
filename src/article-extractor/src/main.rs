use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
struct Relation {
    head: String,
    tail: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegalTexts {
    #[serde(rename = "AI Act")]
    ai_act: Act,
    #[serde(rename = "GDPR")]
    gdpr: Act,
    #[serde(rename = "DGA")]
    dga: Act,
}

#[derive(Debug, Serialize, Deserialize)]
struct Act {
    #[serde(flatten)]
    articles: HashMap<String, Article>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Article {
    #[serde(flatten)]
    points: HashMap<String, PointValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum PointValue {
    Sentence(String),
    Subpoints(HashMap<String, String>),
}

struct ArticleExtractor {
    config: HashMap<String, String>,
    legal_texts: Option<LegalTexts>,
    official_names: HashMap<&'static str, &'static str>,
    external_references: Option<Vec<Relation>>,
}

impl ArticleExtractor {
    fn new(config: HashMap<String, String>) -> Self {
        let official_names = HashMap::from([
            ("GDPR", "Regulation (EU) 2016/679"),
            ("DGA", "Regulation (EU) 2018/1724"),
            ("AI Act", "Regulation(EU) 2024/1689"),
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

    fn find_external_links(&mut self, source: &str, target: &str) {
        let mut references: Vec<Relation> = Vec::new();

        if let Some(legal_text) = self.legal_texts.borrow() {
            let source_regulation = match source {
                "AI Act" => legal_text.ai_act.borrow(),
                "GDPR" => legal_text.gdpr.borrow(),
                "DGA" => legal_text.dga.borrow(),
                _ => legal_text.ai_act.borrow(), // default to ai act
            };
            // TODO: Change `map.ai_act` to `map.<source>`
            for (article_name, article) in source_regulation.articles.iter() {
                for (_point_name, point) in article.points.iter() {
                    match point {
                        PointValue::Sentence(sent) => {
                            if sent.contains(self.official_names.get(target).unwrap()) {
                                references.push(Relation {
                                    head: String::from(article_name),
                                    tail: String::from(target),
                                })
                            }
                        }
                        PointValue::Subpoints(map) => {
                            for (_subpoint_name, subpoint) in map.iter() {
                                if subpoint.contains(self.official_names.get(target).unwrap()) {
                                    references.push(Relation {
                                        head: String::from(article_name),
                                        tail: String::from(target),
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

    fn run(&mut self, source: &str, target: &str) {
        self.parse_legal_texts();
        if self.config.get("extraction_method").unwrap() == "internal" {
            self.find_internal_links();
        } else {
            // external
            self.find_external_links(source, target);
        }
    }
}

fn main() {
    // let test_str = String::from("As mentioned in Regulation (EU) 2016/679, it was very successful");
    // let test_str_ref = &test_str;

    let config = HashMap::from([(String::from("extraction_method"), String::from("external"))]);
    let mut article_extractor = ArticleExtractor::new(config);
    article_extractor.run("AI Act", "GDPR");
    println!("{:#?}", article_extractor.external_references.unwrap());
}
