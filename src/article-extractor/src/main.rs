use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

struct Relation {
    head: String,
    tail: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegalText {
    #[serde(rename = "AI Act")]
    ai_act: HashMap<String, Article>,
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
    Subpoints { subpoints: HashMap<String, String> },
}

struct ArticleExtractor {
    config: HashMap<String, String>,
    legal_texts: Option<LegalText>,
    official_names: HashMap<&'static str, &'static str>,
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
        }
    }

    fn parse_legal_texts(&mut self) {
        // Read in legal_texts.json
        let legal_texts_file = File::open("../legal_texts.json").unwrap();
        let reader = BufReader::new(legal_texts_file);
        let legal_texts: LegalText = serde_json::from_reader(reader).unwrap();
        print!("{legal_texts:#?}");
        self.legal_texts = Some(legal_texts)
    }

    fn find_internal_links(&self) {}

    fn find_external_links(&self, source: &str, target: &str) {
        let references: Vec<Relation> = Vec::new();
        if let Some(map) = self.legal_texts.borrow() {
            // TODO: Change `map.ai_act` to `map.<source>`
            for (article_name, article) in map.ai_act.iter() {
                for (point_name, point) in article.points.iter() {
                    match point {
                        PointValue::Sentence(sent) => {}
                        PointValue::Subpoints { subpoints } => {}
                    }
                }
            }
        }
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
    let config = HashMap::from([(String::from("extraction_method"), String::from("external"))]);
    let mut article_extractor = ArticleExtractor::new(config);
    article_extractor.run("AI Act", "GDPR");
}
